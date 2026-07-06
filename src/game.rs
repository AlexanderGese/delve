// this is the big one - the whole game state plus all the rules. player actions,
// the monsters' turn, going down the stairs, using items, levelling up, dying,
// all of it. it got kind of huge tbh, i should probably split it up but it works.

use crate::ai::{self, Intent};
use crate::color;
use crate::combat;
use crate::entity::Entity;
use crate::flavor;
use crate::fov;
use crate::geom::{Point, Rect};
use crate::items::{Item, Kind, Potion, Scroll};
use crate::log::Log;
use crate::map::Map;
use crate::mapgen;
use crate::rng::Rng;
use crate::spawn;
use crate::tile::Tile;
use std::collections::HashSet;

const FOV_RADIUS: i32 = 8;
const MAX_DEPTH: u32 = 12; // descend past this and you win

#[derive(PartialEq)]
pub enum State {
    Playing,
    Dead,
    Won,
}

pub struct Game {
    pub map: Map,
    pub rooms: Vec<Rect>,
    pub player: Entity,
    pub monsters: Vec<Entity>,
    pub ground: Vec<Item>,
    pub pack: Vec<Item>,
    pub weapon: Option<Item>,
    pub armor: Option<Item>,
    pub depth: u32,
    pub plevel: i32,
    pub xp: i32,
    pub next_xp: i32,
    pub gold: i32,
    pub log: Log,
    pub rng: Rng,
    pub state: State,
    pub turns: u64,
    pub tips: HashSet<&'static str>,
    pub epitaph: String,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut rng = Rng::new(seed);
        let level = mapgen::generate(&mut rng);
        let start = level.rooms[0].center();
        let player = Entity::player(start);
        let (monsters, ground) = spawn::populate(&level.map, &level.rooms, 1, &mut rng);
        let mut g = Game {
            map: level.map,
            rooms: level.rooms,
            player,
            monsters,
            ground,
            pack: Vec::new(),
            weapon: None,
            armor: None,
            depth: 1,
            plevel: 1,
            xp: 0,
            next_xp: 20,
            gold: 0,
            log: Log::new(200),
            rng,
            state: State::Playing,
            turns: 0,
            tips: HashSet::new(),
            epitaph: String::new(),
        };
        g.update_fov();
        let intro = flavor::intro(&mut g.rng);
        g.log.push(intro);
        g.log.push_colored(
            "(tip) Move with WASD or the arrows. Press ? any time for the controls.",
            color::HILITE,
        );
        g
    }

    // show a tutorial hint, but only the FIRST time it comes up. i keep a set of
    // the ones i've already shown so they don't spam the log every single turn
    fn tip(&mut self, key: &'static str, text: &str) {
        if self.tips.insert(key) {
            self.log.push_colored(format!("(tip) {text}"), color::HILITE);
        }
    }

    // look at what's going on around the player and fire the right tips
    pub fn check_tips(&mut self) {
        if self.state != State::Playing {
            return;
        }
        let item_seen = self
            .ground
            .iter()
            .any(|i| i.pos.is_some_and(|q| self.map.is_visible(q)));
        let mon_seen = self.monsters.iter().any(|m| self.map.is_visible(m.pos));
        let hp_low = self.player.stats.hp * 4 <= self.player.stats.max_hp;
        let on_stairs = self.map.tile(self.player.pos) == Tile::StairsDown;

        if item_seen {
            self.tip("item", "That glyph is loot. Stand on it and press E (or g) to grab it.");
        }
        if mon_seen {
            self.tip("monster", "A monster! Walk into it to attack — or just leave. Bravery is optional.");
        }
        if hp_low {
            self.tip("hp", "You're badly hurt. Open your pack with i and quaff a healing potion.");
        }
        if on_stairs {
            self.tip("stairs", "You're on the down stairs. Press E (or >) to descend to your next mistake.");
        }
    }

    pub fn update_fov(&mut self) {
        let p = self.player.pos;
        fov::compute(&mut self.map, p, FOV_RADIUS);
    }

    pub fn player_power(&self) -> i32 {
        let bonus = match &self.weapon {
            Some(Item { kind: Kind::Weapon { power }, .. }) => *power,
            _ => 0,
        };
        self.player.stats.power + bonus
    }

    pub fn player_defense(&self) -> i32 {
        let bonus = match &self.armor {
            Some(Item { kind: Kind::Armor { defense }, .. }) => *defense,
            _ => 0,
        };
        self.player.stats.defense + bonus
    }

    fn monster_at(&self, p: Point) -> Option<usize> {
        self.monsters.iter().position(|m| m.pos == p)
    }

    // ------- player actions (return true when a turn was spent) -------

    pub fn player_move(&mut self, dx: i32, dy: i32) -> bool {
        let dest = self.player.pos.offset(dx, dy);
        if let Some(i) = self.monster_at(dest) {
            let power = self.player_power();
            let def = self.monsters[i].stats.defense;
            let killed = combat::attack("you", power, def, &mut self.monsters[i], &mut self.rng, &mut self.log);
            if killed {
                let name = self.monsters[i].name.clone();
                let xp = self.monsters[i].xp;
                let line = flavor::kill(&mut self.rng, &name);
                self.log.push_colored(line, color::HP_GOOD);
                self.monsters.remove(i);
                self.gain_xp(xp);
            }
            return true;
        }
        if self.map.walkable(dest) {
            self.player.pos = dest;
            self.update_fov();
            if let Some(it) = self.ground.iter().find(|i| i.pos == Some(dest)) {
                self.log.push(format!("You see {} here.", it.name));
            } else if self.map.tile(dest) == Tile::StairsDown {
                self.log.push("There is a staircase leading down here.");
            }
            true
        } else {
            false
        }
    }

    pub fn wait(&mut self) -> bool {
        // resting recovers a sliver of health
        if self.rng.one_in(4) {
            self.player.stats.heal(1);
        }
        if self.rng.one_in(3) {
            let line = flavor::wait(&mut self.rng);
            self.log.push(line);
        }
        true
    }

    // the E key. if i'm standing on an item, grab it. otherwise if i'm on the
    // down stairs, go down. one button that does the sensible thing.
    pub fn interact(&mut self) -> bool {
        let p = self.player.pos;
        if self.ground.iter().any(|i| i.pos == Some(p)) {
            return self.pickup();
        }
        match self.map.tile(p) {
            Tile::StairsDown => self.descend(),
            Tile::StairsUp => {
                self.log.push("These stairs lead up, toward daylight and shame. No going back.");
                false
            }
            _ => {
                self.log.push("There's nothing here to interact with.");
                false
            }
        }
    }

    pub fn pickup(&mut self) -> bool {
        let p = self.player.pos;
        let Some(idx) = self.ground.iter().position(|i| i.pos == Some(p)) else {
            let line = flavor::empty_pickup(&mut self.rng);
            self.log.push(line);
            return false;
        };
        let mut item = self.ground.remove(idx);
        if let Kind::Gold(amount) = item.kind {
            self.gold += amount;
            self.log.push_colored(format!("You pick up {amount} gold."), color::GOLD);
        } else {
            self.log.push(format!("You pick up {}.", item.name));
            item.pos = None;
            self.pack.push(item);
        }
        true
    }

    pub fn descend(&mut self) -> bool {
        if self.map.tile(self.player.pos) != Tile::StairsDown {
            self.log.push("There are no stairs down here.");
            return false;
        }
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            self.state = State::Won;
            return false;
        }
        let level = mapgen::generate(&mut self.rng);
        let start = level.rooms[0].center();
        let (monsters, ground) = spawn::populate(&level.map, &level.rooms, self.depth, &mut self.rng);
        self.map = level.map;
        self.rooms = level.rooms;
        self.player.pos = start;
        self.monsters = monsters;
        self.ground = ground;
        self.update_fov();
        let f = flavor::descend(&mut self.rng);
        self.log.push_colored(format!("You descend to depth {}. {f}", self.depth), color::STAIRS);
        true
    }

    // use or equip pack item number `idx`. returns true if it used up a turn
    pub fn use_item(&mut self, idx: usize) -> bool {
        if idx >= self.pack.len() {
            return false;
        }
        match self.pack[idx].kind.clone() {
            Kind::Potion(p) => {
                let item = self.pack.remove(idx);
                self.quaff(p, &item.name);
                true
            }
            Kind::Scroll(s) => {
                let item = self.pack.remove(idx);
                self.read_scroll(s, &item.name);
                true
            }
            Kind::Weapon { .. } => {
                let item = self.pack.remove(idx);
                let name = item.name.clone();
                if let Some(old) = self.weapon.replace(item) {
                    self.pack.push(old);
                }
                self.log.push(format!("You wield the {name}."));
                true
            }
            Kind::Armor { .. } => {
                let item = self.pack.remove(idx);
                let name = item.name.clone();
                if let Some(old) = self.armor.replace(item) {
                    self.pack.push(old);
                }
                self.log.push(format!("You put on the {name}."));
                true
            }
            Kind::Gold(_) => false,
        }
    }

    fn quaff(&mut self, p: Potion, name: &str) {
        match p {
            Potion::Healing => {
                let amount = self.rng.range(12, 24);
                let got = self.player.stats.heal(amount);
                self.log.push_colored(format!("You drink the {name}. You recover {got} HP."), color::HP_GOOD);
            }
            Potion::Strength => {
                self.player.stats.power += 1;
                self.log.push_colored(format!("You drink the {name}. You feel stronger!"), color::HP_GOOD);
            }
            Potion::Vitality => {
                self.player.stats.max_hp += 5;
                self.player.stats.hp += 5;
                self.log.push_colored(format!("You drink the {name}. You feel more robust!"), color::HP_GOOD);
            }
        }
    }

    fn read_scroll(&mut self, s: Scroll, name: &str) {
        self.log.push(format!("You read the {name}."));
        match s {
            Scroll::Teleport => {
                if let Some(dest) = self.random_floor() {
                    self.player.pos = dest;
                    self.update_fov();
                    self.log.push("The world blurs — you are somewhere else.");
                }
            }
            Scroll::MagicMapping => {
                self.map.reveal_all();
                self.log.push("The layout of the level floods into your mind.");
            }
            Scroll::Lightning => {
                if let Some(i) = self.nearest_visible_monster() {
                    let dmg = self.rng.range(10, 16);
                    self.monsters[i].stats.damage(dmg);
                    let mname = self.monsters[i].name.clone();
                    self.log.push_colored(format!("Lightning arcs into the {mname} for {dmg} damage!"), color::HILITE);
                    if self.monsters[i].is_dead() {
                        let mkname = self.monsters[i].name.clone();
                        let xp = self.monsters[i].xp;
                        let line = flavor::kill(&mut self.rng, &mkname);
                        self.log.push_colored(line, color::HP_GOOD);
                        self.monsters.remove(i);
                        self.gain_xp(xp);
                    }
                } else {
                    self.log.push("The lightning fizzles — nothing in sight.");
                }
            }
            Scroll::EnchantWeapon => {
                if let Some(Item { kind: Kind::Weapon { power }, name: wname, .. }) = self.weapon.as_mut() {
                    *power += 1;
                    self.log.push_colored(format!("Your {wname} glows sharper."), color::HILITE);
                } else {
                    self.log.push("You have no weapon to enchant.");
                }
            }
        }
    }

    fn random_floor(&mut self) -> Option<Point> {
        for _ in 0..200 {
            let x = self.rng.range(1, self.map.w - 1);
            let y = self.rng.range(1, self.map.h - 1);
            let p = Point::new(x, y);
            if self.map.walkable(p) && self.monster_at(p).is_none() && p != self.player.pos {
                return Some(p);
            }
        }
        None
    }

    fn nearest_visible_monster(&self) -> Option<usize> {
        let p = self.player.pos;
        self.monsters
            .iter()
            .enumerate()
            .filter(|(_, m)| self.map.is_visible(m.pos))
            .min_by_key(|(_, m)| m.pos.dist2(p))
            .map(|(i, _)| i)
    }

    // hand out xp and level up if we crossed the threshold. the loop is in case
    // one big kill is enough for two levels at once
    fn gain_xp(&mut self, amount: i32) {
        self.xp += amount;
        while self.xp >= self.next_xp {
            self.xp -= self.next_xp;
            self.plevel += 1;
            // each level needs ~50% more than the last. i just made these
            // numbers up and playtested until it felt ok
            self.next_xp = self.next_xp * 3 / 2 + 10;
            self.player.stats.max_hp += 6;
            self.player.stats.hp = self.player.stats.max_hp;
            self.player.stats.power += 1;
            let f = flavor::level_up(&mut self.rng);
            self.log.push_colored(format!("{f} You are now level {}.", self.plevel), color::HP_GOOD);
        }
    }

    // ------- the monsters' turn -------

    pub fn monsters_turn(&mut self) {
        let ppos = self.player.pos;
        let mut occ: HashSet<Point> = self.monsters.iter().map(|m| m.pos).collect();
        occ.insert(ppos);

        let mut i = 0;
        while i < self.monsters.len() {
            // the borrow checker HATED this. i can't just hand `self` to decide()
            // because it needs to look at the other monsters AND move this one.
            // so i split the borrow of self up by hand (map, rng, and this one
            // monster), and use the snapshot `occ` set for who's standing where
            // instead of touching self again. took me ages to get to compile.
            let intent = {
                let map = &self.map;
                let rng = &mut self.rng;
                let m = &mut self.monsters[i];
                let self_pos = m.pos;
                let occupied = |p: Point| p != self_pos && occ.contains(&p);
                ai::decide(m, ppos, map, &occupied, rng)
            };
            match intent {
                Intent::Attack => {
                    let name = self.monsters[i].name.clone();
                    let power = self.monsters[i].stats.power;
                    let def = self.player_defense();
                    let killed = combat::attack(&name, power, def, &mut self.player, &mut self.rng, &mut self.log);
                    if killed {
                        self.epitaph = flavor::epitaph(&mut self.rng);
                        self.state = State::Dead;
                        return;
                    }
                }
                Intent::Step(to) => {
                    if !occ.contains(&to) {
                        let from = self.monsters[i].pos;
                        occ.remove(&from);
                        occ.insert(to);
                        self.monsters[i].pos = to;
                    }
                }
                Intent::Wait => {}
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Game, State};
    use crate::rng::Rng;

    // Drive the game through many random turns across several seeds; it must
    // never panic and core invariants must always hold.
    #[test]
    fn random_play_is_stable() {
        for seed in 0..8u64 {
            let mut g = Game::new(seed.wrapping_mul(0x9E37_79B9));
            let mut rng = Rng::new(seed ^ 0xABCD);
            for _ in 0..3000 {
                if g.state != State::Playing {
                    g = Game::new(rng.next_u64());
                }
                match rng.below(9) {
                    0 => { g.player_move(-1, 0); }
                    1 => { g.player_move(1, 0); }
                    2 => { g.player_move(0, -1); }
                    3 => { g.player_move(0, 1); }
                    4 => { g.player_move(1, 1); }
                    5 => { g.pickup(); }
                    6 => {
                        if !g.pack.is_empty() {
                            let idx = rng.below(g.pack.len() as u32) as usize;
                            g.use_item(idx);
                        }
                    }
                    7 => { g.descend(); }
                    _ => { g.wait(); }
                }
                if g.state == State::Playing {
                    g.monsters_turn();
                }
                assert!(g.player.stats.hp <= g.player.stats.max_hp);
                assert!(g.map.in_bounds(g.player.pos));
                assert!(g.depth >= 1);
            }
        }
    }
}
