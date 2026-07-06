// filling a new level with monsters and loot. goes room by room (skipping the
// first one, that's where the player starts) and drops stuff in, with more and
// nastier monsters the deeper you are.

use crate::entity::Entity;
use crate::geom::{Point, Rect};
use crate::items::{Item, Potion, Scroll};
use crate::map::Map;
use crate::monster::{self, KINDS, Template};
use crate::rng::Rng;
use crate::tile::Tile;

// weapon/armour tables. index roughly by depth so you find better gear lower down
const WEAPONS: &[(&str, i32)] = &[
    ("dagger", 2),
    ("short sword", 3),
    ("mace", 4),
    ("long sword", 5),
    ("battle axe", 6),
    ("war hammer", 7),
    ("greatsword", 9),
];

const ARMORS: &[(&str, i32)] = &[
    ("leather armour", 1),
    ("studded leather", 2),
    ("ring mail", 3),
    ("chain mail", 4),
    ("plate mail", 6),
];

pub fn populate(
    map: &Map,
    rooms: &[Rect],
    depth: u32,
    rng: &mut Rng,
) -> (Vec<Entity>, Vec<Item>) {
    let mut monsters: Vec<Entity> = Vec::new();
    let mut items: Vec<Item> = Vec::new();

    for room in rooms.iter().skip(1) {
        // 0..a few monsters, more allowed the deeper we go
        let max_mon = 2 + (depth as i32 / 3).min(3);
        for _ in 0..rng.range(0, max_mon) {
            if let Some(pos) = free_floor(map, room, rng, &monsters) {
                monsters.push(monster::spawn(pick_monster(depth, rng), pos));
            }
        }
        // 0..2 items per room
        for _ in 0..rng.range(0, 2) {
            if let Some(pos) = free_floor(map, room, rng, &monsters) {
                items.push(random_item(depth, pos, rng));
            }
        }
    }

    (monsters, items)
}

// pick a monster that's allowed on this depth. i also filter out ones that are
// way too weak for how deep we are, so you stop seeing rats on floor 10
fn pick_monster(depth: u32, rng: &mut Rng) -> &'static Template {
    let lo = depth.saturating_sub(6);
    let mut eligible: Vec<&'static Template> = Vec::new();
    for t in KINDS {
        if t.min_depth <= depth && t.min_depth >= lo {
            eligible.push(t);
        }
    }
    if eligible.is_empty() {
        &KINDS[0] // shouldn't happen, but just grab a rat if it does
    } else {
        *rng.pick(&eligible)
    }
}

// roll for what a random item is. mostly gold and potions, sometimes a scroll,
// occasionally a weapon/armour upgrade
fn random_item(depth: u32, pos: Point, rng: &mut Rng) -> Item {
    match rng.below(100) {
        0..40 => Item::gold(rng.range(2, 12 + depth as i32 * 4), pos),
        40..64 => {
            let p = match rng.below(10) {
                0..6 => Potion::Healing,
                6..8 => Potion::Strength,
                _ => Potion::Vitality,
            };
            Item::potion(p, pos)
        }
        64..84 => {
            let s = match rng.below(10) {
                0..3 => Scroll::Teleport,
                3..6 => Scroll::MagicMapping,
                6..9 => Scroll::Lightning,
                _ => Scroll::EnchantWeapon,
            };
            Item::scroll(s, pos)
        }
        84..93 => {
            let i = ((depth as usize / 2) + rng.below(2) as usize).min(WEAPONS.len() - 1);
            let (name, power) = WEAPONS[i];
            Item::weapon(name, power, pos)
        }
        _ => {
            let i = ((depth as usize / 3) + rng.below(2) as usize).min(ARMORS.len() - 1);
            let (name, def) = ARMORS[i];
            Item::armor(name, def, pos)
        }
    }
}

// find an empty floor tile inside a room. just tries random spots a bunch of
// times and gives up if it can't (rather than doing anything smart)
fn free_floor(map: &Map, room: &Rect, rng: &mut Rng, taken: &[Entity]) -> Option<Point> {
    for _ in 0..20 {
        let x = rng.range(room.x1, room.x2 - 1);
        let y = rng.range(room.y1, room.y2 - 1);
        let p = Point::new(x, y);
        if map.tile(p) == Tile::Floor && !taken.iter().any(|e| e.pos == p) {
            return Some(p);
        }
    }
    None
}
