// saving/loading a game to a plain text file. i did it the classic roguelike
// way where a save is really just a "suspend" - it gets deleted the moment you
// load it, so you can't save-scum your way out of a bad situation. felt fairer.
// (i hand-wrote the whole format instead of using serde, mostly to see if i could.)

use crate::entity::{Entity, Stats};
use crate::game::{Game, State};
use crate::geom::Point;
use crate::items::{Item, Kind, Potion, Scroll};
use crate::log::Log;
use crate::map::Map;
use crate::monster;
use crate::rng::Rng;
use std::collections::HashSet;
use std::path::PathBuf;

fn save_path() -> PathBuf {
    match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".grave_mistakes.sav"),
        Err(_) => PathBuf::from("grave_mistakes.sav"),
    }
}

pub fn has_save() -> bool {
    save_path().exists()
}

pub fn delete() {
    let _ = std::fs::remove_file(save_path());
}

pub fn save(game: &Game) -> std::io::Result<()> {
    std::fs::write(save_path(), serialize(game))
}

pub fn load() -> Option<Game> {
    let data = std::fs::read_to_string(save_path()).ok()?;
    deserialize(&data)
}

// ---- item (de)serialisation: tab-separated so names can contain spaces ----

fn enc_item(it: &Item) -> String {
    let (tag, param) = match &it.kind {
        Kind::Potion(p) => ('P', *p as i32),
        Kind::Scroll(s) => ('S', *s as i32),
        Kind::Weapon { power } => ('W', *power),
        Kind::Armor { defense } => ('A', *defense),
        Kind::Gold(g) => ('G', *g),
    };
    let (px, py) = it.pos.map_or((-1, -1), |p| (p.x, p.y));
    format!("{tag}\t{param}\t{}\t{px}\t{py}", it.name)
}

fn dec_item(line: &str) -> Option<Item> {
    let f: Vec<&str> = line.split('\t').collect();
    if f.len() < 5 {
        return None;
    }
    let tag = f[0].chars().next()?;
    let param: i32 = f[1].parse().ok()?;
    let name = f[2].to_string();
    let px: i32 = f[3].parse().ok()?;
    let py: i32 = f[4].parse().ok()?;
    let pos = if px < 0 { None } else { Some(Point::new(px, py)) };
    let kind = match tag {
        'P' => Kind::Potion(match param {
            1 => Potion::Strength,
            2 => Potion::Vitality,
            _ => Potion::Healing,
        }),
        'S' => Kind::Scroll(match param {
            1 => Scroll::MagicMapping,
            2 => Scroll::Lightning,
            3 => Scroll::EnchantWeapon,
            _ => Scroll::Teleport,
        }),
        'W' => Kind::Weapon { power: param },
        'A' => Kind::Armor { defense: param },
        'G' => Kind::Gold(param),
        _ => return None,
    };
    Some(Item::rebuild(kind, name, pos))
}

// ---- whole-game (de)serialisation ----

fn serialize(g: &Game) -> String {
    let mut s = String::new();
    s.push_str("GRAVEMISTAKES 1\n");
    let (r0, r1) = g.rng.state();
    s.push_str(&format!("rng {r0} {r1}\n"));
    s.push_str(&format!(
        "prog {} {} {} {} {} {}\n",
        g.depth, g.plevel, g.xp, g.next_xp, g.gold, g.turns
    ));
    let p = &g.player;
    s.push_str(&format!(
        "player {} {} {} {} {} {}\n",
        p.pos.x, p.pos.y, p.stats.max_hp, p.stats.hp, p.stats.power, p.stats.defense
    ));
    s.push_str(&format!("weapon {}\n", g.weapon.as_ref().map_or("-".into(), enc_item)));
    s.push_str(&format!("armor {}\n", g.armor.as_ref().map_or("-".into(), enc_item)));
    s.push_str(&format!("map {} {}\n", g.map.w, g.map.h));
    s.push_str(&format!("tiles {}\n", g.map.tile_string()));
    s.push_str(&format!("explored {}\n", g.map.explored_string()));
    s.push_str(&format!("monsters {}\n", g.monsters.len()));
    for m in &g.monsters {
        s.push_str(&format!("{}\t{}\t{}\t{}\n", m.name, m.pos.x, m.pos.y, m.stats.hp));
    }
    s.push_str(&format!("ground {}\n", g.ground.len()));
    for it in &g.ground {
        s.push_str(&enc_item(it));
        s.push('\n');
    }
    s.push_str(&format!("pack {}\n", g.pack.len()));
    for it in &g.pack {
        s.push_str(&enc_item(it));
        s.push('\n');
    }
    s
}

