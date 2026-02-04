//! Type constraints for inference.

use scrap_span::Span;

use crate::types::InferTy;

/// A constraint between types that must be satisfied.
#[derive(Debug, Clone)]
pub enum Constraint<'db> {
    /// Two types must be equal: T1 == T2
    Eq(InferTy<'db>, InferTy<'db>, ConstraintOrigin<'db>),
}

/// Origin of a constraint (for better error messages).
#[derive(Debug, Clone)]
pub struct ConstraintOrigin<'db> {
    pub span: Span<'db>,
    pub kind: ConstraintKind,
}

impl<'db> ConstraintOrigin<'db> {
    pub fn new(span: Span<'db>, kind: ConstraintKind) -> Self {
        Self { span, kind }
    }
}

/// The kind of operation that generated a constraint.
#[derive(Debug, Clone, Copy)]
pub enum ConstraintKind {
    /// Assignment expression: `x = expr`
    Assignment,
    /// Binary operation: `a + b`
    BinaryOp,
    /// Function argument: `foo(arg)`
    FunctionArg,
    /// Function return type: `return expr`
    FunctionReturn,
    /// If condition: `if cond { }`
    IfCondition,
    /// If branches must match: `if { a } else { b }`
    IfBranches,
    /// Let binding with type annotation: `let x: T = expr`
    LetBinding,
    /// Array element types must match
    ArrayElement,
    /// Generic instantiation
    GenericInstantiation,
}
