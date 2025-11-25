//! Expression lowering from AST to IR
//!
//! This module handles the conversion of AST expressions to IR operands and rvalues.
//! It manages temporary allocation, symbol table for variable resolution, and
//! statement generation.

use std::collections::HashMap;

use scrap_ast::{
    block::Block,
    expr::{Expr, ExprKind},
    lit::{Lit, LitKind},
    operators::{AssignOp, AssignOpKind, BinOp, BinOpKind},
};
use scrap_ir as ir;
use scrap_shared::{ident::Symbol, path::Path};

use crate::{cfg_builder::BasicBlockBuilder, BuilderError, MResult};

/// Context for lowering expressions to IR
///
/// This struct maintains the state needed during expression lowering:
/// - Local variable declarations (parameters, named locals, temporaries)
/// - Symbol table for resolving variable names to local IDs
/// - CFG builder for managing basic blocks and control flow
pub struct ExprLowerer<'db> {
    db: &'db dyn scrap_shared::Db,
    /// All local variable declarations (params + named locals + temps)
    pub local_decls: Vec<ir::LocalDecl<'db>>,
    /// CFG builder for managing basic blocks
    pub cfg_builder: BasicBlockBuilder<'db>,
    /// Symbol table mapping names to local IDs
    symbol_table: HashMap<Symbol<'db>, ir::LocalId>,
}

impl<'db> ExprLowerer<'db> {
    /// Create a new expression lowerer
    pub fn new(db: &'db dyn scrap_shared::Db) -> Self {
        Self {
            db,
            local_decls: Vec::new(),
            cfg_builder: BasicBlockBuilder::new(db),
            symbol_table: HashMap::new(),
        }
    }

    /// Allocate an anonymous temporary variable
    pub fn allocate_temp(&mut self, ty: ir::Ty<'db>) -> ir::LocalId {
        let id = ir::LocalId(self.local_decls.len());
        let local_decl = ir::LocalDecl::new(self.db, None, ty);
        self.local_decls.push(local_decl);
        id
    }

    /// Allocate a named local variable
    pub fn allocate_named_local(&mut self, name: Symbol<'db>, ty: ir::Ty<'db>) -> ir::LocalId {
        let id = ir::LocalId(self.local_decls.len());
        let local_decl = ir::LocalDecl::new(self.db, Some(name), ty);
        self.local_decls.push(local_decl);
        id
    }

    /// Emit an assignment statement
    pub fn emit_assign(&mut self, place: ir::Place<'db>, rvalue: ir::Rvalue<'db>) {
        let stmt_kind = ir::StatementKind::Assign(place, rvalue);
        let statement = ir::Statement::new(self.db, stmt_kind);
        self.cfg_builder.emit_statement(statement);
    }

    /// Insert a variable binding into the symbol table
    pub fn insert_binding(&mut self, name: Symbol<'db>, local: ir::LocalId) {
        self.symbol_table.insert(name, local);
    }

    /// Look up a variable binding in the symbol table
    pub fn lookup_binding(&self, name: Symbol<'db>) -> Option<ir::LocalId> {
        self.symbol_table.get(&name).copied()
    }

