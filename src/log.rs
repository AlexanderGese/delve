// the message log at the bottom of the screen. i keep the last few messages in
// a queue and throw away old ones.

use crossterm::style::Color;
use std::collections::VecDeque;

pub struct Entry {
    pub text: String,
    pub color: Color,
}

pub struct Log {
    lines: VecDeque<Entry>,
    cap: usize, // how many messages to remember
}

impl Log {
    pub fn new(cap: usize) -> Log {
        Log { lines: VecDeque::new(), cap }
    }

    pub fn push(&mut self, text: impl Into<String>) {
        // default colour if i don't care what colour it is
        self.push_colored(text, crate::color::TEXT);
    }

    pub fn push_colored(&mut self, text: impl Into<String>, color: Color) {
        self.lines.push_back(Entry { text: text.into(), color });
        // drop old messages so the queue doesn't grow forever
        while self.lines.len() > self.cap {
            self.lines.pop_front();
        }
    }

    // the last n messages, oldest first
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &Entry> {
        let start = self.lines.len().saturating_sub(n);
        self.lines.iter().skip(start)
    }
}
