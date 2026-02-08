//! Path/variable reference lowering

use scrap_ir as ir;
use scrap_shared::path::Path;

use crate::{lowerer::ExprLowerer, BuilderError, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower a path (variable reference) to an operand
    pub(crate) fn lower_path(&mut self, path: &Path<'db>) -> MResult<ir::Operand<'db>> {
        // Single-segment path: variable or function reference
        if let Some(ident) = path.single_segment().map(|s| s.ident) {
            // Try to look up the variable in the symbol table
            if let Some(local_id) = self.lookup_binding(ident.name) {
                return Ok(ir::Operand::Place(ir::Place::Local(local_id)));
            }

            // Not found as a local - treat as a function reference
            let func_id = ir::FunctionId::new(self.db, ident.name.text(self.db).to_string());
            return Ok(ir::Operand::FunctionRef(func_id));
        }

        // Multi-segment path: check for enum unit variant (e.g. Option::None)
        if path.segments.len() == 2 {
            let enum_name = path.segments[0].ident.name.text(self.db).to_string();
            let variant_name = path.segments[1].ident.name;

            if let Some(enum_info) = self.enum_info.get(&enum_name) {
                if let Some((_, variant_idx, variant_info)) = enum_info
                    .variants
                    .iter()
                    .find(|(name, _, _)| *name == variant_name)
                {
                    // Only construct aggregate for unit variants here.
                    // Tuple/struct variants are handled by call/struct_init lowering.
                    if matches!(variant_info, crate::lowerer::VariantInfo::Unit) {
                        let type_id = ir::TypeId::new(self.db, enum_name);
                        let rvalue = ir::Rvalue::Aggregate(
                            ir::AggregateKind::EnumVariant(type_id, *variant_idx),
                            vec![],
                        );
                        let result_ty = ir::Ty::Adt(type_id);
                        let temp = self.allocate_temp(result_ty);
                        self.emit_assign(ir::Place::Local(temp), rvalue);
                        return Ok(ir::Operand::Place(ir::Place::Local(temp)));
                    }
                }
            }
        }

        Err(BuilderError::LowerExpressionError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use scrap_shared::ident::Symbol;
    use scrap_shared::types::IntTy;

    #[scrap_macros::salsa_test]
    fn test_lower_variable_reference(db: &dyn scrap_shared::Db) {
        let expr = create_ident_expr(db, "x");
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // First, create a binding for "x"
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // Now lower the variable reference
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        let operand = result.unwrap();
        assert!(matches!(operand, ir::Operand::Place(ir::Place::Local(_))));

        // Should only have the one local we created
        assert_eq!(lowerer.local_decls.len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_unknown_path_as_function_ref(db: &dyn scrap_shared::Db) {
        let expr = create_ident_expr(db, "some_function");
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Unknown paths are treated as function references (for function calls)
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        let operand = result.unwrap();
        assert!(matches!(operand, ir::Operand::FunctionRef(_)));
    }
}
