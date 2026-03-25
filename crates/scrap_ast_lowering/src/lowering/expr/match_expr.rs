//! Match expression lowering

use scrap_ast::expr::{Arm, Expr};
use scrap_ast::pat::PatKind;
use scrap_ir as ir;
use scrap_shared::ident::Symbol;

use crate::{BuilderError, MResult, lowerer::ExprLowerer};

impl<'db> ExprLowerer<'db> {
    /// Lower a match expression.
    ///
    /// Strategy:
    /// 1. Lower scrutinee into a temp
    /// 2. Read discriminant of scrutinee
    /// 3. SwitchInt on discriminant to per-arm blocks
    /// 4. In each arm block: bind pattern variables via Downcast+Field, lower body
    /// 5. Goto continuation block with result
    pub(crate) fn lower_match(
        &mut self,
        scrutinee: &Expr<'db>,
        arms: &[Arm<'db>],
    ) -> MResult<ir::Operand<'db>> {
        // 1. Lower the scrutinee
        let scrutinee_operand = self.lower_expr(scrutinee)?;
        let scrutinee_place = match &scrutinee_operand {
            ir::Operand::Place(p) => p.clone(),
            _ => {
                // Store in a temp if it's not already a place
                let scrutinee_ty = self.lookup_and_convert_type(scrutinee.id);
                let temp = self.allocate_temp(scrutinee_ty);
                self.emit_assign(ir::Place::Local(temp), ir::Rvalue::Use(scrutinee_operand));
                ir::Place::Local(temp)
            }
        };

        // Look up the enum name from the scrutinee's type
        let scrutinee_ty = self.lookup_and_convert_type(scrutinee.id);
        let enum_name = match &scrutinee_ty {
            ir::Ty::Adt(type_id) => type_id.name(self.db).to_string(),
            _ => return Err(BuilderError::LowerExpressionError),
        };

        // 2. Read discriminant
        let disc_temp = self.allocate_temp(ir::Ty::Int(scrap_shared::types::IntTy::I64));
        self.emit_assign(
            ir::Place::Local(disc_temp),
            ir::Rvalue::Discriminant(scrutinee_place.clone()),
        );

        // 3. Allocate blocks: one per arm + continuation + unreachable
        let cont_bb = self.cfg_builder.start_block();
        let unreachable_bb = self.cfg_builder.start_block();

        let mut arm_blocks = Vec::new();
        for _ in arms {
            arm_blocks.push(self.cfg_builder.start_block());
        }

        // Build SwitchTargets: map each arm's pattern to its variant index
        let mut switch_values = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            if let Some(variant_idx) = self.resolve_pattern_variant_idx(&arm.pat, &enum_name) {
                switch_values.push((variant_idx as u128, arm_blocks[i]));
            }
            // Wildcard/ident patterns will use the otherwise branch
        }

        // Find the otherwise arm (wildcard or ident catch-all)
        let otherwise_bb = arms
            .iter()
            .enumerate()
            .find(|(_, arm)| matches!(arm.pat.kind, PatKind::Wildcard | PatKind::Ident(_, _, _)))
            .map(|(i, _)| arm_blocks[i])
            .unwrap_or(unreachable_bb);

        // Emit SwitchInt
        let terminator = ir::Terminator::SwitchInt {
            discr: ir::Operand::Place(ir::Place::Local(disc_temp)),
            targets: ir::SwitchTargets {
                values: switch_values,
                otherwise: otherwise_bb,
            },
        };
        self.cfg_builder.finish_block(terminator);

        // Allocate result temp
        // Look up match result type from the first arm body
        let result_ty = self.lookup_and_convert_type(arms[0].body.id);
        let result_temp = self.allocate_temp(result_ty);

        // 4. Lower each arm
        for (i, arm) in arms.iter().enumerate() {
            self.cfg_builder.set_current_block(arm_blocks[i]);

            // Bind pattern variables
            self.bind_pattern_variables(&arm.pat, &scrutinee_place, &enum_name)?;

            // Lower arm body
            let body_operand = self.lower_expr(&arm.body)?;

            // Assign body result to result temp
            if !self.cfg_builder.current_block_is_terminated() {
                self.emit_assign(ir::Place::Local(result_temp), ir::Rvalue::Use(body_operand));
                self.cfg_builder
                    .finish_block(ir::Terminator::Goto { target: cont_bb });
            }
        }

        // Set up unreachable block
        self.cfg_builder.set_current_block(unreachable_bb);
        self.cfg_builder.finish_block(ir::Terminator::Unreachable);

        // Continue at continuation block
        self.cfg_builder.set_current_block(cont_bb);

