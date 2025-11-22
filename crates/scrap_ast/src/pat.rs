use scrap_shared::pretty_print::PrettyPrint;
use scrap_span::Span;

use crate::node_id::NodeId;
use scrap_shared::ident::Ident;

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

impl<'db> PrettyPrint for Pat<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        match &self.kind {
            PatKind::Missing => write!(f, "_"),
            PatKind::Ident(binding_mode, ident, subpat) => {
                match binding_mode {
                    BindingMode(ByRef::Yes(mutability), _) => {
                        write!(f, "ref ")?;
                        if *mutability == Mutability::Mut {
                            write!(f, "mut ")?;
                        }
                    }
                    BindingMode(ByRef::No, mutability) => {
                        if *mutability == Mutability::Mut {
                            write!(f, "mut ")?;
                        }
                    }
                }
                ident.pretty_print(f)?;
                if let Some(subpat) = subpat {
                    write!(f, " @ ")?;
                    subpat.pretty_print(f)?;
                }
                Ok(())
            }
        }
    }
}
