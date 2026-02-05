//! Type conversion utilities for converting resolved types to IR types.

use scrap_ir as ir;
use scrap_tycheck::ResolvedTy;

/// Convert a resolved type from type checking to an IR type.
///
/// This function handles the conversion of types that have been resolved
/// during type checking into the type representation used in the IR.
/// Panics if the resolved type is not yet supported in IR.
pub fn resolved_to_ir<'db>(
    db: &'db dyn scrap_shared::Db,
    resolved: &ResolvedTy<'db>,
) -> ir::Ty<'db> {
    match resolved {
        ResolvedTy::Bool => ir::Ty::Bool,
        ResolvedTy::Int => ir::Ty::Int,
        ResolvedTy::Str => ir::Ty::Str,
        ResolvedTy::Never => ir::Ty::Never,

        ResolvedTy::Adt(name) => {
            let type_id = ir::TypeId::new(db, name.text(db).to_string());
            ir::Ty::Adt(type_id)
        }

        // Unsupported types should be caught during type checking
        ResolvedTy::Error => panic!("Cannot lower Error type to IR - type checking should have failed"),
        ResolvedTy::Param(_) => panic!("Generic type parameters not yet supported in IR"),
        ResolvedTy::App(_, _) => panic!("Applied generic types not yet supported in IR"),
        ResolvedTy::Fn(_, _) => panic!("Function types not yet supported in IR"),
        ResolvedTy::Tuple(_) => panic!("Tuple types not yet supported in IR"),
    }
}
