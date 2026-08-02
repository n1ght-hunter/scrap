//! Type conversion utilities for converting resolved types to IR types.

use std::collections::HashSet;

use scrap_ir as ir;
use scrap_tycheck::ResolvedTy;

/// Convert a resolved type from type checking to an IR type.
///
/// This function handles the conversion of types that have been resolved
/// during type checking into the type representation used in the IR.
/// Panics if the resolved type is not yet supported in IR.
pub fn resolved_to_ir<'db>(db: &'db dyn scrap_shared::Db, resolved: &ResolvedTy) -> ir::Ty<'db> {
    resolved_to_ir_inner(db, resolved, &HashSet::new())
}

/// Like [`resolved_to_ir`] but routes ADT names in `rust_type_names` to the
/// memory-backed [`ir::Ty::Rust`] (native Rust interop values) instead of
/// [`ir::Ty::Adt`] (Scrap's SSA-decomposed structs).
pub fn resolved_to_ir_with_rust<'db>(
    db: &'db dyn scrap_shared::Db,
    resolved: &ResolvedTy,
    rust_type_names: &HashSet<String>,
) -> ir::Ty<'db> {
    resolved_to_ir_inner(db, resolved, rust_type_names)
}

fn resolved_to_ir_inner<'db>(
    db: &'db dyn scrap_shared::Db,
    resolved: &ResolvedTy,
    rust_type_names: &HashSet<String>,
) -> ir::Ty<'db> {
    match resolved {
        ResolvedTy::Void => ir::Ty::Void,
        ResolvedTy::Bool => ir::Ty::Bool,
        ResolvedTy::Int(k) => ir::Ty::Int(*k),
        ResolvedTy::Uint(k) => ir::Ty::Uint(*k),
        ResolvedTy::Float(k) => ir::Ty::Float(*k),
        ResolvedTy::Str => ir::Ty::Str,
        ResolvedTy::Never => ir::Ty::Never,

        ResolvedTy::Adt(name) => {
            let name_str = name.text().to_string();
            let type_id = ir::TypeId::new(db, name_str.clone());
            if rust_type_names.contains(&name_str) {
                ir::Ty::Rust(type_id)
            } else {
                ir::Ty::Adt(type_id)
            }
        }

        ResolvedTy::Ref(inner, mutability) => ir::Ty::Ref(
            Box::new(resolved_to_ir_inner(db, inner, rust_type_names)),
            *mutability,
        ),

        ResolvedTy::Ptr(inner) => {
            ir::Ty::Ptr(Box::new(resolved_to_ir_inner(db, inner, rust_type_names)))
        }

        // Unsupported types should be caught during type checking
        ResolvedTy::Error => {
            panic!("Cannot lower Error type to IR - type checking should have failed")
        }
        ResolvedTy::Param(_) => panic!("Generic type parameters not yet supported in IR"),
        ResolvedTy::App(name, args) => {
            let resolved_args: Vec<_> = args.to_vec();
            let mangled = crate::lowering::module::mangle_generic_name_from_resolved(
                db,
                *name,
                &resolved_args,
            );
            let type_id = ir::TypeId::new(db, mangled);
            ir::Ty::Adt(type_id)
        }
        ResolvedTy::Fn(_, _) => panic!("Function types not yet supported in IR"),
        ResolvedTy::Tuple(fields) => ir::Ty::Tuple(
            fields
                .iter()
                .map(|f| resolved_to_ir_inner(db, f, rust_type_names))
                .collect(),
        ),
    }
}
