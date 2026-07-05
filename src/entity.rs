// an "entity" is anything that lives on the map. the player and every monster
// use this exact same struct. the thing that makes something a monster instead
// of the player is just whether it has an `ai`.

use crate::geom::Point;
use crossterm::style::Color;

// combat stats, shared by the player and monsters
#[derive(Clone, Copy)]
pub struct Stats {
    pub max_hp: i32,
    pub hp: i32,
    pub power: i32,   // how hard you hit
    pub defense: i32, // how much damage you soak per hit
}

impl Stats {
    pub fn new(hp: i32, power: i32, defense: i32) -> Stats {
        Stats { max_hp: hp, hp, power, defense }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    // heal, but don't go over max hp. returns how much it actually healed
    pub fn heal(&mut self, amount: i32) -> i32 {
        let before = self.hp;
        self.hp += amount;
        if self.hp > self.max_hp {
            self.hp = self.max_hp;
        }
        self.hp - before
    }

    pub fn damage(&mut self, amount: i32) {
        self.hp -= amount;
    }
}

// a monster's little bit of memory: where it last saw the player, and how many
// turns it's still going to keep chasing after losing sight
#[derive(Clone, Copy, Default)]
pub struct Ai {
    pub target: Option<Point>,
    pub chasing: i32,
}

pub struct Entity {
    pub pos: Point,
    pub glyph: char,
    pub color: Color,
    pub name: String,
    pub stats: Stats,
    pub ai: Option<Ai>, // None = this is the player
    pub xp: i32,        // xp you get for killing it
}

impl Entity {
    pub fn player(pos: Point) -> Entity {
        Entity {
            pos,
            glyph: '@',
            color: crate::color::PLAYER,
            name: "you".into(),
            stats: Stats::new(30, 5, 1), // starting stats, i tweaked these a bunch
            ai: None,
            xp: 0,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.stats.is_dead()
    }
}
