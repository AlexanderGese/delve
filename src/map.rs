// the dungeon map. it's really just one big 1d vector of tiles that i index
// like it's 2d. it also keeps track of which tiles i can see right now and
// which ones i've seen before (so old rooms stay drawn but dimmer).

use crate::geom::Point;
use crate::tile::Tile;

pub const MAP_W: i32 = 80;
pub const MAP_H: i32 = 22;

pub struct Map {
    pub w: i32,
    pub h: i32,
    tiles: Vec<Tile>,
    visible: Vec<bool>,  // can i see it right now
    explored: Vec<bool>, // have i ever seen it
}

impl Map {
    pub fn new(w: i32, h: i32) -> Map {
        let n = (w * h) as usize;
        Map {
            w,
            h,
            tiles: vec![Tile::Wall; n], // start solid, the generator carves it out
            visible: vec![false; n],
            explored: vec![false; n],
        }
    }

    // x,y -> index into the flat vector
    pub fn idx(&self, p: Point) -> usize {
        (p.y * self.w + p.x) as usize
    }

    pub fn in_bounds(&self, p: Point) -> bool {
        p.x >= 0 && p.x < self.w && p.y >= 0 && p.y < self.h
    }

    pub fn tile(&self, p: Point) -> Tile {
        if self.in_bounds(p) {
            self.tiles[self.idx(p)]
        } else {
            Tile::Wall // anything off the map counts as wall
        }
    }

    pub fn set(&mut self, p: Point, t: Tile) {
        if self.in_bounds(p) {
            let i = self.idx(p);
            self.tiles[i] = t;
        }
    }

    pub fn walkable(&self, p: Point) -> bool {
        self.in_bounds(p) && self.tile(p).walkable()
    }

    pub fn opaque(&self, p: Point) -> bool {
        !self.in_bounds(p) || self.tile(p).opaque()
    }

    pub fn clear_visible(&mut self) {
        for v in self.visible.iter_mut() {
            *v = false;
        }
    }

    pub fn is_visible(&self, p: Point) -> bool {
        self.in_bounds(p) && self.visible[self.idx(p)]
    }

    pub fn is_explored(&self, p: Point) -> bool {
        self.in_bounds(p) && self.explored[self.idx(p)]
    }

    pub fn mark_visible(&mut self, p: Point) {
        if self.in_bounds(p) {
            let i = self.idx(p);
            self.visible[i] = true;
            self.explored[i] = true; // if i can see it, it's explored too
        }
    }

    // reveal the whole map - this is what the magic mapping scroll does
    pub fn reveal_all(&mut self) {
        for e in self.explored.iter_mut() {
            *e = true;
        }
    }

    // --- saving ---
    // i turn the tiles into a string of digit codes and explored into 0/1 so i
    // can dump them straight into a text file

    pub fn tile_string(&self) -> String {
        self.tiles.iter().map(|t| (b'0' + t.code()) as char).collect()
    }

    pub fn explored_string(&self) -> String {
        self.explored.iter().map(|&e| if e { '1' } else { '0' }).collect()
    }

    pub fn from_save(w: i32, h: i32, tiles: &str, explored: &str) -> Map {
        let mut m = Map::new(w, h);
        for (i, c) in tiles.chars().enumerate().take(m.tiles.len()) {
            m.tiles[i] = Tile::from_code(c as u8 - b'0');
        }
        for (i, c) in explored.chars().enumerate().take(m.explored.len()) {
            m.explored[i] = c == '1';
        }
        m
    }
}
