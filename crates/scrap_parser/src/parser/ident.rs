use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{Span, ast::NodeId};

use super::{ScrapInput, ScrapParser};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub id: NodeId,
    pub name: String,
    pub span: Span,
}

pub fn parse_ident<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Ident>
where
    I: ScrapInput<'tokens, 'src>,
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
) -> impl ScrapParser<'tokens, 'src, I, Ident>
where
    I: ScrapInput<'tokens, 'src>,
{
    parse_ident().validate(move |id, _, emitter| {
        if !id.name.chars().next().unwrap().is_uppercase() {
            emitter.emit(Rich::custom(id.span, err_msg));
        }

        id
    })
}
