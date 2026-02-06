//! Type lowering from AST to IR

use scrap_ast::typedef::{Ty, TyKind};
use scrap_ir as ir;
use scrap_shared::types::{FloatTy, IntTy, UintTy};

use crate::MResult;

/// Lower a type from AST to IR
pub fn lower_type<'db>(db: &'db dyn scrap_shared::Db, ast_type: &Ty<'db>) -> MResult<ir::Ty<'db>> {
    match &ast_type.kind {
        TyKind::Path(path) => {
            // Get the last segment as the type name
            let type_name = path
                .single_segment()
                .map(|e| e.ident.name.text(db).as_str())
                .unwrap_or("");

            match type_name {
                // Signed integers
                "i8" => Ok(ir::Ty::Int(IntTy::I8)),
                "i16" => Ok(ir::Ty::Int(IntTy::I16)),
                "i32" => Ok(ir::Ty::Int(IntTy::I32)),
                "i64" => Ok(ir::Ty::Int(IntTy::I64)),
                "i128" => Ok(ir::Ty::Int(IntTy::I128)),
                "isize" => Ok(ir::Ty::Int(IntTy::Isize)),
                // Unsigned integers
                "u8" => Ok(ir::Ty::Uint(UintTy::U8)),
                "u16" => Ok(ir::Ty::Uint(UintTy::U16)),
                "u32" => Ok(ir::Ty::Uint(UintTy::U32)),
                "u64" => Ok(ir::Ty::Uint(UintTy::U64)),
                "u128" => Ok(ir::Ty::Uint(UintTy::U128)),
                "usize" => Ok(ir::Ty::Uint(UintTy::Usize)),
                // Floats
                "f16" => Ok(ir::Ty::Float(FloatTy::F16)),
                "f32" => Ok(ir::Ty::Float(FloatTy::F32)),
                "f64" => Ok(ir::Ty::Float(FloatTy::F64)),
                "f128" => Ok(ir::Ty::Float(FloatTy::F128)),
                // Legacy alias
                "int" => Ok(ir::Ty::Int(IntTy::I32)),
                // Other primitives
                "bool" => Ok(ir::Ty::Bool),
                "String" => Ok(ir::Ty::Str),
                _ => {
                    let type_id = ir::TypeId::new(db, type_name);
                    Ok(ir::Ty::Adt(type_id))
                }
            }
        }
        TyKind::Never => Ok(ir::Ty::Never),
        TyKind::Tup(_) => panic!("Tuple types not yet supported in IR lowering"),
        TyKind::Dummy => panic!("Dummy type should not appear in IR lowering"),
        TyKind::Err(_) => panic!("Error type should not appear in IR lowering - type checking should have failed"),
    }
}
