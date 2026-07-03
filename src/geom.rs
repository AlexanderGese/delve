// points and rectangles on the grid. pretty basic stuff but i use it everywhere.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    pub fn offset(self, dx: i32, dy: i32) -> Point {
        Point::new(self.x + dx, self.y + dy)
    }

    // chebyshev distance = number of steps if you're allowed to go diagonally
    pub fn king_dist(self, o: Point) -> i32 {
        (self.x - o.x).abs().max((self.y - o.y).abs())
    }

    // squared distance. i keep it squared so i can compare distances without
    // touching floats or sqrt
    pub fn dist2(self, o: Point) -> i32 {
        let dx = self.x - o.x;
        let dy = self.y - o.y;
        dx * dx + dy * dy
    }
}

// the 8 directions you can move in
pub const DIRS8: [(i32, i32); 8] = [
    (-1, -1), (0, -1), (1, -1),
    (-1, 0),          (1, 0),
    (-1, 1),  (0, 1),  (1, 1),
];

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x1: x, y1: y, x2: x + w, y2: y + h }
    }

    pub fn center(&self) -> Point {
        Point::new((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    // do these two rectangles overlap? i use this so rooms don't spawn on top
    // of each other
    pub fn intersects(&self, o: &Rect) -> bool {
        self.x1 <= o.x2 && self.x2 >= o.x1 && self.y1 <= o.y2 && self.y2 >= o.y1
    }
}
