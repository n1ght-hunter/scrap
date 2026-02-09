//! Method call lowering
//!
//! Desugars `receiver.method(args)` into a regular function call
//! with the mangled name `TypeName::method_name` and receiver as the first argument.

use scrap_ast::expr::Expr;
use scrap_ir as ir;
use scrap_shared::NodeId;
use thin_vec::ThinVec;

use crate::{lowerer::ExprLowerer, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower a method call to an operand (allocates a temporary).
    pub(crate) fn lower_method_call(
        &mut self,
        receiver: &Expr<'db>,
        method: &scrap_shared::ident::Ident<'db>,
        args: &ThinVec<Box<Expr<'db>>>,
        node_id: NodeId,
    ) -> MResult<ir::Operand<'db>> {
        let (func_operand, all_args) = self.lower_method_call_parts(receiver, method, args)?;

        let result_ty = self.lookup_and_convert_type(node_id);
        let never = matches!(result_ty, ir::Ty::Never);
        let result_temp = self.allocate_temp(result_ty);
        let destination = ir::Place::Local(result_temp);

        self.emit_call(func_operand, all_args, destination.clone(), never);
        Ok(ir::Operand::Place(destination))
    }

    /// Lower the common parts of a method call: construct mangled FunctionRef and lower args.
    fn lower_method_call_parts(
        &mut self,
        receiver: &Expr<'db>,
        method: &scrap_shared::ident::Ident<'db>,
        args: &ThinVec<Box<Expr<'db>>>,
    ) -> MResult<(ir::Operand<'db>, Vec<ir::Operand<'db>>)> {
        // Get receiver's type to construct the mangled name
        let type_name = self.lookup_method_type_name(receiver)?;
        let mangled = format!("{}::{}", type_name, method.name.text(self.db));
        let func_id = ir::FunctionId::new(self.db, mangled);
        let func_operand = ir::Operand::FunctionRef(func_id);

        // Lower receiver as the first argument
        let recv = self.lower_expr(receiver)?;
        let mut all_args = vec![recv];
        for arg in args {
            all_args.push(self.lower_expr(arg)?);
        }

        Ok((func_operand, all_args))
    }

    /// Look up the type name of a receiver expression for method call mangling.
    fn lookup_method_type_name(
        &self,
        receiver: &Expr<'db>,
    ) -> MResult<String> {
        if let Some(resolved) = self.lookup_expr_type(receiver.id) {
            match resolved {
                scrap_tycheck::ResolvedTy::Adt(sym) => {
                    Ok(sym.text(self.db).to_string())
                }
                _ => Err(crate::BuilderError::LowerExpressionError),
            }
        } else {
            Err(crate::BuilderError::LowerExpressionError)
        }
    }
}
