use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub id: NodeId,
    pub name: String,
    pub span: Span,
}

pub fn parse_ident<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Ident, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    select! {
        Token::Ident(s) => s,
    }
    .map_with(|s, e| Ident {
        id: NodeId::new(),
        name: s.to_string(),
        span: e.span(),
    })
}

pub fn capital_ident<'tokens, 'src: 'tokens, I>(
    err_msg: &'static str,
) -> impl Parser<'tokens, I, Ident, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parse_ident().validate(move |id, _, emitter| {
        if !id.name.chars().next().unwrap().is_uppercase() {
            emitter.emit(Rich::custom(id.span, err_msg));
        }

        id
    })
}
