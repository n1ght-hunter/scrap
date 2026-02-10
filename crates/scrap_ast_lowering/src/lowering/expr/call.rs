//! Function call lowering

use scrap_ast::expr::Expr;
use scrap_ir as ir;

use crate::{lowerer::ExprLowerer, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower the common parts of a function call (func operand + args).
    fn lower_call_parts(
        &mut self,
        call_expr: &Expr<'db>,
    ) -> MResult<(ir::Operand<'db>, Vec<ir::Operand<'db>>)> {
        let (func, args) = match &call_expr.kind {
            scrap_ast::expr::ExprKind::Call(f, a) => (f, a),
            _ => return Err(crate::BuilderError::LowerExpressionError),
        };

        let func_operand = self.lower_expr(func)?;
        let mut arg_operands = Vec::new();
        for arg in args {
            arg_operands.push(self.lower_expr(arg)?);
        }
        Ok((func_operand, arg_operands))
    }

    /// Emit a call terminator writing the result to `destination`.
    /// If `never` is true, the callee returns `!` and there is no continuation block.
    pub(crate) fn emit_call(
        &mut self,
        func: ir::Operand<'db>,
        args: Vec<ir::Operand<'db>>,
        destination: ir::Place<'db>,
        never: bool,
    ) {
        if never {
            let terminator = ir::Terminator::Call {
                func,
                args,
                destination,
                target: None,
                unwind: ir::UnwindAction::Unreachable,
            };
            self.cfg_builder.finish_block(terminator);
        } else {
            let cont_bb = self.cfg_builder.start_block();
            let terminator = ir::Terminator::Call {
                func,
                args,
                destination,
                target: Some(cont_bb),
                unwind: ir::UnwindAction::Continue,
            };
            self.cfg_builder.finish_block(terminator);
            self.cfg_builder.set_current_block(cont_bb);
        }
    }

    /// Check if a call expression is a `box(value)` builtin.
    fn is_box_call(&self, call_expr: &Expr<'db>) -> bool {
        if let scrap_ast::expr::ExprKind::Call(callee, _) = &call_expr.kind {
            if let scrap_ast::expr::ExprKind::Path(path) = &callee.kind {
                if let Some(seg) = path.single_segment() {
                    return seg.ident.name.text(self.db) == "box";
                }
            }
        }
        false
    }

    /// Lower a `box(value)` expression to `Rvalue::Box`.
    fn lower_box_call(
        &mut self,
        call_expr: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        let args = match &call_expr.kind {
            scrap_ast::expr::ExprKind::Call(_, a) => a,
            _ => return Err(crate::BuilderError::LowerExpressionError),
        };

        // box(value) takes exactly one argument
        let value_operand = self.lower_expr(&args[0])?;

        // The result type of box(value) is *T — extract the inner type T
        let result_ty = self.lookup_and_convert_type(call_expr.id);
        let inner_ty = match &result_ty {
            ir::Ty::Ptr(inner) => (**inner).clone(),
            ir::Ty::Ref(inner, _) => (**inner).clone(),
            _ => self.lookup_and_convert_type(args[0].id),
        };

        let result_temp = self.allocate_temp(result_ty);
        let destination = ir::Place::Local(result_temp);

        self.emit_assign(destination.clone(), ir::Rvalue::Box(inner_ty, value_operand));
        Ok(ir::Operand::Place(destination))
    }

    /// Check if a call is an enum tuple variant construction (e.g. Option::Some(42))
    fn is_enum_variant_call(&self, call_expr: &Expr<'db>) -> Option<(String, scrap_shared::ident::Symbol<'db>, usize)> {
        if let scrap_ast::expr::ExprKind::Call(callee, _) = &call_expr.kind {
            if let scrap_ast::expr::ExprKind::Path(path) = &callee.kind {
                if path.segments.len() == 2 {
                    let enum_name = path.segments[0].ident.name.text(self.db).to_string();
                    let variant_name = path.segments[1].ident.name;
                    if let Some(enum_info) = self.enum_info.get(&enum_name) {
                        if let Some((_, variant_idx, _)) = enum_info
                            .variants
                            .iter()
                            .find(|(name, _, _)| *name == variant_name)
                        {
                            return Some((enum_name, variant_name, *variant_idx));
                        }
                    }
                }
            }
        }
        None
    }

    /// Lower a function call to an operand (allocates a temporary).
    pub(crate) fn lower_call(
        &mut self,
        call_expr: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        // Intercept box(value) builtin
        if self.is_box_call(call_expr) {
            return self.lower_box_call(call_expr);
        }

        // Intercept enum tuple variant construction (e.g. Option::Some(42))
        if let Some((enum_name, _variant_name, variant_idx)) = self.is_enum_variant_call(call_expr) {
            let args = match &call_expr.kind {
                scrap_ast::expr::ExprKind::Call(_, a) => a,
                _ => return Err(crate::BuilderError::LowerExpressionError),
            };

            let mut arg_operands = Vec::new();
            for arg in args {
                arg_operands.push(self.lower_expr(arg)?);
            }

            let type_id = ir::TypeId::new(self.db, enum_name);
            let rvalue = ir::Rvalue::Aggregate(
                ir::AggregateKind::EnumVariant(type_id, variant_idx),
                arg_operands,
            );
            let result_ty = ir::Ty::Adt(type_id);
            let temp = self.allocate_temp(result_ty);
            self.emit_assign(ir::Place::Local(temp), rvalue);
            return Ok(ir::Operand::Place(ir::Place::Local(temp)));
        }

        let (func_operand, arg_operands) = self.lower_call_parts(call_expr)?;

        let result_ty = self.lookup_and_convert_type(call_expr.id);
        let never = matches!(result_ty, ir::Ty::Never);
        let result_temp = self.allocate_temp(result_ty);
        let destination = ir::Place::Local(result_temp);

        self.emit_call(func_operand, arg_operands, destination.clone(), never);
        Ok(ir::Operand::Place(destination))
    }

    /// Lower a `spawn` expression: dispatches on Call, MethodCall, or Block.
    pub(crate) fn lower_spawn(
        &mut self,
        expr: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        match &expr.kind {
            scrap_ast::expr::ExprKind::Call(callee, args) => {
                self.lower_spawn_call(callee, args)
            }
            scrap_ast::expr::ExprKind::Block(block) => self.lower_spawn_block(block),
            // For other expressions (like MethodCall), fall through to a generic
            // lowering that treats the inner as a call.
            _ => Err(crate::BuilderError::LowerExpressionError),
        }
    }

    /// Lower `spawn f(args)` — emit Rvalue::Spawn with function ref and args.
    fn lower_spawn_call(
        &mut self,
        callee: &Expr<'db>,
        args: &[Box<Expr<'db>>],
    ) -> MResult<ir::Operand<'db>> {
        let fn_operand = self.lower_expr(callee)?;
        let mut arg_operands = Vec::new();
        for arg in args {
            arg_operands.push(self.lower_expr(arg)?);
        }

        let result_temp = self.allocate_temp(ir::Ty::Void);
        let dest = ir::Place::Local(result_temp);
        self.emit_assign(dest.clone(), ir::Rvalue::Spawn(fn_operand, arg_operands));
        Ok(ir::Operand::Place(dest))
    }

    /// Lower `spawn { ... }` — generate an anonymous function and emit a zero-arg spawn.
    fn lower_spawn_block(
        &mut self,
        block: &scrap_ast::block::Block<'db>,
    ) -> MResult<ir::Operand<'db>> {
        let name = format!("__spawn_block_{}", self.spawn_block_counter);
        self.spawn_block_counter += 1;
        let name_sym = scrap_shared::ident::Symbol::new(self.db, name.clone());

        // Create a fresh lowerer for the anonymous function body.
        let mut block_lowerer = ExprLowerer::new(self.db, self.source, self.type_table);
        block_lowerer.struct_fields = self.struct_fields.clone();
        block_lowerer.enum_info = self.enum_info.clone();

        // _0 = void return place
        block_lowerer.allocate_temp(ir::Ty::Void);

        // Lower the block's statements into the anonymous function.
        block_lowerer.lower_block(block)?;

        // Ensure the function body is terminated.
        if !block_lowerer.cfg_builder.current_block_is_terminated() {
            block_lowerer.cfg_builder.finish_block(ir::Terminator::Return);
        }

        // Build the anonymous IR function.
        let blocks = block_lowerer.cfg_builder.build();
        let body = ir::Body::new(self.db, blocks, block_lowerer.local_decls, 0);
        let sig = ir::Signature::new(self.db, name_sym, vec![], ir::Ty::Void);
        let func = ir::Function::new(self.db, sig, body);

        // Collect the anonymous function (and any nested spawn blocks).
        self.extra_functions.push(ir::Items::Function(func));
        self.extra_functions.extend(block_lowerer.extra_functions);

        // Emit a zero-arg spawn of the anonymous function.
        let func_id = ir::FunctionId::new(self.db, name);
        let fn_op = ir::Operand::FunctionRef(func_id);
        let result_temp = self.allocate_temp(ir::Ty::Void);
        let dest = ir::Place::Local(result_temp);
        self.emit_assign(dest.clone(), ir::Rvalue::Spawn(fn_op, vec![]));
        Ok(ir::Operand::Place(dest))
    }

    /// Lower a function call directly into a destination place.
    pub(crate) fn lower_call_into(
        &mut self,
        call_expr: &Expr<'db>,
        dest: ir::Place<'db>,
    ) -> MResult<()> {
        // Intercept box(value) builtin
        if self.is_box_call(call_expr) {
            let operand = self.lower_box_call(call_expr)?;
            self.emit_assign(dest, ir::Rvalue::Use(operand));
            return Ok(());
        }

        // Intercept enum tuple variant construction (e.g. Option::Some(42))
        if let Some((enum_name, _variant_name, variant_idx)) = self.is_enum_variant_call(call_expr) {
            let args = match &call_expr.kind {
                scrap_ast::expr::ExprKind::Call(_, a) => a,
                _ => return Err(crate::BuilderError::LowerExpressionError),
            };

            let mut arg_operands = Vec::new();
            for arg in args {
                arg_operands.push(self.lower_expr(arg)?);
            }

            let type_id = ir::TypeId::new(self.db, enum_name);
            let rvalue = ir::Rvalue::Aggregate(
                ir::AggregateKind::EnumVariant(type_id, variant_idx),
                arg_operands,
            );
            self.emit_assign(dest, rvalue);
            return Ok(());
        }

        let result_ty = self.lookup_and_convert_type(call_expr.id);
        let never = matches!(result_ty, ir::Ty::Never);
        let (func_operand, arg_operands) = self.lower_call_parts(call_expr)?;
        self.emit_call(func_operand, arg_operands, dest, never);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use scrap_ast::operators::BinOpKind;
    use scrap_shared::ident::Symbol;
    use scrap_shared::types::IntTy;

    #[scrap_macros::salsa_test]
    fn test_lower_simple_call(db: &dyn scrap_shared::Db) {
        // foo()
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Create binding for foo
        let foo_sym = Symbol::new(db, "foo".to_string());
        let foo_local = lowerer.allocate_named_local(foo_sym, ir::Ty::Never);
        lowerer.insert_binding(foo_sym, foo_local);

        let func = create_ident_expr(db, "foo");
        let call_expr = create_call_expr(db, func, vec![]);

        let result = lowerer.lower_expr(&call_expr);
        assert!(result.is_ok());

        // Should have: foo, result_temp = 2 locals
        assert_eq!(lowerer.local_decls.len(), 2);

        // Should have created at least 2 blocks (call block + continuation)
        assert!(lowerer.cfg_builder.block_count() >= 2);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_with_args(db: &dyn scrap_shared::Db) {
        // add(1, 2)
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create binding for add
        let add_sym = Symbol::new(db, "add".to_string());
        let add_local = lowerer.allocate_named_local(add_sym, ir::Ty::Never);
        lowerer.insert_binding(add_sym, add_local);

        let func = create_ident_expr(db, "add");
        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let call_expr = create_call_expr(db, func, vec![one, two]);

        let result = lowerer.lower_expr(&call_expr);
        assert!(result.is_ok());

        // Should have: add, result_temp = 2 locals (literals are constants)
        assert_eq!(lowerer.local_decls.len(), 2);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_with_expression_args(db: &dyn scrap_shared::Db) {
        // max(x + 1, y * 2)
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create bindings
        let max_sym = Symbol::new(db, "max".to_string());
        let max_local = lowerer.allocate_named_local(max_sym, ir::Ty::Never);
        lowerer.insert_binding(max_sym, max_local);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let y_sym = Symbol::new(db, "y".to_string());
        let y_local = lowerer.allocate_named_local(y_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(y_sym, y_local);

        // Create the call: max(x + 1, y * 2)
        let func = create_ident_expr(db, "max");
        let x_expr = create_ident_expr(db, "x");
        let one = create_int_lit(db, 1);
        let arg1 = create_binary_expr(db, BinOpKind::Add, x_expr, one);

        let y_expr = create_ident_expr(db, "y");
        let two = create_int_lit(db, 2);
        let arg2 = create_binary_expr(db, BinOpKind::Mul, y_expr, two);

        let call_expr = create_call_expr(db, func, vec![arg1, arg2]);

        let result = lowerer.lower_expr(&call_expr);
        assert!(result.is_ok());

        // Should have: max, x, y, pair1, add_result, pair2, mul_result, call_result = 8 locals
        assert_eq!(lowerer.local_decls.len(), 8);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_nested_calls(db: &dyn scrap_shared::Db) {
        // outer(inner(1))
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create bindings
        let outer_sym = Symbol::new(db, "outer".to_string());
        let outer_local = lowerer.allocate_named_local(outer_sym, ir::Ty::Never);
        lowerer.insert_binding(outer_sym, outer_local);

        let inner_sym = Symbol::new(db, "inner".to_string());
        let inner_local = lowerer.allocate_named_local(inner_sym, ir::Ty::Never);
        lowerer.insert_binding(inner_sym, inner_local);

        // Create inner call: inner(1)
        let inner_func = create_ident_expr(db, "inner");
        let one = create_int_lit(db, 1);
        let inner_call = create_call_expr(db, inner_func, vec![one]);

        // Create outer call: outer(inner(1))
        let outer_func = create_ident_expr(db, "outer");
        let outer_call = create_call_expr(db, outer_func, vec![inner_call]);

        let result = lowerer.lower_expr(&outer_call);
        assert!(result.is_ok());

        // Should have: outer, inner, inner_result, outer_result = 4 locals
        assert_eq!(lowerer.local_decls.len(), 4);

        // Should have multiple blocks for nested calls
        assert!(lowerer.cfg_builder.block_count() >= 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_result_assignment(db: &dyn scrap_shared::Db) {
        // result = foo(1, 2)
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create bindings
        let result_sym = Symbol::new(db, "result".to_string());
        let result_local = lowerer.allocate_named_local(result_sym, ir::Ty::Never);
        lowerer.insert_binding(result_sym, result_local);

        let foo_sym = Symbol::new(db, "foo".to_string());
        let foo_local = lowerer.allocate_named_local(foo_sym, ir::Ty::Never);
        lowerer.insert_binding(foo_sym, foo_local);

        // Create call: foo(1, 2)
        let func = create_ident_expr(db, "foo");
        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let call_expr = create_call_expr(db, func, vec![one, two]);

        // Create assignment: result = foo(1, 2)
        let result_expr = create_ident_expr(db, "result");
        let assign_expr = create_assign_expr(db, result_expr, call_expr);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have: result, foo, call_result = 3 locals
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_in_if_condition(db: &dyn scrap_shared::Db) {
        // if is_valid(x) { }
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Create bindings
        let is_valid_sym = Symbol::new(db, "is_valid".to_string());
        let is_valid_local = lowerer.allocate_named_local(is_valid_sym, ir::Ty::Never);
        lowerer.insert_binding(is_valid_sym, is_valid_local);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // Create call: is_valid(x)
        let func = create_ident_expr(db, "is_valid");
        let x_expr = create_ident_expr(db, "x");
        let call_expr = create_call_expr(db, func, vec![x_expr]);

        // Create if: if is_valid(x) { }
        let then_block = create_empty_block(db);
        let if_expr = create_if_expr(db, call_expr, then_block);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Should have multiple blocks for call + if structure
        assert!(lowerer.cfg_builder.block_count() >= 5);
    }
}
