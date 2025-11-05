pub use logos::Logos;
use scrap_macros::expand_tokens;

use crate::error::LexingError;

pub mod error;
pub mod token_stream;

expand_tokens! {

#[derive(Debug, PartialEq, Clone)]
pub enum KeyWords {
   #[token("enum")]
    Enum,
    #[token("struct")]
    Struct,
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("return")]
    Return,
    #[token("mod")]
    Mod,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literals {
    #[regex(r#""(\\.|[^"\\])*""#)]
    Str,

    #[regex(r"[0-9]+\.[0-9]*")]
    Float,

    #[regex(r"[0-9]+")]
    Int,

    #[token("false")]
    #[token("true")]
    Bool,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperators {
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("%")]
    Rem,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("^")]
    BitXor,
    #[token("&")]
    BitAnd,
    #[token("|")]
    BitOr,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token("==")]
    Eq,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token("!=")]
    Ne,
    #[token(">=")]
    Ge,
    #[token(">")]
    Gt,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignOps {
    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("*=")]
    MulAssign,
    #[token("/=")]
    DivAssign,
    #[token("%=")]
    RemAssign,
    #[token("&&=")]
    AndAssign,
    #[token("||=")]
    OrAssign,
    #[token("^=")]
    BitXorAssign,
    #[token("&=")]
    BitAndAssign,
    #[token("|=")]
    BitOrAssign,
    #[token("<<=")]
    ShlAssign,
    #[token(">>=")]
    ShrAssign,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operators {
    #[token("->")]
    Arrow,
    #[token("=")]
    Assign,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Delimiters {
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Visibility {
    #[token("pub")]
    Pub,
}

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
#[logos(error(LexingError, LexingError::from_lexer))]
pub enum Token {
    // Skip whitespace
    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    #[display("<whitespace>")]
    Whitespace,

    // Skip comments
    #[regex(r"//[^\r\n]*", logos::skip)]
    #[display("<comment>")]
    Comment,
    #[regex(r"///[^\r\n]*", logos::skip)]
    #[display("<doc_comment>")]
    DocComment,

    Dummy,
    Eof,
}

}

impl Token {
    pub const fn dummy() -> Self {
        Token::Dummy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let file = std::fs::read_to_string("../../tests/basic.sc").unwrap();

        let mut lexer = Token::lexer(&file);

        let mut has_err = false;

        let mut tokens = Vec::new();

        while let Some(res_token) = lexer.next() {
            match res_token {
                Ok(token) => {
                    tokens.push(token);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    has_err = true;
                }
            }
        }

        assert!(!has_err);

        insta::assert_debug_snapshot!(tokens);
    }
}
