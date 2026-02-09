//! IR type to Cranelift type mapping.

use cranelift::prelude::*;
use cranelift_module::Module;
use scrap_ir as ir;
use scrap_shared::types::{FloatTy, IntTy, UintTy};

use super::emit_codegen_err;

/// Convert an IR type to its Cranelift representation.
/// Returns `None` for void/never types (which have no runtime representation).
/// Returns `Some(None)` on error (diagnostic emitted).
pub fn ir_ty_to_cl(db: &dyn scrap_shared::Db, ty: &ir::Ty) -> Option<Option<types::Type>> {
    match ty {
        ir::Ty::Void | ir::Ty::Never => Some(None),
        ir::Ty::Bool => Some(Some(types::I8)),
        ir::Ty::Int(int_ty) => match int_ty {
            IntTy::I8 => Some(Some(types::I8)),
            IntTy::I16 => Some(Some(types::I16)),
            IntTy::I32 => Some(Some(types::I32)),
            IntTy::I64 => Some(Some(types::I64)),
            IntTy::Isize => Some(Some(types::I64)),
            IntTy::I128 => {
                emit_codegen_err(db, "i128 type is not supported");
                None
            }
        },
        ir::Ty::Uint(uint_ty) => match uint_ty {
            UintTy::U8 => Some(Some(types::I8)),
            UintTy::U16 => Some(Some(types::I16)),
            UintTy::U32 => Some(Some(types::I32)),
            UintTy::U64 => Some(Some(types::I64)),
            UintTy::Usize => Some(Some(types::I64)),
            UintTy::U128 => {
                emit_codegen_err(db, "u128 type is not supported");
                None
            }
        },
        ir::Ty::Float(float_ty) => match float_ty {
            FloatTy::F32 => Some(Some(types::F32)),
            FloatTy::F64 => Some(Some(types::F64)),
            FloatTy::F16 => {
                emit_codegen_err(db, "f16 type is not supported");
                None
            }
            FloatTy::F128 => {
                emit_codegen_err(db, "f128 type is not supported");
                None
            }
        },
        ir::Ty::Str => Some(Some(types::I64)), // pointer
        ir::Ty::Adt(_) => {
            // Struct locals are decomposed into per-field sub-variables (like tuples).
            Some(None)
        }
        ir::Ty::Ref(_, _) => Some(Some(types::I64)), // reference
        ir::Ty::Ptr(_) => Some(Some(types::I64)), // GC-managed pointer
        ir::Ty::Tuple(_) => {
            // Tuple locals are decomposed into per-field sub-variables.
            // They have no single Cranelift type representation.
            Some(None)
        }
    }
}

/// Convert an IR type to a Cranelift type, requiring it to be non-void.
pub fn ir_ty_to_cl_required(db: &dyn scrap_shared::Db, ty: &ir::Ty) -> Option<types::Type> {
    match ir_ty_to_cl(db, ty)? {
        Some(cl_ty) => Some(cl_ty),
        None => {
            emit_codegen_err(db, "expected a concrete type, found void/never");
            None
        }
    }
}

/// Build a Cranelift function signature from an IR signature.
/// ADT (struct) parameters are expanded into their constituent field types.
pub fn build_cl_signature<M: Module>(
    module: &M,
    ir_sig: ir::Signature<'_>,
    db: &dyn scrap_shared::Db,
) -> Option<Signature> {
    let mut sig = module.make_signature();
    sig.call_conv = module.target_config().default_call_conv;

    for param_ty in ir_sig.params(db) {
        expand_param_types(db, param_ty, &mut sig.params)?;
    }

    let ret_ty = ir_sig.return_ty(db);
    if let Some(cl_ret) = ir_ty_to_cl(db, &ret_ty)? {
        sig.returns.push(AbiParam::new(cl_ret));
    }

    Some(sig)
}

/// Expand a single IR parameter type into Cranelift ABI params.
/// ADT types are recursively expanded into their fields.
fn expand_param_types(
    db: &dyn scrap_shared::Db,
    ty: &ir::Ty,
    params: &mut Vec<AbiParam>,
) -> Option<()> {
    match ty {
        ir::Ty::Adt(type_id) => {
            // Struct params are passed as individual field values.
            // At this point we don't have struct_layouts, so we look up
            // the struct/enum from the type_id. For now, we treat each
            // ADT field as I64 since all supported field types are 64-bit.
            // This is a temporary approach — it works because the codegen
            // decomposes structs into per-field variables that are all I64.
            //
            // We need the struct layout. Since we don't have it here,
            // we'll emit an error. The caller should use
            // build_cl_signature_with_layouts instead for functions with
            // ADT params.
            emit_codegen_err(
                db,
                format!("cannot build signature for ADT param '{}' without layout info", type_id.name(db)),
            );
            None
        }
        ir::Ty::Tuple(fields) => {
            for field_ty in fields {
                expand_param_types(db, field_ty, params)?;
            }
            Some(())
        }
        _ => {
            let cl_ty = ir_ty_to_cl_required(db, ty)?;
            params.push(AbiParam::new(cl_ty));
            Some(())
        }
    }
}

/// Build a Cranelift function signature from an IR signature,
/// with access to struct layouts for expanding ADT parameters.
pub fn build_cl_signature_with_layouts<M: Module>(
    module: &M,
    ir_sig: ir::Signature<'_>,
    db: &dyn scrap_shared::Db,
    struct_layouts: &std::collections::HashMap<String, Vec<ir::Ty<'_>>>,
) -> Option<Signature> {
    let mut sig = module.make_signature();
    sig.call_conv = module.target_config().default_call_conv;

    for param_ty in ir_sig.params(db) {
        expand_param_types_with_layouts(db, param_ty, &mut sig.params, struct_layouts)?;
    }

    let ret_ty = ir_sig.return_ty(db);
    if let Some(cl_ret) = ir_ty_to_cl(db, &ret_ty)? {
        sig.returns.push(AbiParam::new(cl_ret));
    }

    Some(sig)
}

/// Expand a single IR parameter type into Cranelift ABI params,
/// using struct layouts to decompose ADT types.
fn expand_param_types_with_layouts(
    db: &dyn scrap_shared::Db,
    ty: &ir::Ty,
    params: &mut Vec<AbiParam>,
    struct_layouts: &std::collections::HashMap<String, Vec<ir::Ty<'_>>>,
) -> Option<()> {
    match ty {
        ir::Ty::Adt(type_id) => {
            let adt_name = type_id.name(db);
            if let Some(field_tys) = struct_layouts.get(adt_name.as_str()) {
                for field_ty in field_tys {
                    expand_param_types_with_layouts(db, field_ty, params, struct_layouts)?;
                }
                Some(())
            } else {
                emit_codegen_err(
                    db,
                    format!("struct layout for '{}' not found", adt_name),
                );
                None
            }
        }
        ir::Ty::Tuple(fields) => {
            for field_ty in fields {
                expand_param_types_with_layouts(db, field_ty, params, struct_layouts)?;
            }
            Some(())
        }
        _ => {
            let cl_ty = ir_ty_to_cl_required(db, ty)?;
            params.push(AbiParam::new(cl_ty));
            Some(())
        }
    }
}
