use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::Span;

#[derive(Debug, Clone)]
pub struct Lit {
    pub kind: LitKind,
    pub temp_lit: TempLit,
    // pub symbol: Symbol,
    // pub suffix: Option<Symbol>,
}

#[derive(Debug, Clone)]
pub enum TempLit {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LitKind {
    Bool,
    Integer,
    Float,
    Str,
}

pub fn lit_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Lit, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    select! {
        Token::Bool(value) => Lit {
            kind: LitKind::Bool,
            temp_lit: TempLit::Bool(value),
        },
        Token::Int(value) => Lit {
            kind: LitKind::Integer,
            temp_lit: TempLit::Int(value),
        },
        Token::Float(value) => Lit {
            kind: LitKind::Float,
            temp_lit: TempLit::Float(value),
        },
        Token::Str(value) => Lit {
            kind: LitKind::Str,
            temp_lit: TempLit::Str(value.to_string()),
        },
    }
}
