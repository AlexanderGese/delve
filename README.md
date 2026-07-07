# Delve

A little roguelike game I made. It all runs in the terminal - no graphics
at all, just coloured letters. You are the `@`, you go down into a dungeon, you
fight things, you grab loot, and eventually you die. Usually pretty fast.

```
        ########
        #......#      r
   #####+......#######.#
   #.........@........>#
   #####.....#######.###
       #!....#     #...#
       ######      #.g.#
                   #####
```

## How to play

```sh
cargo run --release
```

Controls (there's a help screen on `?` too):

| keys | what it does |
|------|------|
| WASD / arrows / hjkl | move (walk into a monster to hit it) |
| yubn | move diagonally |
| e | interact - grab loot or take the stairs |
| g / , | pick up |
| i | inventory (drink/read/equip) |
| > | go down the stairs |
| . | wait a turn |
| m / esc | menu (save + quit) |
| q | quit |

## Building

You just need rust + any terminal.

```sh
cargo build --release
cargo run --release
```

## Stuff it has so far

- random dungeon generation (rooms + corridors)
- field of view so you can only see what's near you (rest stays dim)
- fighting, hp, xp and levelling up
- monsters that chase you when they see you (they use A* to find you)
- potions / scrolls / weapons / armour + an inventory
- it gets harder the deeper you go
- saving (it writes to a file in your home dir)

## TODO / ideas

- traps?
- more monster types
- maybe a boss on the last floor
- sound? (probably not, it's a terminal game lol)
