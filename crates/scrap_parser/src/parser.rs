use chumsky::{input::ValueInput, prelude::*};
use item::Item;

use item::item_parser;
use scrap_lexer::Token;

pub use ident::{capital_ident, parse_ident};

pub mod block;
pub mod enumdef;
pub mod expr;
pub mod field;
pub mod fndef;
pub mod ident;
pub mod item;
pub mod local;
pub mod pat;
pub mod stmt;
pub mod structdef;
pub mod typedef;
pub mod binary;
pub mod lit;

use crate::Span;

/// Parse a sc file into ast
pub fn file_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Vec<Item>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    item_parser().repeated().collect()
}
