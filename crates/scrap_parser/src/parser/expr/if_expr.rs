//! If expressions
//! 
//! This module handles parsing of if-else expressions with optional else branches.

use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, parser::block::block_parser};
use super::{Expr, ExprKind};

/// Parse if expressions with optional else branches
pub fn if_expr_parser<'tokens, 'src: 'tokens, I, P>(
    condition_parser: P,
) -> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
    P: Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone + 'tokens,
{
    recursive(|if_| {
        just(Token::If)
            .ignore_then(condition_parser)
            .then(block_parser())
            .then(
                just(Token::Else)
                    .ignore_then(block_parser().map_with(|_block, e| Expr {
                        id: NodeId::new(),
                        kind: ExprKind::Err, // Placeholder - blocks need to be converted to expressions
                        span: e.span(),
                    }))
                    .or(if_)
                    .or_not(),
            )
            .map_with(|((cond, then), else_opt), e| Expr {
                id: NodeId::new(),
                kind: ExprKind::If(Box::new(cond), Box::new(then), else_opt.map(Box::new)),
                span: e.span(),
            })
    })
}
