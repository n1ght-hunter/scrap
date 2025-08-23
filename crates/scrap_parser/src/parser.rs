use chumsky::{input::ValueInput, prelude::*};
use item::Item;

use item::item_parser;
use scrap_lexer::Token;

pub use ident::{capital_ident, parse_ident};

struct Ctx {}

#[derive(Debug, Clone)]
pub struct State {
    id: u32,
    file_hash: u64,
}

impl State {
    pub fn new(file_name: &str) -> Self {
        let file_hash = wyhash::wyhash(file_name.as_bytes(), 0);
        Self { id: 0, file_hash }
    }

    pub fn new_node_id(&mut self) -> NodeId {
        let id = self.id;
        self.id += 1;
        NodeId::new(id, self.file_hash)
    }
}

impl<'src, I: Input<'src>> chumsky::inspector::Inspector<'src, I> for State {
    type Checkpoint = ();

    #[inline(always)]

    fn on_token(&mut self, _: &<I as Input<'src>>::Token) {}

    #[inline(always)]

    fn on_save<'parse>(&self, _: &chumsky::input::Cursor<'src, 'parse, I>) -> Self::Checkpoint {}

    #[inline(always)]

    fn on_rewind<'parse>(
        &mut self,
        _: &chumsky::input::Checkpoint<'src, 'parse, I, Self::Checkpoint>,
    ) {
    }
}

type Extra<'tokens, 'src> = extra::Full<Rich<'tokens, Token<'src>, Span>, State, ()>;

/// A trait alias to simplify the common parser signature used throughout the codebase.
/// This encapsulates the complex return type:
/// `Parser<'tokens, I, Output, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone`
pub trait ScrapParser<'tokens, 'src, I, Output>:
    Parser<'tokens, I, Output, Extra<'tokens, 'src>> + Clone
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
    P: Parser<'tokens, I, Output, Extra<'tokens, 'src>> + Clone,
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
use crate::ast::NodeId;

/// Parse a sc file into ast
pub fn file_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Vec<Item>>
where
    I: ScrapInput<'tokens, 'src>,
{
    item_parser().repeated().collect()
}
