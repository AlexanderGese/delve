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
}
