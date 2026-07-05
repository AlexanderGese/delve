// A* pathfinding so monsters can route around walls to reach me instead of
// just bumping into a wall forever. i used the standard binary-heap version.
// the annoying part was the heap: rust's BinaryHeap is a MAX heap but A* wants
// the smallest cost first, so i had to flip the ordering (took me way too long
// to figure out why it was exploring the worst nodes first).

use crate::geom::{DIRS8, Point};
use crate::map::Map;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

struct Node {
    cost: i32, // this is f = g + h, the heap orders by it
    pos: Point,
}

impl PartialEq for Node {
    fn eq(&self, o: &Self) -> bool {
        self.cost == o.cost
    }
}
impl Eq for Node {}
impl Ord for Node {
    fn cmp(&self, o: &Self) -> Ordering {
        // REVERSED on purpose so the min cost comes out of the max-heap first
        o.cost.cmp(&self.cost)
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

// find a path from start (not included) to goal (included). `occupied` marks
// tiles blocked by other monsters. the goal tile itself is always allowed so a
// monster can path onto the player's square (it'll attack instead of stepping).
pub fn astar(
    map: &Map,
    start: Point,
    goal: Point,
    occupied: &dyn Fn(Point) -> bool,
) -> Option<Vec<Point>> {
    if start == goal {
        return Some(Vec::new());
    }
    let mut open = BinaryHeap::new();
    let mut came: HashMap<Point, Point> = HashMap::new();
    let mut g: HashMap<Point, i32> = HashMap::new(); // best cost found to each tile

    open.push(Node { cost: 0, pos: start });
    g.insert(start, 0);

    while let Some(Node { pos, .. }) = open.pop() {
        if pos == goal {
            return Some(reconstruct(&came, goal));
        }
        let cur_g = g[&pos];
        for (dx, dy) in DIRS8 {
            let n = pos.offset(dx, dy);
            if !map.walkable(n) {
                continue;
            }
            if n != goal && occupied(n) {
                continue;
            }
            let ng = cur_g + 1;
            if ng < *g.get(&n).unwrap_or(&i32::MAX) {
                came.insert(n, pos);
                g.insert(n, ng);
                let f = ng + n.king_dist(goal); // king_dist is the heuristic
                open.push(Node { cost: f, pos: n });
            }
        }
    }
    None // no path
}

// walk the came-from map backwards to build the path, then flip it round
fn reconstruct(came: &HashMap<Point, Point>, goal: Point) -> Vec<Point> {
    let mut path = vec![goal];
    let mut cur = goal;
    while let Some(&prev) = came.get(&cur) {
        cur = prev;
        path.push(cur);
    }
    path.pop(); // drop the start tile, we don't need to "step" onto where we are
    path.reverse();
    path
}

// just the first step toward the goal, which is all the ai actually needs
pub fn next_step(
    map: &Map,
    start: Point,
    goal: Point,
    occupied: &dyn Fn(Point) -> bool,
) -> Option<Point> {
    astar(map, start, goal, occupied)?.into_iter().next()
}
