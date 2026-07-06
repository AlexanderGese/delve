// all the screen stuff: the top status bar, the message log, the keybind bar at
// the bottom, and the fullscreen menus (title, pause, inventory, help, game
// over). it's all just ascii in coloured cells, no graphics.

use crate::color;
use crate::game::Game;
use crate::inventory;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::Write;

// where things go on screen. map is rows 1..23, log below that, keybar last.
const LOG_Y0: u16 = 23;
const LOG_ROWS: usize = 3;
const KEYBAR_Y: u16 = 26;

pub fn draw_hud(game: &Game, out: &mut impl Write) -> std::io::Result<()> {
    queue!(out, MoveTo(0, 0), Clear(ClearType::CurrentLine))?;
    let hp = game.player.stats.hp.max(0);
    let max = game.player.stats.max_hp;
    // colour the hp green/yellow/red depending on how bad it is
    let hpcol = if hp * 3 <= max {
        color::HP_BAD
    } else if hp * 3 <= max * 2 {
        color::HP_WARN
    } else {
        color::HP_GOOD
    };
    let weapon = game.weapon.as_ref().map(|i| i.name.as_str()).unwrap_or("fists");
    let armor = game.armor.as_ref().map(|i| i.name.as_str()).unwrap_or("none");
    queue!(
        out,
        MoveTo(0, 0),
        SetForegroundColor(hpcol),
        Print(format!("HP {hp}/{max}")),
        SetForegroundColor(color::TEXT),
        Print(format!(
            "  Lv {}  XP {}/{}  Depth {}  Gold {}  wpn:{}  arm:{}",
            game.plevel, game.xp, game.next_xp, game.depth, game.gold, weapon, armor
        )),
        ResetColor
    )
}

pub fn draw_log(game: &Game, out: &mut impl Write) -> std::io::Result<()> {
    // wipe the old log lines first
    for row in 0..LOG_ROWS {
        queue!(out, MoveTo(0, LOG_Y0 + row as u16), Clear(ClearType::CurrentLine))?;
    }
    for (i, entry) in game.log.recent(LOG_ROWS).enumerate() {
        queue!(
            out,
            MoveTo(0, LOG_Y0 + i as u16),
            SetForegroundColor(entry.color),
            Print(&entry.text)
        )?;
    }
    queue!(out, ResetColor)
}

// the little controls reminder along the very bottom
pub fn draw_keybar(out: &mut impl Write) -> std::io::Result<()> {
    queue!(
        out,
        MoveTo(0, KEYBAR_Y),
        Clear(ClearType::CurrentLine),
        SetForegroundColor(color::FLOOR),
        Print("WASD move  E interact  I inventory  > descend  ? help  M menu  Q quit"),
        ResetColor
    )
}

pub fn draw_inventory(game: &Game, out: &mut impl Write) -> std::io::Result<()> {
    queue!(out, Clear(ClearType::All))?;
    line(out, 1, color::HILITE, "Inventory   (press a-z to use or equip, Esc to close)")?;
    let mut row = 3u16;
    if let Some(w) = &game.weapon {
        line(out, row, color::WEAPON, &format!("wielding: {}", inventory::describe(w, false)))?;
        row += 1;
    }
    if let Some(a) = &game.armor {
        line(out, row, color::ARMOR, &format!("wearing:  {}", inventory::describe(a, false)))?;
        row += 1;
    }
    row += 1; // blank line before the pack list
    if game.pack.is_empty() {
        line(out, row, color::TEXT, "(your pack is tragically empty)")?;
    } else {
        for (i, it) in game.pack.iter().enumerate() {
            line(
                out,
                row,
                it.color,
                &format!("{}) {}", inventory::letter(i), inventory::describe(it, false)),
            )?;
            row += 1;
        }
    }
    queue!(out, ResetColor)?;
    out.flush()
}

pub fn draw_help(out: &mut impl Write) -> std::io::Result<()> {
    queue!(out, Clear(ClearType::All))?;
    let lines = [
        "How to play  (spoiler: badly)",
        "",
        "  W A S D   / arrows / hjkl    move  (walk into a monster to attack)",
        "  Y U B N   / numpad corners   move diagonally",
        "  E                            interact: grab loot, or take the stairs",
        "  G  or  ,                     pick up what's under you",
        "  I                            inventory: use a potion/scroll, equip gear",
        "  >                            descend the stairs",
        "  .  or  5                     wait a turn (very brave)",
        "  M  or  Esc                   menu (save / quit)",
        "  Q                            quit",
        "",
        "Go down. Get loot. Don't die. You will die.",
        "",
        "  (press any key)",
    ];
    for (i, l) in lines.iter().enumerate() {
        line(out, 1 + i as u16, color::TEXT, l)?;
    }
    queue!(out, ResetColor)?;
    out.flush()
}

pub fn draw_title(out: &mut impl Write, has_save: bool) -> std::io::Result<()> {
    queue!(out, Clear(ClearType::All))?;
    // little ascii skull. drawing this in comments/strings is fiddly because of
    // the backslashes so i kept it simple
    let art = [
        r"      .-----.",
        r"     ( x   x )        D E L V E",
        r"      \  ^  /",
        r"       | - |          a little roguelike game",
        r"       '---'          i made",
    ];
    for (i, l) in art.iter().enumerate() {
        line(out, 2 + i as u16, color::HP_BAD, l)?;
    }
    line(out, 9, color::HILITE, "[N]  new game")?;
    if has_save {
        line(out, 10, color::HILITE, "[C]  continue")?;
    }
    line(out, 11, color::HILITE, "[?]  how to play")?;
    line(out, 12, color::HILITE, "[Q]  quit")?;
    line(out, 15, color::FLOOR, "you are going to lose. that's the fun part.")?;
    queue!(out, ResetColor)?;
    out.flush()
}

pub fn draw_menu(out: &mut impl Write, can_save: bool) -> std::io::Result<()> {
    queue!(out, Clear(ClearType::All))?;
    line(out, 3, color::HILITE, "== paused ==")?;
    line(out, 5, color::TEXT, "[R]  resume")?;
    if can_save {
        line(out, 6, color::TEXT, "[S]  save & quit")?;
    }
    line(out, 7, color::TEXT, "[?]  help")?;
    line(out, 8, color::TEXT, "[Q]  quit without saving")?;
    queue!(out, ResetColor)?;
    out.flush()
}

pub fn draw_gameover(game: &Game, out: &mut impl Write, won: bool) -> std::io::Result<()> {
    queue!(out, Clear(ClearType::All))?;
    if won {
        line(out, 8, color::HP_GOOD, "YOU WIN")?;
        line(out, 9, color::TEXT, "You clawed to the bottom and back out. Nobody will believe you.")?;
    } else {
        line(out, 8, color::HP_BAD, "YOU DIED")?;
        line(out, 9, color::TEXT, &game.epitaph)?; // the random gravestone line
    }
    line(
        out,
        11,
        color::TEXT,
        &format!(
            "Depth {}   Level {}   Gold {}   Turns {}",
            game.depth, game.plevel, game.gold, game.turns
        ),
    )?;
    line(out, 13, color::FLOOR, "(press any key)")?;
    queue!(out, ResetColor)?;
    out.flush()
}

// tiny helper - i print a coloured line at (2, y) all over the place
fn line(out: &mut impl Write, y: u16, color: Color, text: &str) -> std::io::Result<()> {
    queue!(out, MoveTo(2, y), SetForegroundColor(color), Print(text))
}
