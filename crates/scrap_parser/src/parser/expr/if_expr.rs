//! If expressions
//!
//! This module handles parsing of if-else expressions with optional else branches.

use chumsky::prelude::*;
use scrap_lexer::Token;

use super::{Expr, ExprKind};
use crate::parser::block::block_parser;
use crate::parser::{ScrapInput, ScrapParser};

/// Parse if expressions with optional else branches
pub fn if_expr_parser<'tokens, 'src: 'tokens, I, P>(
    condition_parser: P,
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
    P: ScrapParser<'tokens, 'src, I, Expr> + 'tokens,
{
    recursive(|if_| {
        just(Token::If)
            .ignore_then(condition_parser)
            .then(block_parser())
            .then(
                just(Token::Else)
                    .ignore_then(
                        block_parser()
                            .map_with(|block, e| Expr {
                                id: e.state().new_node_id(),
                                kind: ExprKind::Block(Box::new(block)),
                                span: e.span(),
                            })
                            .or(if_),
                    )
                    .or_not(),
            )
            .map_with(|((cond, then_block), else_opt), e| Expr {
                id: e.state().new_node_id(),
                kind: ExprKind::If(Box::new(cond), Box::new(then_block), else_opt.map(Box::new)),
                span: e.span(),
            })
    })
}
