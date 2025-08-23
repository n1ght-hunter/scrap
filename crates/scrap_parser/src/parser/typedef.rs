use super::{ident::Ident, parse_ident, ScrapParser, ScrapInput};
use crate::{Span, ast::NodeId};
use chumsky::prelude::*;
use scrap_lexer::Token;

#[derive(Debug, Clone)]
pub struct Ty {
    pub id: NodeId,
    pub kind: TyKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TyKind {}

#[derive(Debug, Clone)]
pub struct Type {
    pub id: NodeId,
    pub ident: Ident,
}

pub fn parse_type<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Type>
where
    I: ScrapInput<'tokens, 'src>,
{
    parse_ident().map_with(|ident, _| Type { 
        id: NodeId::new(),
        ident 
    })
}
