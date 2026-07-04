// field of view - working out what the player can actually see.
// how i do it: shoot a straight line (bresenham) from the player out to every
// square on the edge of a box around them, and light up everything each line
// crosses until it hits a wall. it's not the fancy "shadowcasting" one everyone
// uses, but i could actually understand this one and it looks fine.

use crate::geom::Point;
use crate::map::Map;

pub fn compute(map: &mut Map, origin: Point, radius: i32) {
    map.clear_visible();
    map.mark_visible(origin); // you can always see your own tile
    let r2 = radius * radius;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            // only shoot rays at the edge of the box - the rays going out there
            // already pass over everything inside
            if dx.abs() != radius && dy.abs() != radius {
                continue;
            }
            cast_ray(map, origin, Point::new(origin.x + dx, origin.y + dy), r2);
        }
    }
}

// bresenham line from `from` to `to`, stopping at the first wall
fn cast_ray(map: &mut Map, from: Point, to: Point, r2: i32) {
    let (mut x, mut y) = (from.x, from.y);
    let dx = (to.x - from.x).abs();
    let dy = -(to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        let p = Point::new(x, y);
        if from.dist2(p) > r2 {
            break; // past the sight radius
        }
        map.mark_visible(p);
        // stop when we hit something solid, but we still lit that tile so you
        // can see the wall in front of you
        if p != from && map.opaque(p) {
            break;
        }
        if x == to.x && y == to.y {
            break;
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
