pub use logos::Logos;
use scrap_macros::expand_tokens;

use crate::error::LexingError;

pub mod error;

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
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literals<'a> {
    #[regex(r#""(\\.|[^"\\])*""#)]
    Str(&'a str),

    #[regex(r"[0-9]+\.[0-9]*", |lex| lex.slice().parse::<f64>().unwrap())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Int(i64),

    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),


    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operators {
    #[token("->")]
    Arrow,
    #[token("=")]
    Assign,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
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

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error(LexingError, LexingError::from_lexer))]
pub enum Token<'a> {
    // Skip whitespace
    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    Whitespace,

    // Skip comments
    #[regex(r"//[^\r\n]*", logos::skip)]
    Comment,
}

}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Keywords
            Token::Enum => write!(f, "enum"),
            Token::Struct => write!(f, "struct"),
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Return => write!(f, "return"),

            // Literals and Identifiers that carry their own string slice
            Token::Str(s) => write!(f, "{s}"),
            Token::Int(s) => write!(f, "{s}"),
            Token::Float(s) => write!(f, "{s}"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Bool(b) => write!(f, "{b}"),

            // Operators
            Token::Arrow => write!(f, "->"),
            Token::Assign => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::BitXor => write!(f, "^"),
            Token::BitAnd => write!(f, "&"),
            Token::BitOr => write!(f, "|"),
            Token::Shl => write!(f, "<<"),
            Token::Shr => write!(f, ">>"),
            Token::Eq => write!(f, "=="),
            Token::Lt => write!(f, "<"),
            Token::Le => write!(f, "<="),
            Token::Ne => write!(f, "!="),
            Token::Ge => write!(f, ">="),
            Token::Gt => write!(f, ">"),

            // Punctuation
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"), // Double braces to escape in format string
            Token::RBrace => write!(f, "}}"), // Double braces to escape in format string
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),

            // Whitespace is skipped, but we must handle it for an exhaustive match
            Token::Whitespace => write!(f, "<whitespace>"),
            Token::Comment => write!(f, "<comment>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let file = std::fs::read_to_string("../../example/basic.sc").unwrap();

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
