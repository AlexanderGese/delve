// makes the actual dungeon layout. i throw down a bunch of random rooms that
// don't overlap, connect each one to the one before it with an L-shaped
// corridor, then drop the up-stairs in the first room and the down-stairs in
// the last. classic "rooms and corridors", nothing clever but it works.

use crate::geom::{Point, Rect};
use crate::map::{MAP_H, MAP_W, Map};
use crate::rng::Rng;
use crate::tile::Tile;

pub struct Level {
    pub map: Map,
    pub rooms: Vec<Rect>,
}

const MAX_ROOMS: i32 = 16;
const ROOM_MIN: i32 = 5;
const ROOM_MAX: i32 = 12;

pub fn generate(rng: &mut Rng) -> Level {
    let mut map = Map::new(MAP_W, MAP_H);
    let mut rooms: Vec<Rect> = Vec::new();

    // try to place a bunch of rooms. some attempts fail because they'd overlap
    // an existing room, we just skip those and that's why we don't always get
    // MAX_ROOMS rooms
    for _ in 0..MAX_ROOMS {
        let w = rng.range(ROOM_MIN, ROOM_MAX);
        let h = rng.range(ROOM_MIN, ROOM_MAX);
        let x = rng.range(1, MAP_W - w - 1);
        let y = rng.range(1, MAP_H - h - 1);
        let room = Rect::new(x, y, w, h);

        // check it against every room we've placed so far
        let mut overlaps = false;
        for r in &rooms {
            if room.intersects(r) {
                overlaps = true;
                break;
            }
        }
        if overlaps {
            continue;
        }

        carve_room(&mut map, &room);
        if let Some(prev) = rooms.last() {
            connect(&mut map, prev.center(), room.center(), rng);
        }
        rooms.push(room);
    }

    // put the stairs down: up in the first room, down in the last one
    if let Some(first) = rooms.first().copied() {
        map.set(first.center(), Tile::StairsUp);
    }
    if let Some(last) = rooms.last().copied() {
        map.set(last.center(), Tile::StairsDown);
    }

    Level { map, rooms }
}

fn carve_room(map: &mut Map, r: &Rect) {
    for y in r.y1..r.y2 {
        for x in r.x1..r.x2 {
            map.set(Point::new(x, y), Tile::Floor);
        }
    }
}

// connect two points with an L-shaped corridor. i flip a coin for whether to go
// horizontal-then-vertical or the other way, otherwise every hallway bends the
// same direction and it looks weird
fn connect(map: &mut Map, a: Point, b: Point, rng: &mut Rng) {
    if rng.chance(1, 2) {
        h_corridor(map, a.x, b.x, a.y);
        v_corridor(map, a.y, b.y, b.x);
    } else {
        v_corridor(map, a.y, b.y, a.x);
        h_corridor(map, a.x, b.x, b.y);
    }
}

fn dig(map: &mut Map, p: Point) {
    if map.tile(p) == Tile::Wall {
        map.set(p, Tile::Corridor);
    }
}

fn h_corridor(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in x1.min(x2)..=x1.max(x2) {
        dig(map, Point::new(x, y));
    }
}

fn v_corridor(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in y1.min(y2)..=y1.max(y2) {
        dig(map, Point::new(x, y));
    }
}
