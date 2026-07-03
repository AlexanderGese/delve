// the different kinds of tile a dungeon square can be

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
    Corridor, // it's basically floor but i wanted to tell rooms and hallways apart
    StairsDown,
    StairsUp,
}

impl Tile {
    // can you walk on it
    pub fn walkable(self) -> bool {
        self != Tile::Wall
    }

    // does it block your view (only walls do for now)
    pub fn opaque(self) -> bool {
        self == Tile::Wall
    }

    pub fn glyph(self) -> char {
        match self {
            Tile::Wall => '#',
            Tile::Floor => '.',
            Tile::Corridor => '.',
            Tile::StairsDown => '>',
            Tile::StairsUp => '<',
        }
    }

    // turn a tile into a number so i can save it to a file
    pub fn code(self) -> u8 {
        match self {
            Tile::Wall => 0,
            Tile::Floor => 1,
            Tile::Corridor => 2,
            Tile::StairsDown => 3,
            Tile::StairsUp => 4,
        }
    }

    pub fn from_code(c: u8) -> Tile {
        match c {
            1 => Tile::Floor,
            2 => Tile::Corridor,
            3 => Tile::StairsDown,
            4 => Tile::StairsUp,
            _ => Tile::Wall,
        }
    }
}
