// all the monster types. each one is a "template" and i build a real Entity out
// of it when i spawn one. deeper floors unlock the nastier stuff (spawn.rs
// decides which ones can appear).

use crate::color;
use crate::entity::{Ai, Entity, Stats};
use crate::geom::Point;
use crossterm::style::Color;

pub struct Template {
    pub glyph: char,
    pub name: &'static str,
    pub color: Color,
    pub hp: i32,
    pub power: i32,
    pub defense: i32,
    pub xp: i32,
    pub min_depth: u32, // earliest floor this thing can show up on
}

// the whole bestiary. i tweaked these numbers a LOT to get the difficulty to
// feel fair-ish
pub const KINDS: &[Template] = &[
    t('r', "rat", color::VERMIN, 4, 2, 0, 2, 1),
    t('b', "bat", color::VERMIN, 5, 3, 0, 3, 1),
    t('k', "kobold", color::GOBLIN, 7, 3, 0, 4, 1),
    t('g', "goblin", color::GOBLIN, 9, 4, 1, 6, 2),
    t('s', "skeleton", color::UNDEAD, 11, 5, 1, 8, 3),
    t('o', "orc", color::ORC, 14, 6, 2, 12, 4),
    t('z', "zombie", color::UNDEAD, 18, 5, 1, 12, 4),
    t('w', "wolf", color::BEAST, 12, 6, 1, 12, 5),
    t('O', "ogre", color::ORC, 24, 8, 2, 20, 6),
    t('T', "troll", color::BEAST, 30, 10, 3, 30, 8),
    t('D', "demon", color::DEMON, 36, 12, 4, 45, 10),
    t('&', "dragon", color::DRAGON, 50, 14, 5, 80, 12),
];

// just a shorthand so the table above isn't a wall of `Template { ... }`
const fn t(
    glyph: char,
    name: &'static str,
    color: Color,
    hp: i32,
    power: i32,
    defense: i32,
    xp: i32,
    min_depth: u32,
) -> Template {
    Template { glyph, name, color, hp, power, defense, xp, min_depth }
}

// look a monster up by its name - i need this to rebuild monsters from a save
pub fn find_template(name: &str) -> Option<&'static Template> {
    KINDS.iter().find(|t| t.name == name)
}

pub fn spawn(tmpl: &Template, pos: Point) -> Entity {
    Entity {
        pos,
        glyph: tmpl.glyph,
        color: tmpl.color,
        name: tmpl.name.to_string(),
        stats: Stats::new(tmpl.hp, tmpl.power, tmpl.defense),
        ai: Some(Ai::default()), // monsters get an ai, that's what makes them monsters
        xp: tmpl.xp,
    }
}
