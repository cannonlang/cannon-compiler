use std::fmt::{self, Display};

#[derive(Clone, Copy, Debug)]
pub struct Pos(pub usize, pub usize); // row, col

impl Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Debug)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl From<Pos> for Span {
    fn from(pos: Pos) -> Self {
        Self {
            start: pos,
            end: Pos(pos.0, pos.1 + 1),
        }
    }
}

impl From<&Pos> for Span {
    fn from(pos: &Pos) -> Self {
        Self {
            start: *pos,
            end: Pos(pos.0, pos.1 + 1),
        }
    }
}

impl From<&mut Pos> for Span {
    fn from(pos: &mut Pos) -> Self {
        Self {
            start: *pos,
            end: Pos(pos.0, pos.1 + 1),
        }
    }
}

impl From<(Pos, Pos)> for Span {
    fn from(span: (Pos, Pos)) -> Self {
        Self {
            start: span.0,
            end: span.1,
        }
    }
}
