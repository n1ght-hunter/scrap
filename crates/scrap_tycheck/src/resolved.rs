//! Resolved types after type inference.
//!
//! These types have no inference variables and can be stored
//! and transferred between compilation phases.

use scrap_shared::ident::Symbol;

/// Finalized type with no inference variables.
/// Can be stored and transferred between compilation phases.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum ResolvedTy<'db> {
    /// Boolean type
    Bool,
    /// Integer type
    Int,
    /// String type
    Str,
    /// Never type (diverging)
    Never,
    /// User-defined type (struct/enum)
    Adt(Symbol<'db>),
    /// Generic type parameter (e.g., T in fn foo<T>)
    Param(Symbol<'db>),
    /// Applied generic type (e.g., Vec<int>, Option<T>)
    App(Symbol<'db>, Vec<ResolvedTy<'db>>),
    /// Function type
    Fn(Vec<ResolvedTy<'db>>, Box<ResolvedTy<'db>>),
    /// Tuple type
    Tuple(Vec<ResolvedTy<'db>>),
    /// Error type (for unresolved inference variables or type errors)
    Error,
}

impl<'db> ResolvedTy<'db> {
    /// Check if this is an error type.
    pub fn is_error(&self) -> bool {
        matches!(self, ResolvedTy::Error)
    }

    /// Check if this type contains any errors.
    pub fn contains_error(&self) -> bool {
        match self {
            ResolvedTy::Error => true,
            ResolvedTy::App(_, args) => args.iter().any(|a| a.contains_error()),
            ResolvedTy::Fn(params, ret) => {
                params.iter().any(|p| p.contains_error()) || ret.contains_error()
            }
            ResolvedTy::Tuple(elems) => elems.iter().any(|e| e.contains_error()),
            _ => false,
        }
    }
}
