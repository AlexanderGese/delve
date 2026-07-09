# Delve

- A little roguelike I made in Rust, all in the terminal with no graphics, just coloured letters
- You're the `@`. Go down into a dungeon, fight things, grab loot, and eventually die (usually fast)
- Same seed always gives the same dungeon

## How to play

```sh
cargo run --release
```

- WASD / arrows / hjkl — move (walk into a monster to hit it)
- yubn — move diagonally
- e — interact (grab loot or take the stairs)
- g / , — pick up
- i — inventory (drink / read / equip)
- `>` — go down the stairs
- `.` — wait a turn
- m / esc — menu (save + quit)
- q — quit
- `?` — help screen
## Install option 
```sh
cargo install delve-rl
```


## Building

- Just need Rust and any terminal
- Only dependency is `crossterm` for drawing. Everything else is hand-written

```sh
cargo build --release
cargo run --release
```

## What it has

- Random dungeons — non-overlapping rooms joined by L-shaped corridors
- Field of view so you only see what's near you. Visited places stay dimly lit
- Fighting, hp, xp, and levelling up
- Monsters that spot you with line-of-sight and chase with A* around walls, and keep chasing a few turns after losing sight
- A full bestiary — rats and bats up top, orcs and wolves lower, ogres, trolls, demons, and a dragon deep down
- Loot: potions, scrolls, weapons, armour, gold, and an inventory to carry it
- Gets harder the deeper you go
- Suspend-style saving — deleted on load, so no save-scumming death
- Lots of silly one-liners

## Under the hood

- Own xorshift RNG instead of `rand`, so runs are seedable and reproducible
- Player and monsters share one struct — an "ai" is the only thing that makes a monster
- Combat: roll attacker power plus a little random, minus defender defense
- Map is one flat vector indexed as 2D
- Own save format (no serde) — dumps map, monsters, inventory, and RNG state to a text file

