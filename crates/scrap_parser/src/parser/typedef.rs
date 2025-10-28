use super::{ScrapInput, ScrapParser, ident::Ident, parse_ident};
use crate::{Span, ast::NodeId};
use chumsky::prelude::*;

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

/// Parse a type annotation
pub fn parse_type<'tokens, I>() -> impl ScrapParser<'tokens, I, Type>
where
    I: ScrapInput<'tokens>,
{
    parse_ident().map_with(|ident, e| Type {
        id: e.state().new_node_id(),
        ident,
    })
}
