//! Path/variable reference lowering

use scrap_ir as ir;
use scrap_shared::path::Path;

use crate::{lowerer::ExprLowerer, BuilderError, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower a path (variable reference) to an operand
    pub(crate) fn lower_path(&mut self, path: &Path<'db>) -> MResult<ir::Operand<'db>> {
        // Extract the identifier from the path
        let ident = path
            .single_segment()
            .ok_or(BuilderError::LowerExpressionError)?
            .ident;

        // Try to look up the variable in the symbol table
        if let Some(local_id) = self.lookup_binding(ident.name) {
            // Found as a local variable
            return Ok(ir::Operand::Place(ir::Place::Local(local_id)));
        }

        // Not found as a local - treat as a function reference
        let func_id = ir::FunctionId::new(self.db, ident.name.text(self.db).to_string());
        Ok(ir::Operand::FunctionRef(func_id))
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
