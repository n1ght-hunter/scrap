use crate::{Span, ast::NodeId};
use chumsky::prelude::*;

use super::{ScrapInput, ScrapParser, ident::Ident, parse_ident};

#[derive(Debug, Clone, Copy)]
pub enum ByRef {
    Yes(Mutability),
    No,
}

#[derive(Debug, Clone, Copy)]
pub enum Mutability {
    Not,
    Mut,
}

#[derive(Debug, Clone, Copy)]
pub struct BindingMode(pub ByRef, pub Mutability);

#[derive(Debug, Clone)]
pub enum PatKind {
    /// A missing pattern, e.g. for an anonymous param in a bare fn like `fn f(u32)`.
    Missing,
    /// A `PatKind::Ident` may either be a new bound variable (`ref mut binding @ OPT_SUBPATTERN`),
    /// or a unit struct/variant pattern, or a const pattern (in the last two cases the third
    /// field must be `None`). Disambiguation cannot be done with parser alone, so it happens
    /// during name resolution.
    Ident(BindingMode, Ident, Option<Box<Pat>>),
}

#[derive(Debug, Clone)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
    pub span: Span,
}

pub fn pat_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Pat>
where
    I: ScrapInput<'tokens, 'src>,
{
    recursive(|_| {
        parse_ident().map_with(|ident, e| Pat {
            id: e.state().new_node_id(),
            kind: PatKind::Ident(BindingMode(ByRef::No, Mutability::Not), ident, None),
            span: e.span(),
        })
    })
}
