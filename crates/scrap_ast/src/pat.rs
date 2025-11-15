use scrap_span::Span;

use crate::{ident::Ident, node_id::NodeId};

pub use scrap_shared::Mutability;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum ByRef {
    Yes(Mutability),
    No,
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct BindingMode(pub ByRef, pub Mutability);

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum PatKind<'db> {
    /// A missing pattern, e.g. for an anonymous param in a bare fn like `fn f(u32)`.
    Missing,
    /// A `PatKind::Ident` may either be a new bound variable (`ref mut binding @ OPT_SUBPATTERN`),
    /// or a unit struct/variant pattern, or a const pattern (in the last two cases the third
    /// field must be `None`). Disambiguation cannot be done with parser alone, so it happens
    /// during name resolution.
    Ident(BindingMode, Ident<'db>, Option<Box<Pat<'db>>>),
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Pat<'db> {
    pub id: NodeId,
    pub kind: PatKind<'db>,
    pub span: Span<'db>,
}
