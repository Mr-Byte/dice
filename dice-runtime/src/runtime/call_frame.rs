use std::ops::Range;

#[derive(Copy, Clone, Debug)]
pub struct CallFrame {
    start: usize,
    end: usize,
}

impl CallFrame {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn start(self) -> usize {
        self.start
    }

    pub fn range(self) -> Range<usize> {
        self.start..self.end
    }

    pub fn extend(self, count: usize) -> Self {
        Self {
            start: self.start,
            end: self.end + count,
        }
    }

    pub fn prepend(self, count: usize) -> Self {
        Self {
            start: self.start - count,
            end: self.end,
        }
    }
}
