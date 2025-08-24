//! If expressions
//!
//! This module handles parsing of if-else expressions with optional else branches.

use chumsky::prelude::*;
use scrap_lexer::Token;

use super::{Expr, ExprKind, inline_expr_parser};
use crate::parse_error::ParseError;
use crate::parser::block::block_parser;
use crate::parser::{ScrapInput, ScrapParser};

/// Parse if expressions with optional else branches
pub fn if_expr_parser<'tokens, 'src: 'tokens, I>(
    else_required: bool,
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    recursive(|if_| {
        just(Token::If)
            .ignore_then(inline_expr_parser())
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
            .validate(move |expr, e, emitter| {
                if expr.1.is_none() && else_required {
                    emitter.emit(ParseError::custom(
                        e.span(),
                        "else branch is required for if expression",
                    ));
                }

                expr
            })
            .map_with(|((cond, then_block), else_opt), e| Expr {
                id: e.state().new_node_id(),
                kind: ExprKind::If(Box::new(cond), Box::new(then_block), else_opt.map(Box::new)),
                span: e.span(),
            })
    })
}
