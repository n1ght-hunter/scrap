//! Block expressions
//! 
//! This module handles parsing of block expressions enclosed in braces.

use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId};
use super::{Expr, ExprKind};
use crate::parser::block::block_parser;

/// Parse block expressions
pub fn block_expr_parser<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    block_parser()
        .map_with(|block, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Block(Box::new(block)),
            span: e.span(),
        })
        .labelled("block expression")
}
