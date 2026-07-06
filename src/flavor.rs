// all the funny random text lives here. i made little pools of one-liners and
// pick one at random so it's not the same boring message every time. honestly
// writing these was the most fun part of the whole thing.

use crate::rng::Rng;

// pull one random line out of a list
fn one(rng: &mut Rng, lines: &[&str]) -> String {
    (*rng.pick(lines)).to_string()
}

pub fn intro(rng: &mut Rng) -> String {
    one(rng, &[
        "You climb into the dungeon, full of hope and poor decisions.",
        "Another day, another hole full of things that want you dead.",
        "The dungeon exhales. It smells your optimism. It is hungry.",
        "You descend. Below: loot, and also your death. Mostly the death.",
    ])
}

pub fn kill(rng: &mut Rng, name: &str) -> String {
    // {} gets replaced with the monster's name
    one(rng, &[
        "The {} shuffles off this mortal coil.",
        "You turn the {} into a cautionary tale.",
        "The {} has left the dungeon. Permanently.",
        "The {} rage-quits existence.",
        "You delete the {} from the timeline. No undo.",
        "The {} discovers it is, regrettably, mortal.",
        "The {} is now decor.",
        "The {} learns a brief, final lesson about you.",
    ])
    .replace("{}", name)
}

pub fn level_up(rng: &mut Rng) -> String {
    one(rng, &[
        "You feel dangerously competent.",
        "Something clicks. You are marginally less doomed.",
        "You level up. The dungeon rolls its eyes.",
        "Power surges through you. Or maybe that was the gruel.",
    ])
}

pub fn descend(rng: &mut Rng) -> String {
    one(rng, &[
        "You go deeper. This is, statistically, a mistake.",
        "The stairs creak a warning you elect to ignore.",
        "Down you go: worse air, worse monsters, worse ideas.",
        "A new floor. The same terrible plan.",
    ])
}

pub fn wait(rng: &mut Rng) -> String {
    one(rng, &[
        "You wait. The dungeon does not care.",
        "You take a moment to reconsider your life choices.",
        "Time passes. So does your window to run.",
        "You stand very still, like prey.",
    ])
}

pub fn empty_pickup(rng: &mut Rng) -> String {
    one(rng, &[
        "There's nothing here but regret.",
        "You pat the ground hopefully. Zero loot.",
        "Nothing to pick up. You pick up nothing, expertly.",
        "You grab a fistful of dungeon. It is not useful.",
    ])
}

pub fn epitaph(rng: &mut Rng) -> String {
    one(rng, &[
        "Here lies you. Should've run.",
        "Cause of death: confidence.",
        "You died as you lived: poking things that bite.",
        "The dungeon claims another optimist.",
        "You'll be missed by exactly the monster that ate you.",
        "You delved too greedily and too deep.",
    ])
}
