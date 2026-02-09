//! Unary expression lowering (deref, neg, not, address-of)

use scrap_ast::expr::Expr;
use scrap_ir as ir;
use scrap_shared::types::Mutability;
use scrap_shared::NodeId;

use crate::{lowerer::ExprLowerer, BuilderError, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower a dereference expression `*expr` to an operand.
    /// Reads through a GC reference: loads the pointer, then loads the value.
    pub(crate) fn lower_deref(
        &mut self,
        inner: &Expr<'db>,
        expr_id: NodeId,
    ) -> MResult<ir::Operand<'db>> {
        let inner_operand = self.lower_expr(inner)?;

        // The inner operand should be a place (variable holding a reference)
        let inner_place = match inner_operand {
            ir::Operand::Place(place) => place,
            // If it's a constant or function ref, store it in a temp first
            other => {
                let ref_ty = self.lookup_and_convert_type(inner.id);
                let temp = self.allocate_temp(ref_ty);
                self.emit_assign(ir::Place::Local(temp), ir::Rvalue::Use(other));
                ir::Place::Local(temp)
            }
        };

        // Create Place::Deref to read through the reference
        let deref_place = ir::Place::Deref(Box::new(inner_place));

        // Read the dereferenced value into a temp
        let result_ty = self.lookup_and_convert_type(expr_id);
        let result_temp = self.allocate_temp(result_ty);
        self.emit_assign(
            ir::Place::Local(result_temp),
            ir::Rvalue::Use(ir::Operand::Place(deref_place)),
        );
        Ok(ir::Operand::Place(ir::Place::Local(result_temp)))
    }

    /// Lower a negation expression `-expr`.
    pub(crate) fn lower_unary_neg(
        &mut self,
        inner: &Expr<'db>,
        expr_id: NodeId,
    ) -> MResult<ir::Operand<'db>> {
        let inner_operand = self.lower_expr(inner)?;
        let result_ty = self.lookup_and_convert_type(expr_id);
        let result_temp = self.allocate_temp(result_ty);
        self.emit_assign(
            ir::Place::Local(result_temp),
            ir::Rvalue::Intrinsic(ir::IntrinsicOp::Neg, vec![inner_operand]),
        );
        Ok(ir::Operand::Place(ir::Place::Local(result_temp)))
    }

    /// Lower a logical/bitwise NOT expression `!expr`.
    pub(crate) fn lower_unary_not(
        &mut self,
        inner: &Expr<'db>,
        expr_id: NodeId,
    ) -> MResult<ir::Operand<'db>> {
        let inner_operand = self.lower_expr(inner)?;
        let result_ty = self.lookup_and_convert_type(expr_id);
        let result_temp = self.allocate_temp(result_ty);
        self.emit_assign(
            ir::Place::Local(result_temp),
            ir::Rvalue::Intrinsic(ir::IntrinsicOp::Not, vec![inner_operand]),
        );
        Ok(ir::Operand::Place(ir::Place::Local(result_temp)))
    }

    /// Lower an address-of expression `&expr` or `&mut expr`.
    pub(crate) fn lower_addr_of(
        &mut self,
        mutability: Mutability,
        inner: &Expr<'db>,
        expr_id: NodeId,
    ) -> MResult<ir::Operand<'db>> {
        // Check if inner expression is a *T — if so, just copy the pointer value.
        // `&x` where `x: *T` produces a `&T` that is the same pointer at runtime.
        let is_ptr = matches!(
            self.lookup_expr_type(inner.id),
            Some(scrap_tycheck::ResolvedTy::Ptr(_))
        );

        let inner_operand = self.lower_expr(inner)?;

        if is_ptr {
            // *T → &T: just copy the pointer value (no stack address needed)
            let ref_ty = self.lookup_and_convert_type(expr_id);
            let dest = self.allocate_temp(ref_ty);
            self.emit_assign(ir::Place::Local(dest), ir::Rvalue::Use(inner_operand));
            return Ok(ir::Operand::Place(ir::Place::Local(dest)));
        }

        // Normal case: take stack address via Rvalue::Ref
        let inner_place = match inner_operand {
            ir::Operand::Place(place) => place,
            _ => return Err(BuilderError::LowerExpressionError),
        };

        let ref_ty = self.lookup_and_convert_type(expr_id);
        let dest = self.allocate_temp(ref_ty);
        self.emit_assign(
            ir::Place::Local(dest),
            ir::Rvalue::Ref(mutability, inner_place),
        );
        Ok(ir::Operand::Place(ir::Place::Local(dest)))
    }
}
