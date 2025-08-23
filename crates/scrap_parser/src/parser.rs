use chumsky::{input::ValueInput, prelude::*};
use item::Item;

use item::item_parser;
use scrap_lexer::Token;

pub use ident::{capital_ident, parse_ident};

/// A trait alias to simplify the common parser signature used throughout the codebase.
/// This encapsulates the complex return type:
/// `Parser<'tokens, I, Output, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone`
pub trait ScrapParser<'tokens, 'src, I, Output>:
    Parser<'tokens, I, Output, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    'src: 'tokens,
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
}

// Blanket implementation for any type that already implements the required bounds
impl<'tokens, 'src, I, Output, P> ScrapParser<'tokens, 'src, I, Output> for P
where
    'src: 'tokens,
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
    P: Parser<'tokens, I, Output, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone,
{
}

/// Shorthand constraint for the common input type used in parsers
pub trait ScrapInput<'tokens, 'src>: ValueInput<'tokens, Token = Token<'src>, Span = Span>
where
    'src: 'tokens,
{
}

// Blanket implementation for any input that meets our requirements
impl<'tokens, 'src, I> ScrapInput<'tokens, 'src> for I
where
    'src: 'tokens,
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
}

pub mod binary;
pub mod block;
pub mod enumdef;
pub mod expr;
pub mod field;
pub mod fndef;
pub mod ident;
pub mod item;
pub mod lit;
pub mod local;
pub mod pat;
pub mod stmt;
pub mod structdef;
pub mod typedef;

use crate::Span;



/// Parse a sc file into ast
pub fn file_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Vec<Item>>
where
    I: ScrapInput<'tokens, 'src>,
{
    item_parser().repeated().collect()
}
