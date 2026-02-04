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

        // Look up the variable in the symbol table
        let local_id = self
            .lookup_binding(ident.name)
            .ok_or(BuilderError::LowerExpressionError)?;

        // Return a reference to the local variable
        Ok(ir::Operand::Place(ir::Place::Local(local_id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use scrap_shared::ident::Symbol;

    #[scrap_macros::salsa_test]
    fn test_lower_variable_reference(db: &dyn scrap_shared::Db) {
        let expr = create_ident_expr(db, "x");
        let mut lowerer = ExprLowerer::new(db, "");

        // First, create a binding for "x"
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
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
    fn test_lower_undefined_variable(db: &dyn scrap_shared::Db) {
        let expr = create_ident_expr(db, "undefined");
        let mut lowerer = ExprLowerer::new(db, "");

        // Try to lower without binding the variable
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_err());
    }
}
