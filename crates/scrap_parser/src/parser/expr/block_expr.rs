//! Block expressions
//!
//! This module handles parsing of block expressions enclosed in braces.

use chumsky::prelude::*;

use super::{Expr, ExprKind};
use crate::ast::NodeId;
use crate::parser::block::block_parser;
use crate::parser::{ScrapInput, ScrapParser};

/// Parse block expressions
pub fn block_expr_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    block_parser()
        .map_with(|block, e| Expr {
            id: e.state().new_node_id(),
            kind: ExprKind::Block(Box::new(block)),
            span: e.span(),
        })
        .labelled("block expression")
}
