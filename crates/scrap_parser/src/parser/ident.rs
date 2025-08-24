use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{ast::NodeId, parse_error::ParseError, Span};

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
    .map_with(
        |s,
         e: &mut chumsky::input::MapExtra<
            '_,
            '_,
            I,
            extra::Full<ParseError<'tokens, Token<'src>>, crate::parser::State, ()>,
        >| Ident {
            id: e.state().new_node_id(),
            name: s.to_string(),
            span: e.span(),
        },
    )
}

pub fn capital_ident<'tokens, 'src: 'tokens, I>(
    err_msg: &'static str,
) -> impl ScrapParser<'tokens, 'src, I, Ident>
where
    I: ScrapInput<'tokens, 'src>,
{
    parse_ident().validate(move |id, _, emitter| {
        if !id.name.chars().next().unwrap().is_uppercase() {
            emitter.emit(ParseError::custom(id.span, err_msg));
        }

        id
    })
}
