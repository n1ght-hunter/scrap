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

impl LexingError {
    pub fn from_lexer<'a>(lex: &mut logos::Lexer<'a, Token>) -> Self {
        LexingError::NonAsciiCharacter(lex.slice().chars().next().unwrap())
    }
}
