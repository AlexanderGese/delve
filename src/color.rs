// all the colours in one place so i'm not typing Color::DarkGrey everywhere.
// these are just crossterm colours with names that make sense to me.

use crossterm::style::Color;

pub const PLAYER: Color = Color::White;
pub const WALL: Color = Color::Grey;
pub const FLOOR: Color = Color::DarkGrey;
pub const STAIRS: Color = Color::Yellow;
pub const REMEMBERED: Color = Color::DarkGrey; // tiles i've seen but can't see right now

// monsters
pub const VERMIN: Color = Color::DarkYellow;
pub const GOBLIN: Color = Color::Green;
pub const ORC: Color = Color::DarkGreen;
pub const UNDEAD: Color = Color::Grey;
pub const BEAST: Color = Color::Red;
pub const DEMON: Color = Color::DarkRed;
pub const DRAGON: Color = Color::Magenta;

// items
pub const POTION: Color = Color::Magenta;
pub const SCROLL: Color = Color::Cyan;
pub const WEAPON: Color = Color::White;
pub const ARMOR: Color = Color::Blue;
pub const GOLD: Color = Color::Yellow;

// ui stuff
pub const HP_GOOD: Color = Color::Green;
pub const HP_WARN: Color = Color::Yellow;
pub const HP_BAD: Color = Color::Red;
pub const TEXT: Color = Color::Grey;
pub const HILITE: Color = Color::White;
