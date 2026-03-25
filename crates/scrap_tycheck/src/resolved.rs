//! Resolved types after type inference.
//!
//! These types have no inference variables and can be stored
//! and transferred between compilation phases.

use scrap_shared::ident::Symbol;
use scrap_shared::types::{FloatTy, IntTy, Mutability, UintTy};

/// Finalized type with no inference variables.
/// Can be stored and transferred between compilation phases.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum ResolvedTy<'db> {
    /// Void type (no value)
    Void,
    /// Boolean type
    Bool,
    /// Signed integer type
    Int(IntTy),
    /// Unsigned integer type
    Uint(UintTy),
    /// Float type
    Float(FloatTy),
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
    /// GC-managed reference type: `&T` or `&mut T`
    Ref(Box<ResolvedTy<'db>>, Mutability),
    /// GC-managed pointer type: `*T`
    Ptr(Box<ResolvedTy<'db>>),
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
            ResolvedTy::App(_, args) => args.iter().any(ResolvedTy::contains_error),
            ResolvedTy::Fn(params, ret) => {
                params.iter().any(ResolvedTy::contains_error) || ret.contains_error()
            }
            ResolvedTy::Tuple(elems) => elems.iter().any(ResolvedTy::contains_error),
            ResolvedTy::Ref(inner, _) => inner.contains_error(),
            ResolvedTy::Ptr(inner) => inner.contains_error(),
            _ => false,
        }
    }
}
