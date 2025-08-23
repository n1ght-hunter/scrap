//! Block expressions
//! 
//! This module handles parsing of block expressions enclosed in braces.

use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::ast::NodeId;
use super::{Expr, ExprKind};
use crate::parser::block::block_parser;
use crate::parser::{ScrapParser, ScrapInput};

/// Parse block expressions
pub fn block_expr_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    block_parser()
        .map_with(|block, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Block(Box::new(block)),
            span: e.span(),
        })
        .labelled("block expression")
}
