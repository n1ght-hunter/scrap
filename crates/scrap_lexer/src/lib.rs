pub use logos::Logos;

use crate::error::LexingError;

pub mod error;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error(LexingError, LexingError::from_lexer))]
pub enum Token<'a> {
    // ██╗  ██╗███████╗██╗   ██╗    ██╗    ██╗ ██████╗ ██████╗ ██████╗ ███████╗
    // ██║ ██╔╝██╔════╝╚██╗ ██╔╝    ██║    ██║██╔═══██╗██╔══██╗██╔══██╗██╔════╝
    // █████╔╝ █████╗   ╚████╔╝     ██║ █╗ ██║██║   ██║██████╔╝██║  ██║███████╗
    // ██╔═██╗ ██╔══╝    ╚██╔╝      ██║███╗██║██║   ██║██╔══██╗██║  ██║╚════██║
    // ██║  ██╗███████╗   ██║       ╚███╔███╔╝╚██████╔╝██║  ██║██████╔╝███████║
    // ╚═╝  ╚═╝╚══════╝   ╚═╝        ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚══════╝
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

    // ██╗     ██╗████████╗███████╗██████╗  █████╗ ██╗     ███████╗
    // ██║     ██║╚══██╔══╝██╔════╝██╔══██╗██╔══██╗██║     ██╔════╝
    // ██║     ██║   ██║   █████╗  ██████╔╝███████║██║     ███████╗
    // ██║     ██║   ██║   ██╔══╝  ██╔══██╗██╔══██║██║     ╚════██║
    // ███████╗██║   ██║   ███████╗██║  ██║██║  ██║███████╗███████║
    // ╚══════╝╚═╝   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝
    #[regex(r#""(\\.|[^"\\])*""#)]
    Str(&'a str),
    #[regex(r"[0-9]+")]
    Int(&'a str),
    // Floating-point literals (must contain a decimal point)
    #[regex(r"[0-9]+\.[0-9]*")]
    Float(&'a str),
    // Identifiers (captures variable names, type names, etc.)
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),

    // ██████╗ ██████╗ ███████╗██████╗  █████╗ ████████╗ ██████╗ ██████╗ ███████╗
    // ██╔═══██╗██╔══██╗██╔════╝██╔══██╗██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
    // ██║   ██║██████╔╝█████╗  ██████╔╝███████║   ██║   ██║   ██║██████╔╝███████╗
    // ██║   ██║██╔═══╝ ██╔══╝  ██╔══██╗██╔══██║   ██║   ██║   ██║██╔══██╗╚════██║
    // ╚██████╔╝██║     ███████╗██║  ██║██║  ██║   ██║   ╚██████╔╝██║  ██║███████║
    //  ╚═════╝ ╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝
    #[token("->")]
    Arrow,
    #[token("=")]
    Assign,
    #[token("+")]
    Plus,

    // ██████╗ ██╗   ██╗███╗   ██╗ ██████╗████████╗██╗   ██╗ █████╗ ████████╗██╗ ██████╗ ███╗   ██╗
    // ██╔══██╗██║   ██║████╗  ██║██╔════╝╚══██╔══╝██║   ██║██╔══██╗╚══██╔══╝██║██╔═══██╗████╗  ██║
    // ██████╔╝██║   ██║██╔██╗ ██║██║        ██║   ██║   ██║███████║   ██║   ██║██║   ██║██╔██╗ ██║
    // ██╔═══╝ ██║   ██║██║╚██╗██║██║        ██║   ██║   ██║██╔══██║   ██║   ██║██║   ██║██║╚██╗██║
    // ██║     ╚██████╔╝██║ ╚████║╚██████╗   ██║   ╚██████╔╝██║  ██║   ██║   ██║╚██████╔╝██║ ╚████║
    // ╚═╝      ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
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

    // Skip whitespace
    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    Whitespace,
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

            // Literals and Identifiers that carry their own string slice
            Token::Str(s) => write!(f, "{}", s),
            Token::Int(s) => write!(f, "{}", s),
            Token::Float(s) => write!(f, "{}", s),
            Token::Ident(s) => write!(f, "{}", s),

            // Operators
            Token::Arrow => write!(f, "->"),
            Token::Assign => write!(f, "="),
            Token::Plus => write!(f, "+"),

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let file = std::fs::read_to_string("../../example_pg/basic.sc").unwrap();

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
