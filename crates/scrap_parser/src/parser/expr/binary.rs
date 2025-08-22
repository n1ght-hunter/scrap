use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, parser::binary::bin_op_parser};
use super::{Expr, ExprKind, path::lit_or_path_parser};

pub fn binary_expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    lit_or_path_parser()
        .then(bin_op_parser())
        .then(lit_or_path_parser())
        .map_with(|((left, op), right), e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
            span: e.span(),
        })
}
