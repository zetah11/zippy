use std::iter::Sum;
use std::ops::{Add, AddAssign, Range};

pub type File = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub file: File,
}

impl Span {
    pub fn new(file: usize, start: usize, end: usize) -> Self {
        Self { start, end, file }
    }
}

impl Add<Span> for Span {
    type Output = Span;

    fn add(self, rhs: Span) -> Self::Output {
        assert_eq!(self.file, rhs.file);
        let start = self.start.min(rhs.start);
        let end = self.end.max(rhs.end);

        Self::new(self.file, start, end)
    }
}

impl AddAssign<Span> for Span {
    fn add_assign(&mut self, rhs: Span) {
        *self = *self + rhs;
    }
}

impl From<Span> for Range<usize> {
    fn from(s: Span) -> Self {
        Self {
            start: s.start,
            end: s.end,
        }
    }
}

impl Sum for Span {
    fn sum<I: Iterator<Item = Self>>(mut iter: I) -> Self {
        let mut res = iter.next().unwrap();
        for span in iter {
            res += span;
        }
        res
    }
}
