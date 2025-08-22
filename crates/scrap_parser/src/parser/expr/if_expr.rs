use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, parser::block::block_parser};
use super::{Expr, ExprKind, inline::expr_parser};

pub fn if_expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    recursive(|if_| {
        just(Token::If)
            .ignore_then(expr_parser())
            .then(block_parser())
            .then(
                just(Token::Else)
                    .ignore_then(expr_parser().or(if_))
                    .or_not(),
            )
            .map_with(|((cond, then), else_opt), e| Expr {
                id: NodeId::new(),
                kind: ExprKind::If(Box::new(cond), Box::new(then), else_opt.map(Box::new)),
                span: e.span(),
            })
    })
}
