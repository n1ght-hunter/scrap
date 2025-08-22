use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId};
use super::{Expr, ExprKind, atom::atom_parser, items::items_parser};

pub fn call_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    atom_parser().foldl_with(
        items_parser()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .repeated(),
        |f, args, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Call(Box::new(f), args),
            span: e.span(),
        },
    )
}
