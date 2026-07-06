// little helpers for showing the inventory screen. not much here, the actual
// using/equipping happens in game.rs

use crate::items::Item;

// which letter to press for the i-th item (a, b, c, ...)
pub fn letter(i: usize) -> char {
    (b'a' + (i as u8 % 26)) as char
}

// turn a pressed letter back into an index
pub fn index_of(c: char) -> Option<usize> {
    if c.is_ascii_lowercase() {
        Some((c as u8 - b'a') as usize)
    } else {
        None
    }
}

// one line describing an item for the inventory list
pub fn describe(item: &Item, equipped: bool) -> String {
    let mut s = item.name.clone();
    if let Some(b) = item.bonus() {
        s = format!("{s} ({b})");
    }
    if equipped {
        s.push_str("  [equipped]");
    }
    s
}
