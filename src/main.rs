// Delve - my little ascii roguelike. this file boots the terminal into game
// mode (raw mode + alternate screen), shows the title, runs the main game loop,
// and makes sure the terminal gets put back to normal on the way out, even if
// something panics.

mod ai;
mod color;
mod combat;
mod entity;
mod flavor;
mod fov;
mod game;
mod geom;
mod input;
mod inventory;
mod items;
mod log;
mod map;
mod mapgen;
mod monster;
mod pathfind;
mod render;
mod rng;
mod save;
mod spawn;
mod tile;
mod ui;

use crossterm::{
    cursor::{Hide, Show},
    event::KeyCode,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use game::{Game, State};
use input::Action;
use std::io::{self, Write, stdout};
use std::time::{SystemTime, UNIX_EPOCH};

// this little struct puts the terminal into "game mode" when i create it, and
// its Drop impl puts everything back when it goes out of scope. that way i can't
// forget to reset the terminal, even if the game crashes. really handy pattern.
struct Terminal;

impl Terminal {
    fn enter() -> io::Result<Terminal> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, Hide)?;
        Ok(Terminal)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = execute!(stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn seed() -> u64 {
    // let me force a seed with an env var - handy for reproducing a specific
    // dungeon when something breaks (or for grabbing screenshots)
    if let Ok(s) = std::env::var("DELVE_SEED") {
        if let Ok(n) = s.parse::<u64>() {
            return n;
        }
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x0DDB_1A5E_5EED)
}

enum Start {
    New,
    Continue,
    Quit,
}

enum MenuChoice {
    Resume,
    SaveQuit,
    Quit,
}

fn main() -> io::Result<()> {
    let _term = Terminal::enter()?;
    let mut out = stdout();

    // Outer loop: title screen -> play a run -> back to the title. Only an
    // explicit Quit / Save & Quit leaves the program; dying just ends the run.
    loop {
        let mut game = match title_screen(&mut out)? {
            Start::Quit => return Ok(()),
            Start::New => Game::new(seed()),
            Start::Continue => match save::load() {
                Some(g) => {
                    save::delete(); // a save is a suspend; consume it
                    g
                }
                None => continue,
            },
        };

        if run(&mut game, &mut out)? {
            return Ok(()); // player chose to leave entirely
        }
    }
}

fn title_screen(out: &mut impl Write) -> io::Result<Start> {
    loop {
        let has_save = save::has_save();
        ui::draw_title(out, has_save)?;
        match input::read_key()?.code {
            KeyCode::Char('n') | KeyCode::Char('N') => return Ok(Start::New),
            KeyCode::Char('c') | KeyCode::Char('C') if has_save => return Ok(Start::Continue),
            KeyCode::Char('?') => {
                ui::draw_help(out)?;
                input::read_key()?;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(Start::Quit),
            _ => {}
        }
    }
}

// play one game until it ends. returns true if the player wants to quit the
// whole program (they picked quit / save & quit), or false if the run just
// ended (they died or won) - in that case main() loops back to the title.
fn run(game: &mut Game, out: &mut impl Write) -> io::Result<bool> {
    loop {
        if game.state == State::Playing {
            game.check_tips();
        }
        render::draw(game, out)?;

        if game.state != State::Playing {
            ui::draw_gameover(game, out, game.state == State::Won)?;
            input::read_key()?;
            return Ok(false); // back to the title screen
        }

        let mut took_turn = false;
        match input::read_action()? {
            Action::Move(dx, dy) => took_turn = game.player_move(dx, dy),
            Action::Wait => took_turn = game.wait(),
            Action::Interact => took_turn = game.interact(),
            Action::Pickup => took_turn = game.pickup(),
            Action::Descend => took_turn = game.descend(),
            Action::Inventory => took_turn = inventory_screen(game, out)?,
            Action::Help => {
                ui::draw_help(out)?;
                input::read_key()?;
            }
            Action::Menu => match pause_menu(out)? {
                MenuChoice::Resume => {}
                MenuChoice::SaveQuit => {
                    save::save(game)?;
                    return Ok(true);
                }
                MenuChoice::Quit => return Ok(true),
            },
            Action::Quit => return Ok(true),
        }

        if took_turn && game.state == State::Playing {
            game.monsters_turn();
            game.turns += 1;
        }
    }
}

fn pause_menu(out: &mut impl Write) -> io::Result<MenuChoice> {
    loop {
        ui::draw_menu(out, true)?;
        match input::read_key()?.code {
            KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::Esc => return Ok(MenuChoice::Resume),
            KeyCode::Char('s') | KeyCode::Char('S') => return Ok(MenuChoice::SaveQuit),
            KeyCode::Char('?') => {
                ui::draw_help(out)?;
                input::read_key()?;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(MenuChoice::Quit),
            _ => {}
        }
    }
}

// open the inventory and let the player pick something to use/equip. returns
// whether it used up a turn (drinking a potion does, just closing the menu doesn't)
fn inventory_screen(game: &mut Game, out: &mut impl Write) -> io::Result<bool> {
    ui::draw_inventory(game, out)?;
    match input::read_key()?.code {
        KeyCode::Char(c) => {
            if let Some(idx) = inventory::index_of(c) {
                if idx < game.pack.len() {
                    return Ok(game.use_item(idx));
                }
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}
