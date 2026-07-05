// monster brains. a monster that can see the player remembers where they are
// and hunts them down with A*. it keeps chasing for a few turns after losing
// sight (so you can't just duck round a corner and instantly be safe), and if
// it gives up it just wanders around randomly.

use crate::entity::Entity;
use crate::geom::{DIRS8, Point};
use crate::map::Map;
use crate::pathfind;
use crate::rng::Rng;

const SIGHT: i32 = 8; // how far a monster can see
const CHASE_TURNS: i32 = 6; // how long it keeps chasing after losing you

// what a monster wants to do this turn. the game loop actually carries it out
pub enum Intent {
    Attack,
    Step(Point),
    Wait,
}

pub fn decide(
    monster: &mut Entity,
    player_pos: Point,
    map: &Map,
    occupied: &dyn Fn(Point) -> bool,
    rng: &mut Rng,
) -> Intent {
    // can it see me? close enough AND nothing solid in the way
    let sees = monster.pos.king_dist(player_pos) <= SIGHT
        && line_of_sight(map, monster.pos, player_pos);

    if let Some(ai) = monster.ai.as_mut() {
        if sees {
            ai.target = Some(player_pos);
            ai.chasing = CHASE_TURNS;
        } else if ai.chasing > 0 {
            ai.chasing -= 1;
            if ai.chasing == 0 {
                ai.target = None; // lost interest
            }
        }
    }

    match monster.ai.and_then(|a| a.target) {
        Some(target) => {
            if monster.pos.king_dist(player_pos) == 1 {
                Intent::Attack // right next to me, so swing
            } else if let Some(step) = pathfind::next_step(map, monster.pos, target, occupied) {
                Intent::Step(step)
            } else {
                Intent::Wait // can't get to me (boxed in by other monsters probably)
            }
        }
        None => {
            // no target - just shuffle to a random open tile once in a while so
            // idle monsters aren't frozen statues
            if rng.one_in(3) {
                let (dx, dy) = *rng.pick(&DIRS8);
                let n = monster.pos.offset(dx, dy);
                if map.walkable(n) && !occupied(n) {
                    return Intent::Step(n);
                }
            }
            Intent::Wait
        }
    }
}

// bresenham line again, this time just to check if anything solid is between
// two points (that's "line of sight")
fn line_of_sight(map: &Map, a: Point, b: Point) -> bool {
    let (mut x, mut y) = (a.x, a.y);
    let dx = (b.x - a.x).abs();
    let dy = -(b.y - a.y).abs();
    let sx = if a.x < b.x { 1 } else { -1 };
    let sy = if a.y < b.y { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x == b.x && y == b.y {
            return true;
        }
        // don't count the starting tile itself
        if !(x == a.x && y == a.y) && map.opaque(Point::new(x, y)) {
            return false;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
