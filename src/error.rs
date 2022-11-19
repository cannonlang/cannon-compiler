use std::io;

use thiserror::Error;

use crate::span::Pos;

#[derive(Error, Debug)]
pub enum CannonError {
    #[error("unexpected EOF at {0}")]
    Eof(Pos),
    #[error("error reading input file: {0}")]
    ReadError(#[from] io::Error),
    #[error("unexpected {0:?} at {1}")]
    UnexpectedChar(char, Pos),
}
