//! Expression parsing module
//! 
//! This module contains all expression-related parsers organized into separate files:
//! - `atom.rs`: Basic atomic expressions (literals and identifiers)
//! - `binary.rs`: Binary operations (arithmetic, comparison, etc.)
//! - `call.rs`: Function call expressions
//! - `if_expr.rs`: If-else expressions (recursive version)
//! - `inline.rs`: Main expression parsers (non-recursive versions to avoid stack overflow)
//! - `items.rs`: Parser for comma-separated expression lists
//! - `path.rs`: Path expressions and literal-or-path combinations

use crate::{Span, ast::NodeId, utils::LocalVec};
use super::{binary::BinOp, block::Block, lit::Lit};

pub mod atom;
pub mod binary;
pub mod call;
pub mod if_expr;
pub mod inline;
pub mod items;
pub mod path;

// Re-export the main parser functions
pub use atom::atom_parser;
pub use binary::binary_expr_parser;
pub use call::call_parser;
pub use if_expr::if_expr_parser;
pub use inline::{inline_expr_parser, expr_parser};
pub use items::items_parser;
pub use path::{path_expr_parser, lit_or_path_parser};

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Dummy,
    Path(String),
    Call(Box<Expr>, LocalVec<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Lit(Lit),
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
}

// Add a dummy parser for completeness (this was in the original code)
use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

pub fn dummy_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    just(Token::If).not().map_with(|_, e| Expr {
        id: NodeId::new(),
        kind: ExprKind::Dummy,
        span: e.span(),
    })
}
