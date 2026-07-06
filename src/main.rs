// Delve - my little ascii roguelike. sets up the terminal (raw mode + alternate
// screen), shows the title, and runs the game loop. still no saving yet.

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

// puts the terminal into game mode, and the Drop impl puts it back so i can't
// forget to reset it even if the game crashes
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
    Quit,
}

enum MenuChoice {
    Resume,
    Quit,
}

fn main() -> io::Result<()> {
    let _term = Terminal::enter()?;
    let mut out = stdout();

    loop {
        let mut game = match title_screen(&mut out)? {
            Start::Quit => return Ok(()),
            Start::New => Game::new(seed()),
        };
        if run(&mut game, &mut out)? {
            return Ok(());
        }
    }
}

fn title_screen(out: &mut impl Write) -> io::Result<Start> {
    loop {
        ui::draw_title(out, false)?;
        match input::read_key()?.code {
            KeyCode::Char('n') | KeyCode::Char('N') => return Ok(Start::New),
            KeyCode::Char('?') => {
                ui::draw_help(out)?;
                input::read_key()?;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(Start::Quit),
            _ => {}
        }
    }
}

// returns true to leave the program, false to go back to the title
fn run(game: &mut Game, out: &mut impl Write) -> io::Result<bool> {
    loop {
        if game.state == State::Playing {
            game.check_tips();
        }
        render::draw(game, out)?;

        if game.state != State::Playing {
            ui::draw_gameover(game, out, game.state == State::Won)?;
            input::read_key()?;
            return Ok(false);
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
        ui::draw_menu(out, false)?;
        match input::read_key()?.code {
            KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::Esc => return Ok(MenuChoice::Resume),
            KeyCode::Char('?') => {
                ui::draw_help(out)?;
                input::read_key()?;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(MenuChoice::Quit),
            _ => {}
        }
    }
}

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
