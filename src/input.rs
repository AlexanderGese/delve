// reading the keyboard and turning key presses into actions. i support a bunch
// of movement schemes (wasd, arrows, and the classic roguelike hjkl+yubn) so
// whatever you're used to should just work.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;

pub enum Action {
    Move(i32, i32),
    Wait,
    Interact,
    Pickup,
    Inventory,
    Descend,
    Help,
    Menu,
    Quit,
}

// keep reading keys until one of them means something
pub fn read_action() -> io::Result<Action> {
    loop {
        if let Some(a) = to_action(read_key()?) {
            return Ok(a);
        }
    }
}

// wait for one key press
pub fn read_key() -> io::Result<KeyEvent> {
    loop {
        if let Event::Key(k) = event::read()? {
            return Ok(k);
        }
    }
}

fn to_action(k: KeyEvent) -> Option<Action> {
    use KeyCode::*;
    // ctrl+c should always quit
    if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == Char('c') {
        return Some(Action::Quit);
    }
    let a = match k.code {
        // up/down/left/right - wasd, arrows, hjkl, numpad, all mapped to the same thing
        Char('w') | Char('W') | Up | Char('k') | Char('8') => Action::Move(0, -1),
        Char('s') | Char('S') | Down | Char('j') | Char('2') => Action::Move(0, 1),
        Char('a') | Char('A') | Left | Char('h') | Char('4') => Action::Move(-1, 0),
        Char('d') | Char('D') | Right | Char('l') | Char('6') => Action::Move(1, 0),
        // diagonals - the yubn keys and the numpad corners
        Char('y') | Char('7') => Action::Move(-1, -1),
        Char('u') | Char('9') => Action::Move(1, -1),
        Char('b') | Char('1') => Action::Move(-1, 1),
        Char('n') | Char('3') => Action::Move(1, 1),
        // everything else
        Char('.') | Char('5') => Action::Wait,
        Char('e') | Char('E') => Action::Interact,
        Char('g') | Char(',') => Action::Pickup,
        Char('i') | Char('I') => Action::Inventory,
        Char('>') => Action::Descend,
        Char('?') => Action::Help,
        Esc | Char('m') => Action::Menu,
        Char('q') | Char('Q') => Action::Quit,
        _ => return None, // key i don't care about, ignore it
    };
    Some(a)
}