fn ints(line: &str, prefix: &str) -> Option<Vec<i64>> {
    line.strip_prefix(prefix)?
        .split_whitespace()
        .map(|t| t.parse::<i64>().ok())
        .collect()
}

fn deserialize(data: &str) -> Option<Game> {
    let lines: Vec<&str> = data.lines().collect();
    let mut i = 0;
    let mut next = || {
        let l = lines.get(i).copied();
        i += 1;
        l
    };

    if next()? != "GRAVEMISTAKES 1" {
        return None;
    }
    // NOTE: parse these as u64, not i64! the rng state can be bigger than
    // i64::MAX and my load kept silently failing until i figured that out
    let mut rp = next()?.strip_prefix("rng ")?.split_whitespace();
    let rng = Rng::from_state(rp.next()?.parse().ok()?, rp.next()?.parse().ok()?);

    let pr = ints(next()?, "prog ")?;
    let (depth, plevel, xp, next_xp, gold, turns) = (
        *pr.first()? as u32,
        *pr.get(1)? as i32,
        *pr.get(2)? as i32,
        *pr.get(3)? as i32,
        *pr.get(4)? as i32,
        *pr.get(5)? as u64,
    );

    let pl = ints(next()?, "player ")?;
    let mut player = Entity::player(Point::new(*pl.first()? as i32, *pl.get(1)? as i32));
    player.stats = Stats {
        max_hp: *pl.get(2)? as i32,
        hp: *pl.get(3)? as i32,
        power: *pl.get(4)? as i32,
        defense: *pl.get(5)? as i32,
    };

    let weapon = next()?.strip_prefix("weapon ").and_then(|s| dec_item(s));
    let armor = next()?.strip_prefix("armor ").and_then(|s| dec_item(s));

    let dims = ints(next()?, "map ")?;
    let (mw, mh) = (*dims.first()? as i32, *dims.get(1)? as i32);
    let tiles = next()?.strip_prefix("tiles ")?;
    let explored = next()?.strip_prefix("explored ")?;
    let map = Map::from_save(mw, mh, tiles, explored);

    let mcount = *ints(next()?, "monsters ")?.first()? as usize;
    let mut monsters = Vec::new();
    for _ in 0..mcount {
        let f: Vec<&str> = next()?.split('\t').collect();
        if f.len() >= 4 {
            if let Some(t) = monster::find_template(f[0]) {
                let pos = Point::new(f[1].parse().ok()?, f[2].parse().ok()?);
                let mut e = monster::spawn(t, pos);
                e.stats.hp = f[3].parse().ok()?;
                monsters.push(e);
            }
        }
    }

    let gcount = *ints(next()?, "ground ")?.first()? as usize;
    let mut ground = Vec::new();
    for _ in 0..gcount {
        if let Some(it) = dec_item(next()?) {
            ground.push(it);
        }
    }

    let pcount = *ints(next()?, "pack ")?.first()? as usize;
    let mut pack = Vec::new();
    for _ in 0..pcount {
        if let Some(it) = dec_item(next()?) {
            pack.push(it);
        }
    }

    let mut g = Game {
        map,
        rooms: Vec::new(),
        player,
        monsters,
        ground,
        pack,
        weapon,
        armor,
        depth,
        plevel,
        xp,
        next_xp,
        gold,
        log: Log::new(200),
        rng,
        state: State::Playing,
        turns,
        tips: HashSet::new(),
        epitaph: String::new(),
    };
    g.log.push("You wake, resume your descent, and immediately regret it.");
    g.update_fov();
    Some(g)
}

#[cfg(test)]
mod tests {
    use super::{deserialize, serialize};
    use crate::game::Game;

    #[test]
    fn roundtrip_preserves_state() {
        let mut g = Game::new(1234);
        for _ in 0..60 {
            g.player_move(1, 0);
            g.player_move(0, 1);
            g.monsters_turn();
        }
        g.gold = 77;
        g.depth = 3;

        let text = serialize(&g);
        let loaded = deserialize(&text).expect("save must load back");

        assert_eq!(loaded.depth, g.depth);
        assert_eq!(loaded.gold, g.gold);
        assert_eq!(loaded.player.pos, g.player.pos);
        assert_eq!(loaded.player.stats.hp, g.player.stats.hp);
        assert_eq!(loaded.player.stats.max_hp, g.player.stats.max_hp);
        assert_eq!(loaded.monsters.len(), g.monsters.len());
        assert_eq!(loaded.pack.len(), g.pack.len());
        assert_eq!(loaded.ground.len(), g.ground.len());
        assert_eq!(loaded.map.tile_string(), g.map.tile_string());
        assert_eq!(loaded.rng.state(), g.rng.state());
    }
}
