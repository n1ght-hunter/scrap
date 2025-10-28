use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, parse_error::ParseError};

use super::{ScrapInput, ScrapParser};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub id: NodeId,
    pub name: String,
    pub span: Span,
}

pub fn parse_ident<'tokens, I>() -> impl ScrapParser<'tokens, I, Ident>
where
    I: ScrapInput<'tokens>,
{
    select! {
        Token::Ident => "placeholder_ident",
    }
    .map_with(
        |s,
         e: &mut chumsky::input::MapExtra<
            '_,
            '_,
            I,
            extra::Full<ParseError<'tokens, Token>, crate::parser::State, ()>,
        >| Ident {
            id: e.state().new_node_id(),
            name: s.to_string(),
            span: e.span(),
        },
    )
}