    /// Lower an expression to an operand
    pub fn lower_expr(&mut self, expr: &Expr<'db>) -> MResult<ir::Operand<'db>> {
        match &expr.kind {
            ExprKind::Lit(lit) => self.lower_literal(lit),
            ExprKind::Path(path) => self.lower_path(path),
            ExprKind::Binary(op, lhs, rhs) => self.lower_binary_op(op, lhs, rhs),
            ExprKind::Paren(inner) => self.lower_expr(inner),
            ExprKind::Assign(lhs, rhs, _span) => self.lower_assign(lhs, rhs),
            ExprKind::AssignOp(op, lhs, rhs) => self.lower_assign_op(op, lhs, rhs),
            ExprKind::If(cond, then_block, else_expr) => {
                self.lower_if_expr(cond, then_block, else_expr.as_deref())
            }
            ExprKind::Return(value) => self.lower_return(value.as_deref()),
            ExprKind::Block(block) => self.lower_block_expr(block),
            ExprKind::Array(elements) => self.lower_array(elements),
            ExprKind::Call(func, args) => self.lower_call(func, args),
            ExprKind::Err => Err(BuilderError::LowerExpressionError),
            _ => {
                // Unsupported expression types for now
                Err(BuilderError::LowerExpressionError)
            }
        }
    }

    /// Lower a literal to an operand
    fn lower_literal(&mut self, lit: &Lit<'db>) -> MResult<ir::Operand<'db>> {
        // Create the constant based on literal kind
        let constant = match lit.kind {
            LitKind::Integer => {
                // For now, use a placeholder value since we don't store the actual value
                // TODO: Extract actual value from literal token
                ir::Constant::Int(0)
            }
            LitKind::Bool => {
                // TODO: Extract actual boolean value
                ir::Constant::Bool(false)
            }
            LitKind::Str => {
                // TODO: Extract actual string value
                let sym = Symbol::new(self.db, String::new());
                ir::Constant::String(sym)
            }
            LitKind::Float => {
                // TODO: Extract actual float value
                ir::Constant::Float(0)
            }
        };

        // Infer the type of the literal
        let ty = self.infer_literal_type(lit);

        // Allocate a temporary for the literal
        let temp = self.allocate_temp(ty);

        // Emit assignment: temp = constant
        let place = ir::Place::Local(temp);
        let rvalue = ir::Rvalue::Constant(constant);
        self.emit_assign(place, rvalue);

        // Return reference to the temporary
        Ok(ir::Operand::Place(ir::Place::Local(temp)))
    }

    /// Infer the IR type from a literal
    fn infer_literal_type(&self, lit: &Lit<'_>) -> ir::Ty<'db> {
        match lit.kind {
            LitKind::Integer => ir::Ty::Int,
            LitKind::Bool => ir::Ty::Bool,
            LitKind::Str => ir::Ty::Str,
            LitKind::Float => ir::Ty::Int, // TODO: Add Float type to IR
        }
    }

    /// Lower a path (variable reference) to an operand
    fn lower_path(&mut self, path: &Path<'db>) -> MResult<ir::Operand<'db>> {
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

    /// Lower a binary operation to an operand
    fn lower_binary_op(
        &mut self,
        op: &BinOp<'db>,
        lhs: &Expr<'db>,
        rhs: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        // Recursively lower the left and right operands
        let lhs_operand = self.lower_expr(lhs)?;
        let rhs_operand = self.lower_expr(rhs)?;

        // Convert AST binary operator to IR binary operator
        let ir_op = self.convert_bin_op(op.node)?;

        // Allocate a temporary for the result
        // TODO: Better type inference based on operand types
        let result_ty = ir::Ty::Infer;
        let temp = self.allocate_temp(result_ty);

        // Emit assignment: temp = lhs op rhs
        let place = ir::Place::Local(temp);
        let rvalue = ir::Rvalue::BinaryOp(ir_op, lhs_operand, rhs_operand);
        self.emit_assign(place, rvalue);

        // Return reference to the result temporary
        Ok(ir::Operand::Place(ir::Place::Local(temp)))
    }

    /// Convert AST binary operator to IR binary operator
    fn convert_bin_op(&self, op: BinOpKind) -> MResult<ir::BinOp> {
        match op {
            // Arithmetic operators
            BinOpKind::Add => Ok(ir::BinOp::Add),
            BinOpKind::Sub => Ok(ir::BinOp::Sub),
            BinOpKind::Mul => Ok(ir::BinOp::Mul),
            BinOpKind::Div => Ok(ir::BinOp::Div),
            BinOpKind::Rem => Ok(ir::BinOp::Rem),

            // Logical operators
            BinOpKind::And => Ok(ir::BinOp::And),
            BinOpKind::Or => Ok(ir::BinOp::Or),

            // Bitwise operators
            BinOpKind::BitXor => Ok(ir::BinOp::BitXor),
            BinOpKind::BitAnd => Ok(ir::BinOp::BitAnd),
            BinOpKind::BitOr => Ok(ir::BinOp::BitOr),
            BinOpKind::Shl => Ok(ir::BinOp::Shl),
            BinOpKind::Shr => Ok(ir::BinOp::Shr),

            // Comparison operators
            BinOpKind::Eq => Ok(ir::BinOp::Eq),
            BinOpKind::Lt => Ok(ir::BinOp::Lt),
            BinOpKind::Le => Ok(ir::BinOp::Le),
            BinOpKind::Ne => Ok(ir::BinOp::Ne),
            BinOpKind::Ge => Ok(ir::BinOp::Ge),
            BinOpKind::Gt => Ok(ir::BinOp::Gt),
        }
    }

    /// Convert AST assignment operator to IR binary operator
    fn convert_assign_op(&self, op: AssignOpKind) -> MResult<ir::BinOp> {
        match op {
            AssignOpKind::AddAssign => Ok(ir::BinOp::Add),
            AssignOpKind::SubAssign => Ok(ir::BinOp::Sub),
            AssignOpKind::MulAssign => Ok(ir::BinOp::Mul),
            AssignOpKind::DivAssign => Ok(ir::BinOp::Div),
            AssignOpKind::RemAssign => Ok(ir::BinOp::Rem),
            AssignOpKind::BitXorAssign => Ok(ir::BinOp::BitXor),
            AssignOpKind::BitAndAssign => Ok(ir::BinOp::BitAnd),
            AssignOpKind::BitOrAssign => Ok(ir::BinOp::BitOr),
            AssignOpKind::ShlAssign => Ok(ir::BinOp::Shl),
            AssignOpKind::ShrAssign => Ok(ir::BinOp::Shr),
        }
    }

    /// Lower an expression to a place (for use as LHS of assignment)
    fn lower_place(&mut self, expr: &Expr<'db>) -> MResult<ir::Place<'db>> {
        match &expr.kind {
            ExprKind::Path(path) => {
                // Extract the identifier from the path
                let ident = path
                    .single_segment()
                    .ok_or(BuilderError::LowerExpressionError)?
                    .ident;

                // Look up the variable in the symbol table
                let local_id = self
                    .lookup_binding(ident.name)
                    .ok_or(BuilderError::LowerExpressionError)?;

                Ok(ir::Place::Local(local_id))
            }
            _ => {
                // Only paths can be places for now
                // Future: field access, array indexing, etc.
                Err(BuilderError::LowerExpressionError)
            }
        }
    }

    /// Lower an assignment expression: lhs = rhs
    fn lower_assign(&mut self, lhs: &Expr<'db>, rhs: &Expr<'db>) -> MResult<ir::Operand<'db>> {
        // Lower the LHS to a place
        let place = self.lower_place(lhs)?;

        // Lower the RHS to an operand
        let rhs_operand = self.lower_expr(rhs)?;

        // Emit the assignment: place = Use(rhs_operand)
        let rvalue = ir::Rvalue::Use(rhs_operand);
        self.emit_assign(place, rvalue);

        // Assignments produce unit value (represented as a constant)
        // For now, we'll return a dummy constant
        // TODO: Proper unit type representation
        Ok(ir::Operand::Constant(ir::Constant::Int(0)))
    }

    /// Lower a compound assignment expression: lhs op= rhs
    /// Desugars to: lhs = lhs op rhs
    fn lower_assign_op(
        &mut self,
        op: &AssignOp<'db>,
        lhs: &Expr<'db>,
        rhs: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        // Lower the LHS to a place
        let place = self.lower_place(lhs)?;

        // Lower the RHS to an operand
        let rhs_operand = self.lower_expr(rhs)?;

        // Create an operand for the LHS (to read its current value)
        let lhs_operand = ir::Operand::Place(place.clone());

        // Convert the assignment operator to a binary operator
        let bin_op = self.convert_assign_op(op.node)?;

        // Create the binary operation: lhs op rhs
        let rvalue = ir::Rvalue::BinaryOp(bin_op, lhs_operand, rhs_operand);

        // Emit the assignment: place = lhs op rhs
        self.emit_assign(place, rvalue);

        // Assignments produce unit value
        Ok(ir::Operand::Constant(ir::Constant::Int(0)))
    }

    /// Lower an if-expression with optional else
    fn lower_if_expr(
        &mut self,
        cond: &Expr<'db>,
        then_block: &Block<'db>,
        else_expr: Option<&Expr<'db>>,
    ) -> MResult<ir::Operand<'db>> {
        // Evaluate the condition in the current block
        let cond_operand = self.lower_expr(cond)?;

        // Allocate blocks for the CFG
        let then_bb = self.cfg_builder.start_block();
        let else_bb = self.cfg_builder.start_block();
        let cont_bb = self.cfg_builder.start_block();

        // Finish current block with SwitchInt
        // Note: SwitchInt takes a vector of targets - we use [then_bb, else_bb]
        // where the first is for true and second for false
        let terminator = ir::Terminator::SwitchInt {
            discr: cond_operand,
            targets: vec![then_bb, else_bb],
        };
        self.cfg_builder.finish_block(terminator);

        // Lower the then block
        self.cfg_builder.set_current_block(then_bb);
        self.lower_block(then_block)?;
        if !self.cfg_builder.current_block_is_terminated() {
            self.cfg_builder.finish_block(ir::Terminator::Goto { target: cont_bb });
        }

        // Lower the else expression/block
        self.cfg_builder.set_current_block(else_bb);
        if let Some(else_expr) = else_expr {
            self.lower_expr(else_expr)?;
        }
        if !self.cfg_builder.current_block_is_terminated() {
            self.cfg_builder.finish_block(ir::Terminator::Goto { target: cont_bb });
        }

        // Continue at the continuation block
        self.cfg_builder.set_current_block(cont_bb);

        // If-expressions produce unit value for now
        // TODO: Proper handling of if-expression results
        Ok(ir::Operand::Constant(ir::Constant::Int(0)))
    }

    /// Lower a return statement
    fn lower_return(&mut self, value: Option<&Expr<'db>>) -> MResult<ir::Operand<'db>> {
        // Lower the return value expression (if any) for side effects
        if let Some(expr) = value {
            self.lower_expr(expr)?;
        }

        // Emit the return terminator
        self.cfg_builder.finish_block(ir::Terminator::Return);

        // Returns don't produce a value, but we return a dummy
        Ok(ir::Operand::Constant(ir::Constant::Int(0)))
    }

    /// Lower a block expression
    fn lower_block_expr(&mut self, block: &Block<'db>) -> MResult<ir::Operand<'db>> {
        self.lower_block(block)?;
        // Blocks produce unit value for now
        // TODO: Handle implicit return from last expression
        Ok(ir::Operand::Constant(ir::Constant::Int(0)))
    }

    /// Lower a block's statements
    fn lower_block(&mut self, block: &Block<'db>) -> MResult<()> {
        for stmt in &block.stmts {
            match &stmt.kind {
                scrap_ast::stmt::StmtKind::Semi(expr) | scrap_ast::stmt::StmtKind::Expr(expr) => {
                    // Lower the expression for side effects
                    self.lower_expr(expr)?;
                }
                scrap_ast::stmt::StmtKind::Let(_local) => {
                    // TODO: Handle let statements with initializers
                    // For now, skip
                }
                scrap_ast::stmt::StmtKind::Item(_) | scrap_ast::stmt::StmtKind::Empty => {
                    // Skip these for now
                }
            }
        }
        Ok(())
    }

    /// Lower an array literal to an operand
    fn lower_array(&mut self, elements: &[Box<Expr<'db>>]) -> MResult<ir::Operand<'db>> {
        // Lower each element to an operand
        let mut element_operands = Vec::new();
        for element in elements {
            let operand = self.lower_expr(element)?;
            element_operands.push(operand);
        }

        // Allocate a temporary for the array
        // TODO: Better type inference based on element types
        let result_ty = ir::Ty::Infer;
        let temp = self.allocate_temp(result_ty);

        // Emit assignment: temp = [elem1, elem2, ...]
        let place = ir::Place::Local(temp);
        let rvalue = ir::Rvalue::Array(element_operands);
        self.emit_assign(place, rvalue);

        // Return reference to the array temporary
        Ok(ir::Operand::Place(ir::Place::Local(temp)))
    }

    /// Lower a function call to an operand
    fn lower_call(
        &mut self,
        func: &Expr<'db>,
        args: &[Box<Expr<'db>>],
    ) -> MResult<ir::Operand<'db>> {
        // Lower the function expression (typically a path)
        let func_operand = self.lower_expr(func)?;

        // Lower each argument to an operand
        let mut arg_operands = Vec::new();
        for arg in args {
            let operand = self.lower_expr(arg)?;
            arg_operands.push(operand);
        }

        // Allocate a temporary for the return value
        // TODO: Better type inference based on function signature
        let result_ty = ir::Ty::Infer;
        let result_temp = self.allocate_temp(result_ty);
        let destination = ir::Place::Local(result_temp);

        // Create a continuation block for after the call
        let cont_bb = self.cfg_builder.start_block();

        // Emit the call terminator
        let terminator = ir::Terminator::Call {
            func: func_operand,
            args: arg_operands,
            destination: destination.clone(),
            target: cont_bb,
        };
        self.cfg_builder.finish_block(terminator);

        // Continue at the continuation block
        self.cfg_builder.set_current_block(cont_bb);

        // Return reference to the result temporary
        Ok(ir::Operand::Place(destination))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[scrap_macros::salsa_test]
    fn test_lower_int_literal(db: &dyn scrap_shared::Db) {
        let expr = create_int_lit(db, 42);
        let mut lowerer = ExprLowerer::new(db);

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        let operand = result.unwrap();
        assert!(matches!(operand, ir::Operand::Place(ir::Place::Local(_))));

        // Should have created one local declaration
        assert_eq!(lowerer.local_decls.len(), 1);

        // Check the local declaration type
        assert_eq!(lowerer.local_decls[0].ty(db), ir::Ty::Int);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_bool_literal(db: &dyn scrap_shared::Db) {
        let expr = create_bool_lit(db, true);
        let mut lowerer = ExprLowerer::new(db);

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        assert_eq!(lowerer.local_decls.len(), 1);
        assert_eq!(lowerer.local_decls[0].ty(db), ir::Ty::Bool);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_string_literal(db: &dyn scrap_shared::Db) {
        let expr = create_string_lit(db, "hello");
        let mut lowerer = ExprLowerer::new(db);

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        assert_eq!(lowerer.local_decls.len(), 1);
        assert_eq!(lowerer.local_decls[0].ty(db), ir::Ty::Str);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_variable_reference(db: &dyn scrap_shared::Db) {
        let expr = create_ident_expr(db, "x");
        let mut lowerer = ExprLowerer::new(db);

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
        let mut lowerer = ExprLowerer::new(db);

        // Try to lower without binding the variable
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_err());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_add(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 5);
        let rhs = create_int_lit(db, 3);
        let expr = create_binary_expr(db, BinOpKind::Add, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db);
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        // Should have 3 locals: lhs temp, rhs temp, result temp
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_sub(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 10);
        let rhs = create_int_lit(db, 4);
        let expr = create_binary_expr(db, BinOpKind::Sub, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db);
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_mul(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 6);
        let rhs = create_int_lit(db, 7);
        let expr = create_binary_expr(db, BinOpKind::Mul, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db);
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_comparison(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 5);
        let rhs = create_int_lit(db, 10);
        let expr = create_binary_expr(db, BinOpKind::Lt, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db);
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_logical_and(db: &dyn scrap_shared::Db) {
        let lhs = create_bool_lit(db, true);
        let rhs = create_bool_lit(db, false);
        let expr = create_binary_expr(db, BinOpKind::And, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db);
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_bitwise(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 5);
        let rhs = create_int_lit(db, 3);
        let expr = create_binary_expr(db, BinOpKind::BitAnd, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db);
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_nested_binary(db: &dyn scrap_shared::Db) {
        // (5 + 3) * 2
        let five = create_int_lit(db, 5);
        let three = create_int_lit(db, 3);
        let add_expr = create_binary_expr(db, BinOpKind::Add, five, three);

        let two = create_int_lit(db, 2);
        let mul_expr = create_binary_expr(db, BinOpKind::Mul, add_expr, two);

        let mut lowerer = ExprLowerer::new(db);
        let result = lowerer.lower_expr(&mul_expr);
        assert!(result.is_ok());

        // Should have: 5_temp, 3_temp, add_result, 2_temp, mul_result = 5 locals
        assert_eq!(lowerer.local_decls.len(), 5);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_parenthesized(db: &dyn scrap_shared::Db) {
        let inner = create_int_lit(db, 42);
        let expr = create_paren_expr(db, inner);

        let mut lowerer = ExprLowerer::new(db);
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        // Parentheses don't add anything, just unwrap
        assert_eq!(lowerer.local_decls.len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_operator_conversion(db: &dyn scrap_shared::Db) {
        let lowerer = ExprLowerer::new(db);

        // Test all arithmetic operators
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Add).unwrap(), ir::BinOp::Add);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Sub).unwrap(), ir::BinOp::Sub);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Mul).unwrap(), ir::BinOp::Mul);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Div).unwrap(), ir::BinOp::Div);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Rem).unwrap(), ir::BinOp::Rem);

        // Test comparison operators
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Eq).unwrap(), ir::BinOp::Eq);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Lt).unwrap(), ir::BinOp::Lt);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Gt).unwrap(), ir::BinOp::Gt);
    }

