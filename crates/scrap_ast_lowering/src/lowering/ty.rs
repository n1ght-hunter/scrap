//! Type lowering from AST to IR

use scrap_ast::typedef::{Ty, TyKind};
use scrap_ir as ir;

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
                "int" => Ok(ir::Ty::Int),
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
