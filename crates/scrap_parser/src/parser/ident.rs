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
