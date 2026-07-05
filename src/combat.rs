// combat. when someone hits someone, i roll their power with a tiny bit of
// randomness, take away the defender's defense, and whatever is left is the
// damage. simple but it feels ok to play.

use crate::color;
use crate::entity::Entity;
use crate::log::Log;
use crate::rng::Rng;

// one attack. `power` is the attacker's real power (the weapon bonus is already
// added in before this is called). returns true if the defender died.
pub fn attack(
    attacker: &str,
    power: i32,
    defense: i32,
    defender: &mut Entity,
    rng: &mut Rng,
    log: &mut Log,
) -> bool {
    let you_attack = attacker == "you";
    let roll = rng.range(power - 1, power + 1).max(1);
    let dmg = (roll - defense).max(0);
    let hits = if you_attack { "hit" } else { "hits" };

    if dmg == 0 {
        // hit but the armour ate it all. grammar (you/it) is annoying so i just
        // special-case it
        log.push(format!(
            "{} {hits} {} but do{} no damage.",
            cap(&article(attacker)),
            article(&defender.name),
            if you_attack { "" } else { "es" },
        ));
        return false;
    }

    defender.stats.damage(dmg);
    let col = if you_attack { color::TEXT } else { color::HP_WARN };
    log.push_colored(
        format!(
            "{} {hits} {} for {dmg} damage.",
            cap(&article(attacker)),
            article(&defender.name),
        ),
        col,
    );

    if defender.is_dead() {
        // only shout about the *player* dying here. when a monster dies, game.rs
        // prints a funnier line instead
        if defender.name == "you" {
            log.push_colored("You die...".to_string(), color::HP_BAD);
        }
        return true;
    }
    false
}

// "you" stays "you"; anything else gets a "the" stuck on the front
fn article(name: &str) -> String {
    if name == "you" {
        "you".to_string()
    } else {
        format!("the {name}")
    }
}

// capitalise the first letter so a sentence doesn't start lowercase
fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
