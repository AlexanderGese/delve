// my own little random number generator so i don't have to pull in the rand
// crate. it's xorshift128+ (found the algorithm online) seeded through
// splitmix64. rolling my own also means the game is repeatable from one seed,
// which was really handy for debugging.

pub struct Rng {
    s: [u64; 2],
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        // mix the seed first with splitmix64 so even a boring seed like 1 gives
        // a decent starting state
        let mut z = seed;
        let mut next = || {
            z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut r = z;
            r = (r ^ (r >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            r = (r ^ (r >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            r ^ (r >> 31)
        };
        // the | 1 makes sure the state is never all zeros - xorshift gets stuck
        // forever on zero (learned that the hard way)
        Rng { s: [next() | 1, next() | 1] }
    }

    pub fn state(&self) -> (u64, u64) {
        (self.s[0], self.s[1])
    }

    pub fn from_state(a: u64, b: u64) -> Rng {
        Rng { s: [a, b] }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.s[0];
        let y = self.s[1];
        self.s[0] = y;
        x ^= x << 23;
        x ^= x >> 17;
        x ^= y ^ (y >> 26);
        self.s[1] = x;
        x.wrapping_add(y)
    }

    // random number from 0 up to n (not including n)
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        // yeah this has a tiny modulo bias but for a game it really doesn't matter
        (self.next_u64() % n as u64) as u32
    }

    // random int between lo and hi, both included
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + self.below((hi - lo + 1) as u32) as i32
    }

    // true num out of den times
    pub fn chance(&mut self, num: u32, den: u32) -> bool {
        self.below(den) < num
    }

    pub fn one_in(&mut self, n: u32) -> bool {
        self.below(n) == 0
    }

    // grab a random element from a slice
    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len() as u32) as usize]
    }
}