        Ok(ir::Operand::Place(ir::Place::Local(result_temp)))
    }

    /// Resolve a pattern to its variant index within the enum.
    fn resolve_pattern_variant_idx(
        &self,
        pat: &scrap_ast::pat::Pat<'db>,
        enum_name: &str,
    ) -> Option<usize> {
        match &pat.kind {
            PatKind::Path(path) | PatKind::TupleStruct(path, _) | PatKind::Struct(path, _) => {
                if path.segments.len() == 2 {
                    let variant_name = path.segments[1].ident.name;
                    if let Some(enum_info) = self.enum_info.get(enum_name) {
                        return enum_info
                            .variants
                            .iter()
                            .find(|(name, _, _)| *name == variant_name)
                            .map(|(_, idx, _)| *idx);
                    }
                }
                None
            }
            // Wildcard and ident patterns match everything — no specific index
            _ => None,
        }
    }

    /// Bind pattern variables from a match arm pattern.
    fn bind_pattern_variables(
        &mut self,
        pat: &scrap_ast::pat::Pat<'db>,
        scrutinee_place: &ir::Place<'db>,
        enum_name: &str,
    ) -> MResult<()> {
        match &pat.kind {
            PatKind::Wildcard | PatKind::Missing | PatKind::Lit(_) | PatKind::Path(_) => {
                // No variables to bind
                Ok(())
            }
            PatKind::Ident(_, ident, _) => {
                // Catch-all binding: bind the whole scrutinee
                let scrutinee_operand = ir::Operand::Place(scrutinee_place.clone());
                let ty = self.lookup_and_convert_type(pat.id);
                let local_id = self.allocate_named_local(ident.name, ty);
                self.insert_binding(ident.name, local_id);
                self.emit_assign(
                    ir::Place::Local(local_id),
                    ir::Rvalue::Use(scrutinee_operand),
                );
                Ok(())
            }
            PatKind::TupleStruct(path, sub_pats) => {
                if path.segments.len() != 2 {
                    return Err(BuilderError::LowerExpressionError);
                }
                let variant_name = path.segments[1].ident.name;
                let variant_idx = self
                    .resolve_pattern_variant_idx(pat, enum_name)
                    .ok_or(BuilderError::LowerExpressionError)?;

                // For each sub-pattern, project via Downcast + Field
                for (field_idx, sub_pat) in sub_pats.iter().enumerate() {
                    if let PatKind::Ident(_, ident, _) = &sub_pat.kind {
                        // Look up the field type from enum info
                        let field_ty = self
                            .lookup_variant_field_ty(enum_name, variant_idx, field_idx)
                            .unwrap_or(ir::Ty::Void);

                        let local_id = self.allocate_named_local(ident.name, field_ty);
                        self.insert_binding(ident.name, local_id);

                        // _local = copy (scrutinee as Variant).field_idx
                        let downcast_place = ir::Place::Downcast(
                            Box::new(scrutinee_place.clone()),
                            variant_idx,
                            Some(variant_name),
                        );
                        let field_place = ir::Place::Field(
                            Box::new(downcast_place),
                            field_idx,
                            None, // tuple variant fields are positional, no name
                        );
                        self.emit_assign(
                            ir::Place::Local(local_id),
                            ir::Rvalue::Use(ir::Operand::Place(field_place)),
                        );
                    }
                }
                Ok(())
            }
            PatKind::Struct(path, field_pats) => {
                if path.segments.len() != 2 {
                    return Err(BuilderError::LowerExpressionError);
                }
                let variant_name = path.segments[1].ident.name;
                let variant_idx = self
                    .resolve_pattern_variant_idx(pat, enum_name)
                    .ok_or(BuilderError::LowerExpressionError)?;

                for fp in field_pats {
                    if let PatKind::Ident(_, ident, _) = &fp.pat.kind {
                        // Resolve field index by name
                        let field_idx = self
                            .lookup_variant_field_idx(enum_name, variant_idx, fp.ident.name)
                            .ok_or(BuilderError::LowerExpressionError)?;
                        let field_ty = self
                            .lookup_variant_field_ty(enum_name, variant_idx, field_idx)
                            .unwrap_or(ir::Ty::Void);

                        let local_id = self.allocate_named_local(ident.name, field_ty);
                        self.insert_binding(ident.name, local_id);

                        let downcast_place = ir::Place::Downcast(
                            Box::new(scrutinee_place.clone()),
                            variant_idx,
                            Some(variant_name),
                        );
                        let field_place =
                            ir::Place::Field(Box::new(downcast_place), field_idx, Some(ident.name));
                        self.emit_assign(
                            ir::Place::Local(local_id),
                            ir::Rvalue::Use(ir::Operand::Place(field_place)),
                        );
                    }
                }
                Ok(())
            }
        }
    }

    /// Look up a variant's field type by variant index and field index.
    fn lookup_variant_field_ty(
        &self,
        enum_name: &str,
        variant_idx: usize,
        field_idx: usize,
    ) -> Option<ir::Ty<'db>> {
        let enum_info = self.enum_info.get(enum_name)?;
        let (_, _, variant_info) = enum_info.variants.get(variant_idx)?;
        match variant_info {
            crate::lowerer::VariantInfo::Tuple(tys) => tys.get(field_idx).cloned(),
            crate::lowerer::VariantInfo::Struct(fields) => {
                fields.get(field_idx).map(|(_, ty)| ty.clone())
            }
            crate::lowerer::VariantInfo::Unit => None,
        }
    }

    /// Look up a variant's field index by name (for struct variants).
    fn lookup_variant_field_idx(
        &self,
        enum_name: &str,
        variant_idx: usize,
        field_name: Symbol<'db>,
    ) -> Option<usize> {
        let enum_info = self.enum_info.get(enum_name)?;
        let (_, _, variant_info) = enum_info.variants.get(variant_idx)?;
        match variant_info {
            crate::lowerer::VariantInfo::Struct(fields) => {
                fields.iter().position(|(name, _)| *name == field_name)
            }
            _ => None,
        }
    }
}
