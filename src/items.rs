// items you find on the floor and carry in your pack - potions, scrolls,
// weapons, armour and gold.

use crate::color;
use crate::geom::Point;
use crossterm::style::Color;

#[derive(Clone, Copy, PartialEq)]
pub enum Potion {
    Healing,
    Strength, // +1 power permanently
    Vitality, // +max hp permanently
}

#[derive(Clone, Copy, PartialEq)]
pub enum Scroll {
    Teleport,
    MagicMapping,
    Lightning,     // zap the nearest monster i can see
    EnchantWeapon, // +1 to whatever weapon i'm holding
}

#[derive(Clone, PartialEq)]
pub enum Kind {
    Potion(Potion),
    Scroll(Scroll),
    Weapon { power: i32 },
    Armor { defense: i32 },
    Gold(i32),
}

pub struct Item {
    pub pos: Option<Point>, // Some = on the floor, None = in my pack
    pub glyph: char,
    pub color: Color,
    pub name: String,
    pub kind: Kind,
}

impl Item {
    // little helper so the constructors below aren't all copy-pasted
    fn ground(glyph: char, color: Color, name: &str, kind: Kind, pos: Point) -> Item {
        Item { pos: Some(pos), glyph, color, name: name.into(), kind }
    }

    pub fn potion(p: Potion, pos: Point) -> Item {
        let name = match p {
            Potion::Healing => "potion of healing",
            Potion::Strength => "potion of strength",
            Potion::Vitality => "potion of vitality",
        };
        Item::ground('!', color::POTION, name, Kind::Potion(p), pos)
    }

    pub fn scroll(s: Scroll, pos: Point) -> Item {
        let name = match s {
            Scroll::Teleport => "scroll of teleportation",
            Scroll::MagicMapping => "scroll of magic mapping",
            Scroll::Lightning => "scroll of lightning",
            Scroll::EnchantWeapon => "scroll of enchant weapon",
        };
        Item::ground('?', color::SCROLL, name, Kind::Scroll(s), pos)
    }

    pub fn weapon(name: &str, power: i32, pos: Point) -> Item {
        Item::ground('/', color::WEAPON, name, Kind::Weapon { power }, pos)
    }

    pub fn armor(name: &str, defense: i32, pos: Point) -> Item {
        Item::ground('[', color::ARMOR, name, Kind::Armor { defense }, pos)
    }

    pub fn gold(amount: i32, pos: Point) -> Item {
        Item::ground('$', color::GOLD, &format!("{amount} gold"), Kind::Gold(amount), pos)
    }

    // rebuild an item after loading a save. i don't save the glyph/colour since
    // they're always the same for a given kind, so i just look them back up
    pub fn rebuild(kind: Kind, name: String, pos: Option<Point>) -> Item {
        let (glyph, color) = match kind {
            Kind::Potion(_) => ('!', color::POTION),
            Kind::Scroll(_) => ('?', color::SCROLL),
            Kind::Weapon { .. } => ('/', color::WEAPON),
            Kind::Armor { .. } => ('[', color::ARMOR),
            Kind::Gold(_) => ('$', color::GOLD),
        };
        Item { pos, glyph, color, name, kind }
    }

    // the little "+2 power" text shown next to gear in the inventory
    pub fn bonus(&self) -> Option<String> {
        match self.kind {
            Kind::Weapon { power } => Some(format!("+{power} power")),
            Kind::Armor { defense } => Some(format!("+{defense} defense")),
            _ => None,
        }
    }
}
