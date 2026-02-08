//! Struct expression lowering (construction and field access)

use scrap_ast::expr::{Expr, StructExpr};
use scrap_ir as ir;
use scrap_shared::ident::Ident;

use crate::{lowerer::ExprLowerer, BuilderError, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower a struct initialization expression to an operand.
    pub(crate) fn lower_struct_init(
        &mut self,
        struct_expr: &StructExpr<'db>,
        node_id: scrap_shared::NodeId,
    ) -> MResult<ir::Operand<'db>> {
        let struct_ty = self.lookup_and_convert_type(node_id);
        let dest = ir::Place::Local(self.allocate_temp(struct_ty));
        self.lower_struct_init_into(struct_expr, dest.clone())?;
        Ok(ir::Operand::Place(dest))
    }

    /// Lower a struct init directly into a destination place.
    pub(crate) fn lower_struct_init_into(
        &mut self,
        struct_expr: &StructExpr<'db>,
        dest: ir::Place<'db>,
    ) -> MResult<()> {
        let struct_name = struct_expr
            .path
            .single_segment()
            .ok_or(BuilderError::LowerExpressionError)?
            .ident
            .name;

        let type_id = ir::TypeId::new(self.db, struct_name.text(self.db).to_string());

        // Lower each field expression to an operand
        let mut operands = Vec::new();
        let mut field_names = Vec::new();
        for field_init in struct_expr.fields.iter() {
            let op = self.lower_expr(&field_init.expr)?;
            operands.push(op);
            field_names.push(field_init.ident.name);
        }

        let rvalue = ir::Rvalue::Aggregate(ir::AggregateKind::Struct(type_id, field_names), operands);
        self.emit_assign(dest, rvalue);
        Ok(())
    }

    /// Lower a field access expression to an operand.
    pub(crate) fn lower_field_access(
        &mut self,
        base: &Expr<'db>,
        field_ident: &Ident<'db>,
        _node_id: scrap_shared::NodeId,
    ) -> MResult<ir::Operand<'db>> {
        let base_operand = self.lower_expr(base)?;

        let base_place = match base_operand {
            ir::Operand::Place(place) => place,
            _ => return Err(BuilderError::LowerExpressionError),
        };

        let field_idx = self.resolve_field_index(base.id, field_ident.name)?;
        let field_place = ir::Place::Field(Box::new(base_place), field_idx, Some(field_ident.name));
        Ok(ir::Operand::Place(field_place))
    }

    /// Resolve a field name to its index within the struct's field list.
    fn resolve_field_index(
        &self,
        base_node_id: scrap_shared::NodeId,
        field_name: scrap_shared::ident::Symbol<'db>,
    ) -> MResult<usize> {
        let base_resolved_ty = self
            .lookup_expr_type(base_node_id)
            .ok_or(BuilderError::LowerExpressionError)?;

        if let scrap_tycheck::ResolvedTy::Adt(struct_sym) = base_resolved_ty {
            let struct_name = struct_sym.text(self.db);
            if let Some(field_map) = self.struct_fields.get(struct_name.as_str()) {
                if let Some(&idx) = field_map.get(&field_name) {
                    return Ok(idx);
                }
            }
        }

        Err(BuilderError::LowerExpressionError)
    }
}
