//! # scrap_typeck
//!
//! Type checking and inference for the Scrap programming language.
//!
//! This crate provides:
//! - Type inference using a constraint-based approach with unification
//! - Type checking for expressions, statements, and items
//! - Support for generic type parameters (preserved for late monomorphization)
//! - Detailed error messages for type mismatches
//!
//! ## Architecture
//!
//! The type checker works in two passes:
//! 1. **Signature collection**: Collect function signatures before checking bodies
//! 2. **Body checking**: Type check function bodies with full signature information
//!
//! During body checking, the type checker:
//! - Infers types for expressions
//! - Collects type constraints (e.g., `T1 == T2`)
//! - Solves constraints using unification
//! - Reports errors for unsatisfiable constraints

mod check;
mod constraints;
mod context;
mod infer;
mod resolved;
mod table;
mod types;
mod unify;

pub use context::{RustFieldVis, RustTypeVis, TypeContext};
pub use resolved::ResolvedTy;
use scrap_shared::InputFile;
pub use table::TypeTable;
pub use types::{InferTy, TyVid};

use scrap_ast::Can;

/// Type check a parsed AST.
///
/// This is the main entry point for type checking. It:
/// 1. Collects all function signatures
/// 2. Type checks all function bodies
/// 3. Solves type constraints
/// 4. Reports any type errors via the diagnostic context
/// 5. Returns a type table mapping expressions to their resolved types
#[salsa::tracked(returns(ref), persist)]
pub fn check_types<'db>(
    db: &'db dyn scrap_shared::Db,
    can: Can<'db>,
    file: InputFile<'db>,
    rust_types: Vec<RustTypeVis>,
) -> TypeTable {
    let mut ctx = TypeContext::new(
        db,
        file.content(db),
        file.path(db).to_str().unwrap_or("<unknown>"),
    );

    for vis in rust_types {
        ctx.register_rust_type(vis);
    }

    ctx.check_can(can);
    ctx.check_pending_alloc_array_elems();

    let (expr_types, local_types, fn_return_types, generic_instantiations) = ctx.finalize_types();

    TypeTable::new(
        expr_types,
        local_types,
        fn_return_types,
        generic_instantiations,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use scrap_shared::types::IntTy;

    #[scrap_macros::salsa_test]
    fn test_type_context_creation(db: &dyn scrap_shared::Db) {
        let _ctx = TypeContext::new(db, "", "test.sc");
        assert!(!db.dcx().has_errors());
    }

    #[scrap_macros::salsa_test]
    fn test_fresh_type_var(db: &dyn scrap_shared::Db) {
        let mut ctx = TypeContext::new(db, "", "test.sc");

        let var1 = ctx.fresh_ty_var();
        let var2 = ctx.fresh_ty_var();

        // Each fresh type var should have a different ID
        match (&var1, &var2) {
            (InferTy::Var(v1), InferTy::Var(v2)) => {
                assert_ne!(v1.0, v2.0);
            }
            _ => panic!("Expected type variables"),
        }
    }

    #[scrap_macros::salsa_test]
    fn test_scope_management(db: &dyn scrap_shared::Db) {
        let mut ctx = TypeContext::new(db, "", "test.sc");

        let sym = scrap_shared::ident::Symbol::new("x");

        // Define in outer scope
        ctx.define_var(sym, InferTy::Int(IntTy::I32));
        assert!(ctx.lookup_var(sym).is_some());

        // Push inner scope
        ctx.push_scope();

        // Can still see outer scope
        assert!(ctx.lookup_var(sym).is_some());

        // Shadow in inner scope
        ctx.define_var(sym, InferTy::Bool);
        assert_eq!(ctx.lookup_var(sym), Some(InferTy::Bool));

        // Pop inner scope
        ctx.pop_scope();

        // Back to outer scope value
        assert_eq!(ctx.lookup_var(sym), Some(InferTy::Int(IntTy::I32)));
    }

    #[scrap_macros::salsa_test]
    fn test_type_recording(db: &dyn scrap_shared::Db) {
        use scrap_shared::NodeId;

        let mut ctx = TypeContext::new(db, "", "test.sc");

        // Create some fake NodeIds
        let expr_id = NodeId::new(1, 0);
        let local_id = NodeId::new(2, 0);

        // Record some types
        ctx.record_expr_type(expr_id, InferTy::Int(IntTy::I32));
        ctx.record_local_type(local_id, InferTy::Bool);

        // Finalize types and create TypeTable
        let (expr_types, local_types, fn_return_types, generic_instantiations) =
            ctx.finalize_types();
        let table = TypeTable::new(
            expr_types,
            local_types,
            fn_return_types,
            generic_instantiations,
        );

        // Verify types are recorded
        assert_eq!(table.expr_type(expr_id), Some(&ResolvedTy::Int(IntTy::I32)));
        assert_eq!(table.local_type(local_id), Some(&ResolvedTy::Bool));
    }

    #[scrap_macros::salsa_test]
    fn test_type_resolution_in_table(db: &dyn scrap_shared::Db) {
        use scrap_shared::NodeId;
        use scrap_span::Span;

        let mut ctx = TypeContext::new(db, "", "test.sc");

        // Create a type variable
        let var = ctx.fresh_ty_var();
        let expr_id = NodeId::new(1, 0);

        // Record it before unification
        ctx.record_expr_type(expr_id, var.clone());

        // Unify the type variable with Int
        let span = Span::new(0, 0);
        ctx.unify(&var, &InferTy::Int(IntTy::I32), span);

        // Finalize - should resolve the type variable to Int
        let (expr_types, local_types, fn_return_types, generic_instantiations) =
            ctx.finalize_types();
        let table = TypeTable::new(
            expr_types,
            local_types,
            fn_return_types,
            generic_instantiations,
        );

        // The type should be resolved to Int, not a type variable
        assert_eq!(table.expr_type(expr_id), Some(&ResolvedTy::Int(IntTy::I32)));
    }
}
