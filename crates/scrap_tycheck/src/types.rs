//! Type representation for type checking and inference.

use scrap_shared::ident::Symbol;
use scrap_shared::types::{FloatTy, IntTy, UintTy};

/// Type variable ID for inference.
/// Represents an unknown type that will be solved during unification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TyVid(pub u32);

/// Internal type representation during type checking.
/// This is separate from the AST types and IR types - it's used only
/// during the type checking phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferTy<'db> {
    /// Inference variable (unknown type to be solved)
    Var(TyVid),

    /// Primitive boolean type
    Bool,

    /// Signed integer type
    Int(IntTy),

    /// Unsigned integer type
    Uint(UintTy),

    /// Float type
    Float(FloatTy),

    /// Primitive string type
    Str,

    /// Never type (diverging, e.g., `return`, `panic!`)
    Never,

    /// User-defined type (struct/enum) without generic arguments
    Adt(Symbol<'db>),

    /// Generic type parameter (e.g., `T` in `fn foo<T>`)
    Param(Symbol<'db>),

    /// Applied generic type (e.g., `Vec<int>`, `Option<T>`)
    /// First Symbol is the type name, Vec contains the type arguments
    App(Symbol<'db>, Vec<InferTy<'db>>),

    /// Function type (for first-class functions)
    /// Parameters followed by return type
    Fn(Vec<InferTy<'db>>, Box<InferTy<'db>>),

    /// Tuple type (including unit `()` as empty tuple)
    Tuple(Vec<InferTy<'db>>),

    /// Error type (for error recovery - unifies with anything)
    Error,
}

impl<'db> InferTy<'db> {
    /// Returns true if this is the unit type (empty tuple)
    pub fn is_unit(&self) -> bool {
        matches!(self, InferTy::Tuple(elems) if elems.is_empty())
    }

    /// Returns true if this is an inference variable
    pub fn is_var(&self) -> bool {
        matches!(self, InferTy::Var(_))
    }

    /// Returns true if this is the error type
    pub fn is_error(&self) -> bool {
        matches!(self, InferTy::Error)
    }

    /// Returns true if this is the never type
    pub fn is_never(&self) -> bool {
        matches!(self, InferTy::Never)
    }

    /// Create a unit type (empty tuple)
    pub fn unit() -> Self {
        InferTy::Tuple(vec![])
    }
}

impl std::fmt::Display for TyVid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?{}", self.0)
    }
}
