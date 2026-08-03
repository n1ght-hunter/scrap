//! Index expression lowering (`base[index]`), with bounds checking.

use scrap_ast::expr::Expr;
use scrap_ir as ir;
use scrap_shared::NodeId;
use scrap_shared::types::UintTy;

use crate::{MResult, lowerer::ExprLowerer};

impl<'db> ExprLowerer<'db> {
    /// Lower `base[index]` to a bounds-checked `Place::Index`.
    ///
    /// Shared by both the rvalue path (`lower_index`) and the assignment-target path
    /// (`lower_place`), so the check can never be skipped on one side. Emits:
    /// ```text
    ///   _len = array_len(base);
    ///   _ok = index < _len;
    ///   assert(_ok, BoundsCheck) -> ok_bb;
    /// ok_bb:
    /// ```
    /// and returns `base[index]` as a place, ready to read from or write into.
    pub(crate) fn lower_index_place(
        &mut self,
        base: &Expr<'db>,
        index: &Expr<'db>,
    ) -> MResult<ir::Place<'db>> {
        let base_operand = self.lower_expr(base)?;
        let base_place = match base_operand {
            ir::Operand::Place(place) => place,
            other => {
                let base_ty = self.lookup_and_convert_type(base.id);
                let temp = self.allocate_temp(base_ty);
                self.emit_assign(ir::Place::Local(temp), ir::Rvalue::Use(other));
                ir::Place::Local(temp)
            }
        };

        let idx_operand = self.lower_expr(index)?;

        let len_temp = self.allocate_temp(ir::Ty::Uint(UintTy::Usize));
        self.emit_assign(
            ir::Place::Local(len_temp),
            ir::Rvalue::ArrayLen(ir::Operand::Place(base_place.clone())),
        );

        let ok_temp = self.allocate_temp(ir::Ty::Bool);
        self.emit_assign(
            ir::Place::Local(ok_temp),
            ir::Rvalue::Intrinsic(
                ir::IntrinsicOp::Lt,
                vec![
                    idx_operand.clone(),
                    ir::Operand::Place(ir::Place::Local(len_temp)),
                ],
            ),
        );

        let ok_bb = self.cfg_builder.start_block();
        self.cfg_builder.finish_block(ir::Terminator::Assert {
            cond: ir::Operand::Place(ir::Place::Local(ok_temp)),
            expected: true,
            msg: ir::AssertMessage::BoundsCheck,
            target: ok_bb,
            unwind: ir::UnwindAction::Continue,
        });
        self.cfg_builder.set_current_block(ok_bb);

        Ok(ir::Place::Index(
            Box::new(base_place),
            Box::new(idx_operand),
        ))
    }

    /// Lower `base[index]` to an operand (rvalue position): builds the place, then reads it.
    pub(crate) fn lower_index(
        &mut self,
        base: &Expr<'db>,
        index: &Expr<'db>,
        expr_id: NodeId,
    ) -> MResult<ir::Operand<'db>> {
        let index_place = self.lower_index_place(base, index)?;

        let result_ty = self.lookup_and_convert_type(expr_id);
        let result_temp = self.allocate_temp(result_ty);
        self.emit_assign(
            ir::Place::Local(result_temp),
            ir::Rvalue::Use(ir::Operand::Place(index_place)),
        );
        Ok(ir::Operand::Place(ir::Place::Local(result_temp)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use scrap_ir::Ty;
    use scrap_shared::ident::Symbol;

    fn setup_ptr_local<'db>(
        lowerer: &mut ExprLowerer<'db>,
        name: &str,
        pointee: Ty<'db>,
    ) -> Symbol {
        let sym = Symbol::new(name);
        let local = lowerer.allocate_named_local(sym, Ty::Ptr(Box::new(pointee)));
        lowerer.insert_binding(sym, local);
        sym
    }

    #[scrap_macros::salsa_test]
    fn test_lower_index_read_constant(db: &dyn scrap_shared::Db) {
        // arr[0]
        let tt = create_test_type_table();
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, &tt);
        setup_ptr_local(&mut lowerer, "arr", Ty::Uint(UintTy::Usize));

        let base = create_ident_expr(db, "arr");
        let index = create_int_lit(db, 0);
        let expr = create_index_expr(db, base, index);

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        // len temp, ok temp, result temp (base is already a place, index is a constant)
        assert_eq!(lowerer.local_decls.len(), 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_index_read_dynamic(db: &dyn scrap_shared::Db) {
        // arr[i]
        let tt = create_test_type_table();
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, &tt);
        setup_ptr_local(&mut lowerer, "arr", Ty::Uint(UintTy::Usize));
        let i_sym = Symbol::new("i");
        let i_local = lowerer.allocate_named_local(i_sym, Ty::Uint(UintTy::Usize));
        lowerer.insert_binding(i_sym, i_local);

        let base = create_ident_expr(db, "arr");
        let index = create_ident_expr(db, "i");
        let expr = create_index_expr(db, base, index);

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_index_write(db: &dyn scrap_shared::Db) {
        // arr[0] = 5
        let tt = create_test_type_table();
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, &tt);
        setup_ptr_local(&mut lowerer, "arr", Ty::Uint(UintTy::Usize));

        let base = create_ident_expr(db, "arr");
        let index = create_int_lit(db, 0);
        let index_expr = create_index_expr(db, base, index);
        let rhs = create_int_lit(db, 5);
        let assign_expr = create_assign_expr(db, index_expr, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_index_nested(db: &dyn scrap_shared::Db) {
        // arr[i][j] where arr: **usize (pointer to pointer to usize)
        let tt = create_test_type_table();
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, &tt);
        setup_ptr_local(
            &mut lowerer,
            "arr",
            Ty::Ptr(Box::new(Ty::Uint(UintTy::Usize))),
        );
        let i_sym = Symbol::new("i");
        let i_local = lowerer.allocate_named_local(i_sym, Ty::Uint(UintTy::Usize));
        lowerer.insert_binding(i_sym, i_local);
        let j_sym = Symbol::new("j");
        let j_local = lowerer.allocate_named_local(j_sym, Ty::Uint(UintTy::Usize));
        lowerer.insert_binding(j_sym, j_local);

        let base = create_ident_expr(db, "arr");
        let i = create_ident_expr(db, "i");
        let inner = create_index_expr(db, base, i);
        let j = create_ident_expr(db, "j");
        let outer = create_index_expr(db, inner, j);

        let result = lowerer.lower_expr(&outer);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_index_emits_assert(db: &dyn scrap_shared::Db) {
        // arr[0] must branch into a fresh success block via an Assert terminator,
        // not just fall through — confirmed by the block count growing.
        let tt = create_test_type_table();
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, &tt);
        setup_ptr_local(&mut lowerer, "arr", Ty::Uint(UintTy::Usize));

        let before = lowerer.cfg_builder.block_count();

        let base = create_ident_expr(db, "arr");
        let index = create_int_lit(db, 0);
        let expr = create_index_expr(db, base, index);

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        assert!(
            lowerer.cfg_builder.block_count() > before,
            "expected the bounds check to start a new success block"
        );
        assert!(!lowerer.cfg_builder.current_block_is_terminated());
    }
}
