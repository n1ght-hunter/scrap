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
            emit_codegen_err(db, "ADT types are not yet supported");
            None
        }
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
pub fn build_cl_signature<M: Module>(
    module: &M,
    ir_sig: ir::Signature<'_>,
    db: &dyn scrap_shared::Db,
) -> Option<Signature> {
    let mut sig = module.make_signature();
    sig.call_conv = module.target_config().default_call_conv;

    for param_ty in ir_sig.params(db) {
        let cl_ty = ir_ty_to_cl_required(db, param_ty)?;
        sig.params.push(AbiParam::new(cl_ty));
    }

    let ret_ty = ir_sig.return_ty(db);
    if let Some(cl_ret) = ir_ty_to_cl(db, &ret_ty)? {
        sig.returns.push(AbiParam::new(cl_ret));
    }

    Some(sig)
}
