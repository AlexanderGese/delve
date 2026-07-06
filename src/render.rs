// drawing the world to the terminal with crossterm. tiles i can see get their
// real colour, tiles i've been to but can't see right now are drawn dim, and
// tiles i've never seen are just blank.

use crate::color;
use crate::game::Game;
use crate::geom::Point;
use crate::tile::Tile;
use crate::ui;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::Write;

pub const MAP_Y0: u16 = 1; // row 0 is the status bar, so the map starts on row 1

pub fn draw(game: &Game, out: &mut impl Write) -> std::io::Result<()> {
    let m = &game.map;
    // just redraw every cell every frame. it's an 80x22 grid so it's tiny, no
    // point being clever about only redrawing what changed
    for y in 0..m.h {
        for x in 0..m.w {
            let p = Point::new(x, y);
            let (ch, col) = cell(game, p);
            queue!(
                out,
                MoveTo(x as u16, MAP_Y0 + y as u16),
                SetForegroundColor(col),
                Print(ch)
            )?;
        }
    }
    ui::draw_hud(game, out)?;
    ui::draw_log(game, out)?;
    ui::draw_keybar(out)?;
    queue!(out, ResetColor)?;
    out.flush() // nothing actually shows up until i flush
}

fn tile_color(t: Tile) -> Color {
    match t {
        Tile::Wall => color::WALL,
        Tile::Floor | Tile::Corridor => color::FLOOR,
        Tile::StairsDown | Tile::StairsUp => color::STAIRS,
    }
}

// work out what to draw at a tile. order of priority: me > a monster > an item
// > the floor/wall itself
fn cell(game: &Game, p: Point) -> (char, Color) {
    let m = &game.map;
    if game.player.pos == p {
        return ('@', color::PLAYER);
    }
    if m.is_visible(p) {
        if let Some(mon) = game.monsters.iter().find(|e| e.pos == p) {
            return (mon.glyph, mon.color);
        }
        if let Some(it) = game.ground.iter().find(|i| i.pos == Some(p)) {
            return (it.glyph, it.color);
        }
        let t = m.tile(p);
        (t.glyph(), tile_color(t))
    } else if m.is_explored(p) {
        // seen before but not right now - draw it dim, and don't show monsters
        // or items i can't currently see
        (m.tile(p).glyph(), color::REMEMBERED)
    } else {
        (' ', color::FLOOR)
    }
}
