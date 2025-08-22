use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, utils::LocalVec};
use super::{Expr, inline::expr_parser};

pub fn items_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, LocalVec<Expr>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    expr_parser()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<LocalVec<_>>()
}