    // ===== Phase 2 Tests: Assignments & Mutations =====

    #[scrap_macros::salsa_test]
    fn test_lower_simple_assignment(db: &dyn scrap_shared::Db) {
        // x = 5
        let mut lowerer = ExprLowerer::new(db);

        // First, create a binding for "x"
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        // Create the assignment: x = 5
        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 5);
        let assign_expr = create_assign_expr(db, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have 2 locals: x, and temp for 5
        assert_eq!(lowerer.local_decls.len(), 2);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_assignment_to_undefined_variable(db: &dyn scrap_shared::Db) {
        // undefined = 5 (should fail)
        let mut lowerer = ExprLowerer::new(db);

        let lhs = create_ident_expr(db, "undefined");
        let rhs = create_int_lit(db, 5);
        let assign_expr = create_assign_expr(db, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_err());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_add(db: &dyn scrap_shared::Db) {
        // x += 5
        let mut lowerer = ExprLowerer::new(db);

        // Create a binding for "x"
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        // Create the assignment: x += 5
        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 5);
        let assign_expr = create_assign_op_expr(db, AssignOpKind::AddAssign, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have 2 locals: x, temp for 5
        assert_eq!(lowerer.local_decls.len(), 2);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_sub(db: &dyn scrap_shared::Db) {
        // x -= 3
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 3);
        let assign_expr = create_assign_op_expr(db, AssignOpKind::SubAssign, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_mul(db: &dyn scrap_shared::Db) {
        // x *= 2
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 2);
        let assign_expr = create_assign_op_expr(db, AssignOpKind::MulAssign, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_bitwise(db: &dyn scrap_shared::Db) {
        // x <<= 1
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 1);
        let assign_expr = create_assign_op_expr(db, AssignOpKind::ShlAssign, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_assignment_with_expression(db: &dyn scrap_shared::Db) {
        // x = 5 + 3
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let five = create_int_lit(db, 5);
        let three = create_int_lit(db, 3);
        let add_expr = create_binary_expr(db, BinOpKind::Add, five, three);
        let assign_expr = create_assign_expr(db, lhs, add_expr);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have: x, 5_temp, 3_temp, add_result_temp
        assert_eq!(lowerer.local_decls.len(), 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_chained_assignment_operations(db: &dyn scrap_shared::Db) {
        // x += 5; then x *= 2 (two separate operations)
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        // First: x += 5
        let lhs1 = create_ident_expr(db, "x");
        let rhs1 = create_int_lit(db, 5);
        let assign1 = create_assign_op_expr(db, AssignOpKind::AddAssign, lhs1, rhs1);
        let result1 = lowerer.lower_expr(&assign1);
        assert!(result1.is_ok());

        // Second: x *= 2
        let lhs2 = create_ident_expr(db, "x");
        let rhs2 = create_int_lit(db, 2);
        let assign2 = create_assign_op_expr(db, AssignOpKind::MulAssign, lhs2, rhs2);
        let result2 = lowerer.lower_expr(&assign2);
        assert!(result2.is_ok());

        // Should have accumulated locals from both operations
        // x, 5_temp, 2_temp
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_assign_op_conversion(db: &dyn scrap_shared::Db) {
        let lowerer = ExprLowerer::new(db);

        // Test all assignment operators
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::AddAssign).unwrap(), ir::BinOp::Add);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::SubAssign).unwrap(), ir::BinOp::Sub);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::MulAssign).unwrap(), ir::BinOp::Mul);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::DivAssign).unwrap(), ir::BinOp::Div);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::RemAssign).unwrap(), ir::BinOp::Rem);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::BitXorAssign).unwrap(), ir::BinOp::BitXor);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::BitAndAssign).unwrap(), ir::BinOp::BitAnd);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::BitOrAssign).unwrap(), ir::BinOp::BitOr);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::ShlAssign).unwrap(), ir::BinOp::Shl);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::ShrAssign).unwrap(), ir::BinOp::Shr);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_place_from_path(db: &dyn scrap_shared::Db) {
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let expr = create_ident_expr(db, "x");
        let result = lowerer.lower_place(&expr);
        assert!(result.is_ok());

        let place = result.unwrap();
        assert!(matches!(place, ir::Place::Local(_)));
    }

    #[scrap_macros::salsa_test]
    fn test_lower_place_from_literal_fails(db: &dyn scrap_shared::Db) {
        let mut lowerer = ExprLowerer::new(db);

        // Trying to use a literal as an lvalue should fail
        let expr = create_int_lit(db, 42);
        let result = lowerer.lower_place(&expr);
        assert!(result.is_err());
    }

    // ===== Phase 3 Tests: Control Flow - Conditionals =====

    #[scrap_macros::salsa_test]
    fn test_lower_if_without_else(db: &dyn scrap_shared::Db) {
        // if x > 0 { }
        let mut lowerer = ExprLowerer::new(db);

        // Create variable x
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        // Create condition: x > 0
        let x_expr = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero);

        // Create empty then block
        let then_block = create_empty_block(db);

        // Create if expression
        let if_expr = create_if_expr(db, cond, then_block);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Should have created multiple blocks for the CFG
        assert!(lowerer.cfg_builder.block_count() >= 4); // entry, then, else, cont
    }

    #[scrap_macros::salsa_test]
    fn test_lower_if_with_else(db: &dyn scrap_shared::Db) {
        // if x > 0 { } else { }
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        // Condition: x > 0
        let x_expr = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero);

        // Then and else blocks
        let then_block = create_empty_block(db);
        let else_block = create_empty_block(db);
        let else_expr = Expr {
            id: test_node_id(),
            kind: ExprKind::Block(Box::new(else_block)),
            span: test_span(db),
        };

        let if_expr = create_if_else_expr(db, cond, then_block, else_expr);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Should have created CFG with multiple blocks
        assert!(lowerer.cfg_builder.block_count() >= 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_nested_if(db: &dyn scrap_shared::Db) {
        // if x > 0 { if y > 0 { } }
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let y_sym = Symbol::new(db, "y".to_string());
        let y_local = lowerer.allocate_named_local(y_sym, ir::Ty::Int);
        lowerer.insert_binding(y_sym, y_local);

        // Outer condition: x > 0
        let x_expr = create_ident_expr(db, "x");
        let zero1 = create_int_lit(db, 0);
        let outer_cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero1);

        // Inner condition: y > 0
        let y_expr = create_ident_expr(db, "y");
        let zero2 = create_int_lit(db, 0);
        let inner_cond = create_binary_expr(db, BinOpKind::Gt, y_expr, zero2);

        // Inner if
        let inner_then = create_empty_block(db);
        let inner_if = create_if_expr(db, inner_cond, inner_then);

        // Outer then block contains inner if
        let stmt = create_expr_stmt(db, inner_if);
        let outer_then = create_block(db, vec![stmt]);

        // Outer if
        let outer_if = create_if_expr(db, outer_cond, outer_then);

        let result = lowerer.lower_expr(&outer_if);
        assert!(result.is_ok());

        // Nested ifs create even more blocks
        assert!(lowerer.cfg_builder.block_count() >= 7);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_return_without_value(db: &dyn scrap_shared::Db) {
        // return;
        let mut lowerer = ExprLowerer::new(db);

        let return_expr = create_return_expr(db, None);

        let result = lowerer.lower_expr(&return_expr);
        assert!(result.is_ok());

        // Current block should be terminated
        assert!(lowerer.cfg_builder.current_block_is_terminated());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_return_with_value(db: &dyn scrap_shared::Db) {
        // return 42;
        let mut lowerer = ExprLowerer::new(db);

        let value = create_int_lit(db, 42);
        let return_expr = create_return_expr(db, Some(value));

        let result = lowerer.lower_expr(&return_expr);
        assert!(result.is_ok());

        // Should have created a local for the literal
        assert_eq!(lowerer.local_decls.len(), 1);

        // Current block should be terminated
        assert!(lowerer.cfg_builder.current_block_is_terminated());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_if_with_return_in_then(db: &dyn scrap_shared::Db) {
        // if x > 0 { return 1; }
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        // Condition
        let x_expr = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero);

        // Then block with return
        let one = create_int_lit(db, 1);
        let return_expr = create_return_expr(db, Some(one));
        let stmt = create_expr_stmt(db, return_expr);
        let then_block = create_block(db, vec![stmt]);

        let if_expr = create_if_expr(db, cond, then_block);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // CFG should have multiple blocks
        assert!(lowerer.cfg_builder.block_count() >= 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_early_return(db: &dyn scrap_shared::Db) {
        // return 42; (followed by more code would be dead)
        let mut lowerer = ExprLowerer::new(db);

        let value = create_int_lit(db, 42);
        let return_expr = create_return_expr(db, Some(value));

        lowerer.lower_expr(&return_expr).unwrap();

        // Try to add another expression (this should be dead code)
        let dead_expr = create_int_lit(db, 99);
        lowerer.lower_expr(&dead_expr).unwrap();

        // Should handle dead code gracefully
        assert!(lowerer.cfg_builder.block_count() >= 1);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_block_expr(db: &dyn scrap_shared::Db) {
        // { x = 5; }
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 5);
        let assign = create_assign_expr(db, lhs, rhs);
        let stmt = create_semi_stmt(db, assign);
        let block = create_block(db, vec![stmt]);

        let block_expr = Expr {
            id: test_node_id(),
            kind: ExprKind::Block(Box::new(block)),
            span: test_span(db),
        };

        let result = lowerer.lower_expr(&block_expr);
        assert!(result.is_ok());

        // Should have created locals for x and the literal
        assert_eq!(lowerer.local_decls.len(), 2);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_if_with_complex_condition(db: &dyn scrap_shared::Db) {
        // if x > 0 && y < 10 { }
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let y_sym = Symbol::new(db, "y".to_string());
        let y_local = lowerer.allocate_named_local(y_sym, ir::Ty::Int);
        lowerer.insert_binding(y_sym, y_local);

        // x > 0
        let x_expr = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let left_cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero);

        // y < 10
        let y_expr = create_ident_expr(db, "y");
        let ten = create_int_lit(db, 10);
        let right_cond = create_binary_expr(db, BinOpKind::Lt, y_expr, ten);

        // x > 0 && y < 10
        let cond = create_binary_expr(db, BinOpKind::And, left_cond, right_cond);

        let then_block = create_empty_block(db);
        let if_expr = create_if_expr(db, cond, then_block);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Should have created multiple temporaries for the complex condition
        assert!(lowerer.local_decls.len() >= 5); // x, y, 0, 10, and temps for comparisons and &&
    }

    #[scrap_macros::salsa_test]
    fn test_cfg_builder_integration(db: &dyn scrap_shared::Db) {
        // Test that we can build a complete CFG
        let mut lowerer = ExprLowerer::new(db);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        // if x > 0 { return 1; } else { return 0; }
        let x_expr1 = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let cond = create_binary_expr(db, BinOpKind::Gt, x_expr1, zero);

        let one = create_int_lit(db, 1);
        let return_one = create_return_expr(db, Some(one));
        let then_stmt = create_expr_stmt(db, return_one);
        let then_block = create_block(db, vec![then_stmt]);

        let zero2 = create_int_lit(db, 0);
        let return_zero = create_return_expr(db, Some(zero2));
        let else_block_expr = Expr {
            id: test_node_id(),
            kind: ExprKind::Block(Box::new(create_block(db, vec![create_expr_stmt(db, return_zero)]))),
            span: test_span(db),
        };

        let if_expr = create_if_else_expr(db, cond, then_block, else_block_expr);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Build the CFG
        let blocks = lowerer.cfg_builder.build();

        // Should have created blocks
        assert!(blocks.len() >= 4);

        // Check that each block has a terminator
        for block in &blocks {
            // Every block should have a terminator
            let term = block.terminator(db);
            assert!(!matches!(term, ir::Terminator::Unreachable) || block.statements(db).is_empty());
        }
    }

    // ===== Phase 5 Tests: Arrays and Function Calls =====

    #[scrap_macros::salsa_test]
    fn test_lower_empty_array(db: &dyn scrap_shared::Db) {
        // []
        let mut lowerer = ExprLowerer::new(db);

        let array_expr = create_array_expr(db, vec![]);

        let result = lowerer.lower_expr(&array_expr);
        assert!(result.is_ok());

        // Should have created one temporary for the array
        assert_eq!(lowerer.local_decls.len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_array_with_literals(db: &dyn scrap_shared::Db) {
        // [1, 2, 3]
        let mut lowerer = ExprLowerer::new(db);

        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let three = create_int_lit(db, 3);
        let array_expr = create_array_expr(db, vec![one, two, three]);

        let result = lowerer.lower_expr(&array_expr);
        assert!(result.is_ok());

        // Should have: 1_temp, 2_temp, 3_temp, array_temp = 4 locals
        assert_eq!(lowerer.local_decls.len(), 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_array_with_variables(db: &dyn scrap_shared::Db) {
        // [x, y]
        let mut lowerer = ExprLowerer::new(db);

        // Create bindings for x and y
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let y_sym = Symbol::new(db, "y".to_string());
        let y_local = lowerer.allocate_named_local(y_sym, ir::Ty::Int);
        lowerer.insert_binding(y_sym, y_local);

        // Create the array
        let x_expr = create_ident_expr(db, "x");
        let y_expr = create_ident_expr(db, "y");
        let array_expr = create_array_expr(db, vec![x_expr, y_expr]);

        let result = lowerer.lower_expr(&array_expr);
        assert!(result.is_ok());

        // Should have: x, y, array_temp = 3 locals
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_array_with_expressions(db: &dyn scrap_shared::Db) {
        // [1 + 2, 3 * 4]
        let mut lowerer = ExprLowerer::new(db);

        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let add_expr = create_binary_expr(db, BinOpKind::Add, one, two);

        let three = create_int_lit(db, 3);
        let four = create_int_lit(db, 4);
        let mul_expr = create_binary_expr(db, BinOpKind::Mul, three, four);

        let array_expr = create_array_expr(db, vec![add_expr, mul_expr]);

        let result = lowerer.lower_expr(&array_expr);
        assert!(result.is_ok());

        // Should have: 1, 2, add_result, 3, 4, mul_result, array = 7 locals
        assert_eq!(lowerer.local_decls.len(), 7);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_nested_array(db: &dyn scrap_shared::Db) {
        // [[1, 2], [3, 4]]
        let mut lowerer = ExprLowerer::new(db);

        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let inner1 = create_array_expr(db, vec![one, two]);

        let three = create_int_lit(db, 3);
        let four = create_int_lit(db, 4);
        let inner2 = create_array_expr(db, vec![three, four]);

        let outer = create_array_expr(db, vec![inner1, inner2]);

        let result = lowerer.lower_expr(&outer);
        assert!(result.is_ok());

        // Should have: 1, 2, inner1_array, 3, 4, inner2_array, outer_array = 7 locals
        assert_eq!(lowerer.local_decls.len(), 7);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_array_assignment(db: &dyn scrap_shared::Db) {
        // arr = [1, 2, 3]
        let mut lowerer = ExprLowerer::new(db);

        // Create binding for arr
        let arr_sym = Symbol::new(db, "arr".to_string());
        let arr_local = lowerer.allocate_named_local(arr_sym, ir::Ty::Infer);
        lowerer.insert_binding(arr_sym, arr_local);

        // Create the array
        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let three = create_int_lit(db, 3);
        let array_expr = create_array_expr(db, vec![one, two, three]);

        // Create the assignment
        let arr_expr = create_ident_expr(db, "arr");
        let assign_expr = create_assign_expr(db, arr_expr, array_expr);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have: arr, 1, 2, 3, array_temp = 5 locals
        assert_eq!(lowerer.local_decls.len(), 5);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_simple_call(db: &dyn scrap_shared::Db) {
        // foo()
        let mut lowerer = ExprLowerer::new(db);

        // Create binding for foo
        let foo_sym = Symbol::new(db, "foo".to_string());
        let foo_local = lowerer.allocate_named_local(foo_sym, ir::Ty::Infer);
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
        let mut lowerer = ExprLowerer::new(db);

        // Create binding for add
        let add_sym = Symbol::new(db, "add".to_string());
        let add_local = lowerer.allocate_named_local(add_sym, ir::Ty::Infer);
        lowerer.insert_binding(add_sym, add_local);

        let func = create_ident_expr(db, "add");
        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let call_expr = create_call_expr(db, func, vec![one, two]);

        let result = lowerer.lower_expr(&call_expr);
        assert!(result.is_ok());

        // Should have: add, 1_temp, 2_temp, result_temp = 4 locals
        assert_eq!(lowerer.local_decls.len(), 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_with_expression_args(db: &dyn scrap_shared::Db) {
        // max(x + 1, y * 2)
        let mut lowerer = ExprLowerer::new(db);

        // Create bindings
        let max_sym = Symbol::new(db, "max".to_string());
        let max_local = lowerer.allocate_named_local(max_sym, ir::Ty::Infer);
        lowerer.insert_binding(max_sym, max_local);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
        lowerer.insert_binding(x_sym, x_local);

        let y_sym = Symbol::new(db, "y".to_string());
        let y_local = lowerer.allocate_named_local(y_sym, ir::Ty::Int);
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

        // Should have: max, x, y, 1, add_result, 2, mul_result, call_result = 8 locals
        assert_eq!(lowerer.local_decls.len(), 8);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_nested_calls(db: &dyn scrap_shared::Db) {
        // outer(inner(1))
        let mut lowerer = ExprLowerer::new(db);

        // Create bindings
        let outer_sym = Symbol::new(db, "outer".to_string());
        let outer_local = lowerer.allocate_named_local(outer_sym, ir::Ty::Infer);
        lowerer.insert_binding(outer_sym, outer_local);

        let inner_sym = Symbol::new(db, "inner".to_string());
        let inner_local = lowerer.allocate_named_local(inner_sym, ir::Ty::Infer);
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

        // Should have: outer, inner, 1, inner_result, outer_result = 5 locals
        assert_eq!(lowerer.local_decls.len(), 5);

        // Should have multiple blocks for nested calls
        assert!(lowerer.cfg_builder.block_count() >= 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_result_assignment(db: &dyn scrap_shared::Db) {
        // result = foo(1, 2)
        let mut lowerer = ExprLowerer::new(db);

        // Create bindings
        let result_sym = Symbol::new(db, "result".to_string());
        let result_local = lowerer.allocate_named_local(result_sym, ir::Ty::Infer);
        lowerer.insert_binding(result_sym, result_local);

        let foo_sym = Symbol::new(db, "foo".to_string());
        let foo_local = lowerer.allocate_named_local(foo_sym, ir::Ty::Infer);
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

        // Should have: result, foo, 1, 2, call_result = 5 locals
        assert_eq!(lowerer.local_decls.len(), 5);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_in_if_condition(db: &dyn scrap_shared::Db) {
        // if is_valid(x) { }
        let mut lowerer = ExprLowerer::new(db);

        // Create bindings
        let is_valid_sym = Symbol::new(db, "is_valid".to_string());
        let is_valid_local = lowerer.allocate_named_local(is_valid_sym, ir::Ty::Infer);
        lowerer.insert_binding(is_valid_sym, is_valid_local);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int);
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
