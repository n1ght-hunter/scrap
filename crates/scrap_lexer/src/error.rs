use std::num::ParseIntError;

use crate::Token;

#[derive(Default, Debug, Clone, PartialEq, thiserror::Error)]

pub enum LexingError {
    #[error("invalid integer: {0}")]
    InvalidInteger(String),
    #[error("non-ASCII character: {0}")]
    NonAsciiCharacter(char),
    #[default]
    #[error("other error")]
    Other,
}

/// Error type returned by calling `lex.slice().parse()` to u8.
impl From<ParseIntError> for LexingError {
    fn from(err: ParseIntError) -> Self {
        use std::num::IntErrorKind::*;
        match err.kind() {
            PosOverflow | NegOverflow => LexingError::InvalidInteger("overflow error".to_owned()),
            _ => LexingError::InvalidInteger("other error".to_owned()),
        }
    }
}

impl LexingError {
    pub fn from_lexer<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> Self {
        LexingError::NonAsciiCharacter(lex.slice().chars().next().unwrap())
    }
}
