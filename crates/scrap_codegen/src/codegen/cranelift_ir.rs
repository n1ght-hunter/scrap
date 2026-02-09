//! Translation of IR statements, terminators, operands, and rvalues to Cranelift instructions.

use cranelift::prelude::*;
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use cranelift_object::ObjectModule;
use scrap_ir as ir;
use scrap_shared::types::{FloatVal, IntVal, UintVal};
use std::collections::HashMap;

use super::emit_codegen_err;
use super::ResultExt;

/// Compute the (size, align, pointer_offsets) for a GC-allocated type.
fn compute_type_layout(_db: &dyn scrap_shared::Db, ty: &ir::Ty) -> (u64, u64, Vec<u64>) {
    match ty {
        ir::Ty::Bool => (1, 1, vec![]),
        ir::Ty::Int(k) => {
            let bytes = match k {
                scrap_shared::types::IntTy::I8 => 1,
                scrap_shared::types::IntTy::I16 => 2,
                scrap_shared::types::IntTy::I32 => 4,
                scrap_shared::types::IntTy::I64 | scrap_shared::types::IntTy::Isize => 8,
                scrap_shared::types::IntTy::I128 => 16,
            };
            (bytes, bytes, vec![])
        }
        ir::Ty::Uint(k) => {
            let bytes = match k {
                scrap_shared::types::UintTy::U8 => 1,
                scrap_shared::types::UintTy::U16 => 2,
                scrap_shared::types::UintTy::U32 => 4,
                scrap_shared::types::UintTy::U64 | scrap_shared::types::UintTy::Usize => 8,
                scrap_shared::types::UintTy::U128 => 16,
            };
            (bytes, bytes, vec![])
        }
        ir::Ty::Float(k) => {
            let bytes = match k {
                scrap_shared::types::FloatTy::F16 => 2,
                scrap_shared::types::FloatTy::F32 => 4,
                scrap_shared::types::FloatTy::F64 => 8,
                scrap_shared::types::FloatTy::F128 => 16,
            };
            (bytes, bytes, vec![])
        }
        ir::Ty::Str => (8, 8, vec![]),
        ir::Ty::Ref(_, _) => (8, 8, vec![0]),
        ir::Ty::Ptr(_) => (8, 8, vec![0]),
        _ => (8, 8, vec![]),
    }
}

/// Per-function translation context (holds only immutable/shared data).
pub struct FuncTranslator<'a, 'db> {
    pub db: &'db dyn scrap_shared::Db,
    /// IR LocalId index → Cranelift Variable (for non-tuple locals)
    pub variables: &'a HashMap<usize, Variable>,
    /// (IR LocalId index, field_index) → Cranelift Variable (for tuple locals)
    pub tuple_variables: &'a HashMap<(usize, usize), Variable>,
    /// IR BasicBlockId → Cranelift Block
    pub block_map: &'a HashMap<usize, Block>,
    /// Function name → FuncId (for call resolution)
    pub functions: &'a HashMap<String, FuncId>,
    /// Local declarations for type lookup
    pub local_decls: &'db [ir::LocalDecl<'db>],
    /// Whether the function returns void/never
    pub returns_void: bool,
    /// Counter for unique data section labels (interior mutability for &self methods)
    pub next_data_id: &'a std::cell::Cell<usize>,
    /// GcShape DataIds for box allocation (interior mutability)
    pub gc_shapes: &'a std::cell::RefCell<HashMap<String, cranelift_module::DataId>>,
    /// Struct name → field types (for aggregate construction)
    pub struct_layouts: &'a HashMap<String, Vec<ir::Ty<'db>>>,
    /// Enum name → per-variant field types
    pub enum_layouts: &'a HashMap<String, Vec<Vec<ir::Ty<'db>>>>,
    /// IR local index → Cranelift Variable for the discriminant
    pub enum_discriminants: &'a HashMap<usize, Variable>,
    /// (local_id, variant_idx, field_idx) → Cranelift Variable for variant fields
    pub enum_variant_variables: &'a HashMap<(usize, usize, usize), Variable>,
}

impl<'a, 'db> FuncTranslator<'a, 'db> {
    pub fn lower_statement(
        &self,
        stmt: ir::Statement<'db>,
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
    ) -> Option<()> {
        match stmt.kind(self.db) {
            ir::StatementKind::Assign(place, rvalue) => {
                // Special handling for checked intrinsics that produce tuple values
                if let ir::Rvalue::Intrinsic(op, ref operands) = rvalue {
                    if Self::is_checked_intrinsic(op) {
                        return self.lower_checked_intrinsic_assign(
                            &place, op, operands, builder, module,
                        );
                    }
                }

                // Special handling for aggregate (struct/tuple/enum) construction
                if let ir::Rvalue::Aggregate(ref kind, ref fields) = rvalue {
                    return self.lower_aggregate_assign_with_kind(&place, kind, fields, builder, module);
                }

                let value = self.lower_rvalue(&rvalue, builder, module)?;
                self.assign_to_place(&place, value, builder)?;
            }
        }
        Some(())
    }

    fn assign_to_place(
        &self,
        place: &ir::Place<'db>,
        value: Value,
        builder: &mut FunctionBuilder,
    ) -> Option<()> {
        match place {
            ir::Place::Local(local_id) => {
                let var = match self.variables.get(&local_id.0) {
                    Some(v) => v,
                    None => {
                        emit_codegen_err(self.db, format!("variable '_{}' not found", local_id.0));
                        return None;
                    }
                };
                builder.def_var(*var, value);
                Some(())
            }
            ir::Place::Field(base, field_idx, _) => {
                // Check for enum variant field: Field(Downcast(Local(x), variant_idx), field_idx)
                if let ir::Place::Downcast(inner, variant_idx, _) = base.as_ref() {
                    if let ir::Place::Local(local_id) = inner.as_ref() {
                        let var = match self.enum_variant_variables.get(&(local_id.0, *variant_idx, *field_idx)) {
                            Some(v) => v,
                            None => {
                                emit_codegen_err(
                                    self.db,
                                    format!(
                                        "enum variant variable '_{}.variant{}.{}' not found",
                                        local_id.0, variant_idx, field_idx
                                    ),
                                );
                                return None;
                            }
                        };
                        builder.def_var(*var, value);
                        return Some(());
                    }
                }
                // Struct/tuple field: Field(Local(x), field_idx)
                if let ir::Place::Local(local_id) = base.as_ref() {
                    let var = match self.tuple_variables.get(&(local_id.0, *field_idx)) {
                        Some(v) => v,
                        None => {
                            emit_codegen_err(
                                self.db,
                                format!(
                                    "tuple variable '_{}.{}' not found",
                                    local_id.0, field_idx
                                ),
                            );
                            return None;
                        }
                    };
                    builder.def_var(*var, value);
                    Some(())
                } else {
                    emit_codegen_err(self.db, "nested field projection not supported");
                    None
                }
            }
            ir::Place::Deref(inner) => {
                // Write through a GC reference: store value at the address held by inner
                let ptr = self.lower_place(inner, builder)?;
                builder.ins().store(MemFlags::new(), value, ptr, 0);
                Some(())
            }
            ir::Place::Downcast(_, _, _) => {
                // Downcast alone is not a valid assignment target;
                // it should always be wrapped in Field(Downcast(...), ...)
                emit_codegen_err(self.db, "downcast alone is not a valid assignment target");
                None
            }
            ir::Place::__Phantom(_) => unreachable!(),
        }
    }

    fn lower_rvalue(
        &self,
        rvalue: &ir::Rvalue<'db>,
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
    ) -> Option<Value> {
        match rvalue {
            ir::Rvalue::Use(operand) => self.lower_operand(operand, builder, module),
            ir::Rvalue::Constant(c) => self.lower_constant(c, builder, module),
            ir::Rvalue::Intrinsic(op, operands) => {
                // Non-checked intrinsics (binary and unary)
                self.lower_unchecked_intrinsic(*op, operands, builder, module)
            }
            ir::Rvalue::Aggregate(_, _) => {
                emit_codegen_err(self.db, "aggregate construction is not yet supported");
                None
            }
            ir::Rvalue::Array(_) => {
                emit_codegen_err(self.db, "array literal is not yet supported");
                None
            }
            ir::Rvalue::Box(inner_ty, value_op) => {
                self.lower_box_alloc(inner_ty, value_op, builder, module)
            }
            ir::Rvalue::Discriminant(place) => {
                // Read the discriminant variable for an enum local
                let local_id = match place {
                    ir::Place::Local(id) => id.0,
                    _ => {
                        emit_codegen_err(self.db, "discriminant must operate on a local");
                        return None;
                    }
                };
                let disc_var = match self.enum_discriminants.get(&local_id) {
                    Some(v) => v,
                    None => {
                        emit_codegen_err(
                            self.db,
                            format!("enum discriminant for '_{local_id}' not found"),
                        );
                        return None;
                    }
                };
                Some(builder.use_var(*disc_var))
            }
        }
    }

    /// Lower an unchecked intrinsic (binary op, unary op, comparison, etc.)
    /// that produces a single value.
    fn lower_unchecked_intrinsic(
        &self,
        op: ir::IntrinsicOp,
        operands: &[ir::Operand<'db>],
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
    ) -> Option<Value> {
        match op {
            // Binary operations
            ir::IntrinsicOp::Add
            | ir::IntrinsicOp::Sub
            | ir::IntrinsicOp::Mul
            | ir::IntrinsicOp::Div
            | ir::IntrinsicOp::Rem
            | ir::IntrinsicOp::Eq
            | ir::IntrinsicOp::Ne
            | ir::IntrinsicOp::Lt
            | ir::IntrinsicOp::Le
            | ir::IntrinsicOp::Gt
            | ir::IntrinsicOp::Ge
            | ir::IntrinsicOp::And
            | ir::IntrinsicOp::Or
            | ir::IntrinsicOp::BitAnd
            | ir::IntrinsicOp::BitOr
            | ir::IntrinsicOp::BitXor
            | ir::IntrinsicOp::Shl
            | ir::IntrinsicOp::Shr => {
                let lhs = self.lower_operand(&operands[0], builder, module)?;
                let rhs = self.lower_operand(&operands[1], builder, module)?;
                self.lower_binop(op, lhs, rhs, &operands[0], builder)
            }
            // Unary operations
            ir::IntrinsicOp::Neg | ir::IntrinsicOp::Not => {
                let val = self.lower_operand(&operands[0], builder, module)?;
                self.lower_unop(op, val, &operands[0], builder)
            }
            // Checked ops should not reach here (handled in lower_statement)
            _ => {
                emit_codegen_err(
                    self.db,
                    format!("unexpected checked intrinsic in unchecked context: {:?}", op),
                );
                None
            }
        }
    }

    /// Lower a checked intrinsic assignment: _pair = checked_op(lhs, rhs)
    /// Stores the result value and overflow flag into tuple sub-variables.
    fn lower_checked_intrinsic_assign(
        &self,
        place: &ir::Place<'db>,
        op: ir::IntrinsicOp,
        operands: &[ir::Operand<'db>],
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
    ) -> Option<()> {
        let local_id = match place {
            ir::Place::Local(id) => id.0,
            _ => {
                emit_codegen_err(self.db, "checked intrinsic must assign to a local");
                return None;
            }
        };

        let lhs = self.lower_operand(&operands[0], builder, module)?;
        let rhs = self.lower_operand(&operands[1], builder, module)?;
        let (result, overflow) =
            self.lower_checked_intrinsic(op, lhs, rhs, &operands[0], builder)?;

        // Store into tuple sub-variables
        let var0 = match self.tuple_variables.get(&(local_id, 0)) {
            Some(v) => v,
            None => {
                emit_codegen_err(
                    self.db,
                    format!("tuple variable '_{}.0' not found", local_id),
                );
                return None;
            }
        };
        let var1 = match self.tuple_variables.get(&(local_id, 1)) {
            Some(v) => v,
            None => {
                emit_codegen_err(
                    self.db,
                    format!("tuple variable '_{}.1' not found", local_id),
                );
                return None;
            }
        };
        builder.def_var(*var0, result);
        builder.def_var(*var1, overflow);
        Some(())
    }

    /// Lower an aggregate (struct/tuple/enum variant) assignment.
    fn lower_aggregate_assign_with_kind(
        &self,
        place: &ir::Place<'db>,
        kind: &ir::AggregateKind<'db>,
        fields: &[ir::Operand<'db>],
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
    ) -> Option<()> {
        let local_id = match place {
            ir::Place::Local(id) => id.0,
            _ => {
                emit_codegen_err(self.db, "aggregate must assign to a local");
                return None;
            }
        };

        match kind {
            ir::AggregateKind::EnumVariant(_, variant_idx) => {
                // Set discriminant to variant_idx
                let disc_var = match self.enum_discriminants.get(&local_id) {
                    Some(v) => v,
                    None => {
                        emit_codegen_err(
                            self.db,
                            format!("enum discriminant for '_{local_id}' not found"),
                        );
                        return None;
                    }
                };
                let disc_val = builder.ins().iconst(types::I64, *variant_idx as i64);
                builder.def_var(*disc_var, disc_val);

                // Set variant field variables
                for (field_idx, operand) in fields.iter().enumerate() {
                    let value = self.lower_operand(operand, builder, module)?;
                    let var = match self.enum_variant_variables.get(&(local_id, *variant_idx, field_idx)) {
                        Some(v) => v,
                        None => {
                            emit_codegen_err(
                                self.db,
                                format!(
                                    "enum variant variable '_{}.variant{}.{}' not found",
                                    local_id, variant_idx, field_idx
                                ),
                            );
                            return None;
                        }
                    };
                    builder.def_var(*var, value);
                }
                Some(())
            }
            ir::AggregateKind::Struct(_, _) => {
                // Struct/tuple: store each field into tuple sub-variables
                for (field_idx, operand) in fields.iter().enumerate() {
                    let value = self.lower_operand(operand, builder, module)?;
                    let var = match self.tuple_variables.get(&(local_id, field_idx)) {
                        Some(v) => v,
                        None => {
                            emit_codegen_err(
                                self.db,
                                format!("tuple variable '_{}.{}' not found", local_id, field_idx),
                            );
                            return None;
                        }
                    };
                    builder.def_var(*var, value);
                }
                Some(())
            }
        }
    }

    /// Lower a checked intrinsic to produce (result, overflow_flag).
    fn lower_checked_intrinsic(
        &self,
        op: ir::IntrinsicOp,
        lhs: Value,
        rhs: Value,
        lhs_operand: &ir::Operand<'db>,
        builder: &mut FunctionBuilder,
    ) -> Option<(Value, Value)> {
        let signed = self.is_signed_operand(lhs_operand);

        match op {
            ir::IntrinsicOp::AddWithOverflow => {
                let (result, overflow) = if signed {
                    builder.ins().sadd_overflow(lhs, rhs)
                } else {
                    builder.ins().uadd_overflow(lhs, rhs)
                };
                Some((result, overflow))
            }
            ir::IntrinsicOp::SubWithOverflow => {
                let (result, overflow) = if signed {
                    builder.ins().ssub_overflow(lhs, rhs)
                } else {
                    builder.ins().usub_overflow(lhs, rhs)
                };
                Some((result, overflow))
            }
            ir::IntrinsicOp::MulWithOverflow => {
                let (result, overflow) = if signed {
                    builder.ins().smul_overflow(lhs, rhs)
                } else {
                    builder.ins().umul_overflow(lhs, rhs)
                };
                Some((result, overflow))
            }
            ir::IntrinsicOp::DivWithZeroCheck => {
                // Check rhs == 0
                let cl_ty = builder.func.dfg.value_type(rhs);
                let zero = builder.ins().iconst(cl_ty, 0);
                let is_zero = builder.ins().icmp(IntCC::Equal, rhs, zero);

                // Perform the division (will be guarded by Assert in the IR)
                let result = if signed {
                    builder.ins().sdiv(lhs, rhs)
                } else {
                    builder.ins().udiv(lhs, rhs)
                };
                Some((result, is_zero))
            }
            ir::IntrinsicOp::RemWithZeroCheck => {
                let cl_ty = builder.func.dfg.value_type(rhs);
                let zero = builder.ins().iconst(cl_ty, 0);
                let is_zero = builder.ins().icmp(IntCC::Equal, rhs, zero);

                let result = if signed {
                    builder.ins().srem(lhs, rhs)
                } else {
                    builder.ins().urem(lhs, rhs)
                };
                Some((result, is_zero))
            }
            ir::IntrinsicOp::ShlChecked | ir::IntrinsicOp::ShrChecked => {
                // Check if shift amount >= bit width
                let cl_ty = builder.func.dfg.value_type(lhs);
                let bit_width = cl_ty.bits() as i64;
                let max_shift = builder.ins().iconst(cl_ty, bit_width);
                // Overflow if rhs >= bit_width (unsigned comparison)
                let overflow = builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, rhs, max_shift);

                let result = if op == ir::IntrinsicOp::ShlChecked {
                    builder.ins().ishl(lhs, rhs)
                } else if signed {
                    builder.ins().sshr(lhs, rhs)
                } else {
                    builder.ins().ushr(lhs, rhs)
                };
                Some((result, overflow))
            }
            _ => {
                emit_codegen_err(
                    self.db,
                    format!("unexpected intrinsic in checked context: {:?}", op),
                );
                None
            }
        }
    }

    /// Check if an IntrinsicOp is a checked variant.
    fn is_checked_intrinsic(op: ir::IntrinsicOp) -> bool {
        matches!(
            op,
            ir::IntrinsicOp::AddWithOverflow
                | ir::IntrinsicOp::SubWithOverflow
                | ir::IntrinsicOp::MulWithOverflow
                | ir::IntrinsicOp::DivWithZeroCheck
                | ir::IntrinsicOp::RemWithZeroCheck
                | ir::IntrinsicOp::ShlChecked
                | ir::IntrinsicOp::ShrChecked
        )
    }

    pub fn lower_operand(
        &self,
        operand: &ir::Operand<'db>,
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
    ) -> Option<Value> {
        match operand {
            ir::Operand::Place(place) => self.lower_place(place, builder),
            ir::Operand::Constant(c) => self.lower_constant(c, builder, module),
            ir::Operand::FunctionRef(_) => {
                emit_codegen_err(
                    self.db,
                    "function ref as value is not supported (handled in Call terminator)",
                );
                None
            }
        }
    }

    /// Lower an operand for use as a function call argument.
    /// If the operand is a struct (ADT), expand it into individual field values.
    fn lower_operand_expanding_structs(
        &self,
        operand: &ir::Operand<'db>,
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
        out: &mut Vec<Value>,
    ) -> Option<()> {
        // Check if the operand is a Place referencing a struct local
        if let ir::Operand::Place(ir::Place::Local(local_id)) = operand {
            let decl_ty = self.local_decls[local_id.0].ty(self.db);
            if let ir::Ty::Adt(type_id) = &decl_ty {
                let adt_name = type_id.name(self.db);
                if let Some(field_tys) = self.struct_layouts.get(adt_name.as_str()) {
                    // Expand struct into per-field values
                    for (field_idx, _) in field_tys.iter().enumerate() {
                        if let Some(var) = self.tuple_variables.get(&(local_id.0, field_idx)) {
                            out.push(builder.use_var(*var));
                        } else {
                            emit_codegen_err(
                                self.db,
                                format!("struct field variable not found for _{}.{}", local_id.0, field_idx),
                            );
                            return None;
                        }
                    }
                    return Some(());
                }
            }
        }
        // Not a struct — lower as a single value
        let val = self.lower_operand(operand, builder, module)?;
        out.push(val);
        Some(())
    }

    fn lower_place(
        &self,
        place: &ir::Place<'db>,
        builder: &mut FunctionBuilder,
    ) -> Option<Value> {
        match place {
            ir::Place::Local(local_id) => {
                let var = match self.variables.get(&local_id.0) {
                    Some(v) => v,
                    None => {
                        emit_codegen_err(self.db, format!("variable '_{}' not found", local_id.0));
                        return None;
                    }
                };
                Some(builder.use_var(*var))
            }
            ir::Place::Field(base, field_idx, _) => {
                // Check for enum variant field read: Field(Downcast(Local(x), variant_idx), field_idx)
                if let ir::Place::Downcast(inner, variant_idx, _) = base.as_ref() {
                    if let ir::Place::Local(local_id) = inner.as_ref() {
                        let var = match self.enum_variant_variables.get(&(local_id.0, *variant_idx, *field_idx)) {
                            Some(v) => v,
                            None => {
                                emit_codegen_err(
                                    self.db,
                                    format!(
                                        "enum variant variable '_{}.variant{}.{}' not found",
                                        local_id.0, variant_idx, field_idx
                                    ),
                                );
                                return None;
                            }
                        };
                        return Some(builder.use_var(*var));
                    }
                }
                // Struct/tuple field read: Field(Local(x), field_idx)
                if let ir::Place::Local(local_id) = base.as_ref() {
                    let var = match self.tuple_variables.get(&(local_id.0, *field_idx)) {
                        Some(v) => v,
                        None => {
                            emit_codegen_err(
                                self.db,
                                format!(
                                    "tuple variable '_{}.{}' not found",
                                    local_id.0, field_idx
                                ),
                            );
                            return None;
                        }
                    };
                    Some(builder.use_var(*var))
                } else {
                    emit_codegen_err(self.db, "nested field projection not supported");
                    None
                }
            }
            ir::Place::Deref(inner) => {
                // Read through a GC reference: load value from the address held by inner
                let ptr = self.lower_place(inner, builder)?;
                // Determine the pointed-to type for the load
                let result_ty = self.deref_result_type(inner)?;
                Some(builder.ins().load(result_ty, MemFlags::new(), ptr, 0))
            }
            ir::Place::Downcast(_, _, _) => {
                // Downcast alone is not a valid read target;
                // it should always be wrapped in Field(Downcast(...), ...)
                emit_codegen_err(self.db, "downcast alone is not a valid read target");
                None
            }
            ir::Place::__Phantom(_) => unreachable!(),
        }
    }

    fn lower_constant(
        &self,
        c: &ir::Constant<'db>,
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
    ) -> Option<Value> {
        match c {
            ir::Constant::Int(int_val) => {
                let (cl_ty, val): (types::Type, i64) = match int_val {
                    IntVal::I8(v) => (types::I8, *v as i64),
                    IntVal::I16(v) => (types::I16, *v as i64),
                    IntVal::I32(v) => (types::I32, *v as i64),
                    IntVal::I64(v) => (types::I64, *v),
                    IntVal::Isize(v) => (types::I64, *v as i64),
                    IntVal::I128(_) => {
                        emit_codegen_err(self.db, "i128 constant is not supported");
                        return None;
                    }
                };
                Some(builder.ins().iconst(cl_ty, val))
            }
            ir::Constant::Uint(uint_val) => {
                let (cl_ty, val): (types::Type, i64) = match uint_val {
                    UintVal::U8(v) => (types::I8, *v as i64),
                    UintVal::U16(v) => (types::I16, *v as i64),
                    UintVal::U32(v) => (types::I32, *v as i64),
                    UintVal::U64(v) => (types::I64, *v as i64),
                    UintVal::Usize(v) => (types::I64, *v as i64),
                    UintVal::U128(_) => {
                        emit_codegen_err(self.db, "u128 constant is not supported");
                        return None;
                    }
                };
                Some(builder.ins().iconst(cl_ty, val))
            }
            ir::Constant::Float(float_val) => match float_val {
                FloatVal::F32(v) => Some(builder.ins().f32const(*v)),
                FloatVal::F64(v) => Some(builder.ins().f64const(*v)),
            },
            ir::Constant::Bool(b) => {
                Some(builder.ins().iconst(types::I8, *b as i64))
            }
            ir::Constant::String(sym) => {
                let s = sym.text(self.db);
                let bytes = s.as_bytes();

                let id = self.next_data_id.get();
                self.next_data_id.set(id + 1);
                let name = format!(".Lstr.{id}");

                let data_id = module
                    .declare_data(&name, Linkage::Local, false, false)
                    .or_emit(self.db)?;
                let mut desc = DataDescription::new();
                desc.define(bytes.to_vec().into_boxed_slice());
                module.define_data(data_id, &desc).or_emit(self.db)?;

                let gv = module.declare_data_in_func(data_id, builder.func);
                let addr = builder.ins().global_value(types::I64, gv);
                Some(addr)
            }
            ir::Constant::Void => {
                emit_codegen_err(self.db, "void constant is not supported");
                None
            }
        }
    }

    fn lower_binop(
        &self,
        op: ir::IntrinsicOp,
        lhs: Value,
        rhs: Value,
        lhs_operand: &ir::Operand<'db>,
        builder: &mut FunctionBuilder,
    ) -> Option<Value> {
        let is_float = self.is_float_operand(lhs_operand);
        let signed = self.is_signed_operand(lhs_operand);

        match op {
            ir::IntrinsicOp::Add => {
                if is_float {
                    Some(builder.ins().fadd(lhs, rhs))
                } else {
                    Some(builder.ins().iadd(lhs, rhs))
                }
            }
            ir::IntrinsicOp::Sub => {
                if is_float {
                    Some(builder.ins().fsub(lhs, rhs))
                } else {
                    Some(builder.ins().isub(lhs, rhs))
                }
            }
            ir::IntrinsicOp::Mul => {
                if is_float {
                    Some(builder.ins().fmul(lhs, rhs))
                } else {
                    Some(builder.ins().imul(lhs, rhs))
                }
            }
            ir::IntrinsicOp::Div => {
                if is_float {
                    Some(builder.ins().fdiv(lhs, rhs))
                } else if signed {
                    Some(builder.ins().sdiv(lhs, rhs))
                } else {
                    Some(builder.ins().udiv(lhs, rhs))
                }
            }
            ir::IntrinsicOp::Rem => {
                if is_float {
                    // Cranelift doesn't have a float rem instruction on all targets,
                    // but for now emit the intrinsic
                    emit_codegen_err(self.db, "float remainder is not yet supported");
                    None
                } else if signed {
                    Some(builder.ins().srem(lhs, rhs))
                } else {
                    Some(builder.ins().urem(lhs, rhs))
                }
            }
            ir::IntrinsicOp::BitAnd | ir::IntrinsicOp::And => {
                Some(builder.ins().band(lhs, rhs))
            }
            ir::IntrinsicOp::BitOr | ir::IntrinsicOp::Or => {
                Some(builder.ins().bor(lhs, rhs))
            }
            ir::IntrinsicOp::BitXor => Some(builder.ins().bxor(lhs, rhs)),
            ir::IntrinsicOp::Shl => Some(builder.ins().ishl(lhs, rhs)),
            ir::IntrinsicOp::Shr => {
                if signed {
                    Some(builder.ins().sshr(lhs, rhs))
                } else {
                    Some(builder.ins().ushr(lhs, rhs))
                }
            }
            ir::IntrinsicOp::Eq => Some(builder.ins().icmp(IntCC::Equal, lhs, rhs)),
            ir::IntrinsicOp::Ne => Some(builder.ins().icmp(IntCC::NotEqual, lhs, rhs)),
            ir::IntrinsicOp::Lt => {
                if signed {
                    Some(builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs))
                } else {
                    Some(builder.ins().icmp(IntCC::UnsignedLessThan, lhs, rhs))
                }
            }
            ir::IntrinsicOp::Le => {
                if signed {
                    Some(builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs))
                } else {
                    Some(builder.ins().icmp(IntCC::UnsignedLessThanOrEqual, lhs, rhs))
                }
            }
            ir::IntrinsicOp::Gt => {
                if signed {
                    Some(builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs))
                } else {
                    Some(builder.ins().icmp(IntCC::UnsignedGreaterThan, lhs, rhs))
                }
            }
            ir::IntrinsicOp::Ge => {
                if signed {
                    Some(builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs))
                } else {
                    Some(builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, lhs, rhs))
                }
            }
            _ => {
                emit_codegen_err(
                    self.db,
                    format!("unexpected intrinsic in binary context: {:?}", op),
                );
                None
            }
        }
    }

    fn lower_unop(
        &self,
        op: ir::IntrinsicOp,
        val: Value,
        operand: &ir::Operand<'db>,
        builder: &mut FunctionBuilder,
    ) -> Option<Value> {
        match op {
            ir::IntrinsicOp::Neg => {
                if self.is_float_operand(operand) {
                    Some(builder.ins().fneg(val))
                } else {
                    Some(builder.ins().ineg(val))
                }
            }
            ir::IntrinsicOp::Not => Some(builder.ins().bnot(val)),
            _ => {
                emit_codegen_err(
                    self.db,
                    format!("unexpected intrinsic in unary context: {:?}", op),
                );
                None
            }
        }
    }

    pub fn lower_terminator(
        &self,
        term: &ir::Terminator<'db>,
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
    ) -> Option<()> {
        match term {
            ir::Terminator::Return => {
                if self.returns_void {
                    builder.ins().return_(&[]);
                } else {
                    let ret_var = match self.variables.get(&0) {
                        Some(v) => v,
                        None => {
                            emit_codegen_err(self.db, "variable '_0' (return place) not found");
                            return None;
                        }
                    };
                    let ret_val = builder.use_var(*ret_var);
                    builder.ins().return_(&[ret_val]);
                }
                Some(())
            }
            ir::Terminator::Goto { target } => {
                let target_block = match self.block_map.get(&target.0) {
                    Some(b) => b,
                    None => {
                        emit_codegen_err(self.db, format!("unknown block bb{}", target.0));
                        return None;
                    }
                };
                builder.ins().jump(*target_block, &[]);
                Some(())
            }
            ir::Terminator::SwitchInt { discr, targets } => {
                let discr_val = self.lower_operand(discr, builder, module)?;
                let otherwise_block = self.block_map[&targets.otherwise.0];

                if targets.values.is_empty() {
                    // No specific values — unconditional jump to otherwise
                    builder.ins().jump(otherwise_block, &[]);
                } else if targets.values.len() == 1 && targets.values[0].0 == 0 {
                    // Special case: boolean switch (0 → false_block, otherwise → true_block)
                    let false_block = self.block_map[&targets.values[0].1 .0];
                    builder
                        .ins()
                        .brif(discr_val, otherwise_block, &[], false_block, &[]);
                } else {
                    // N-way switch: chain of comparisons
                    let cl_ty = builder.func.dfg.value_type(discr_val);
                    for (i, (val, bb)) in targets.values.iter().enumerate() {
                        let target_block = self.block_map[&bb.0];
                        let val_const = builder.ins().iconst(cl_ty, *val as i64);
                        let cmp = builder.ins().icmp(IntCC::Equal, discr_val, val_const);

                        if i == targets.values.len() - 1 {
                            // Last value: branch to target or otherwise
                            builder
                                .ins()
                                .brif(cmp, target_block, &[], otherwise_block, &[]);
                        } else {
                            // More values follow: branch to target or fall through to next comparison
                            let next_cmp_block = builder.create_block();
                            builder
                                .ins()
                                .brif(cmp, target_block, &[], next_cmp_block, &[]);
                            builder.switch_to_block(next_cmp_block);
                        }
                    }
                }
                Some(())
            }
            ir::Terminator::Call {
                func,
                args,
                destination,
                target,
                unwind: _,
            } => {
                let func_id = match func {
                    ir::Operand::FunctionRef(fid) => {
                        let name = fid.text(self.db);
                        match self.functions.get(name).copied() {
                            Some(id) => id,
                            None => {
                                emit_codegen_err(
                                    self.db,
                                    format!("function '{name}' not found"),
                                );
                                return None;
                            }
                        }
                    }
                    _ => {
                        emit_codegen_err(self.db, "indirect function call is not supported");
                        return None;
                    }
                };

                let func_ref = module.declare_func_in_func(func_id, builder.func);

                let mut arg_vals = Vec::new();
                for a in args.iter() {
                    self.lower_operand_expanding_structs(a, builder, module, &mut arg_vals)?;
                }

                let call = builder.ins().call(func_ref, &arg_vals);

                let results = builder.inst_results(call);
                if !results.is_empty() {
                    let result_val = results[0];
                    self.assign_to_place(destination, result_val, builder)?;
                }

                if let Some(target_bb) = target {
                    let target_block = match self.block_map.get(&target_bb.0) {
                        Some(b) => b,
                        None => {
                            emit_codegen_err(self.db, format!("unknown block bb{}", target_bb.0));
                            return None;
                        }
                    };
                    builder.ins().jump(*target_block, &[]);
                } else {
                    // Callee returns `!` — no continuation, trap as unreachable
                    builder.ins().trap(TrapCode::user(1).unwrap());
                }
                Some(())
            }
            ir::Terminator::Assert {
                cond,
                expected,
                msg,
                target,
                unwind: _,
            } => {
                let cond_val = self.lower_operand(cond, builder, module)?;
                let success_block = match self.block_map.get(&target.0) {
                    Some(b) => *b,
                    None => {
                        emit_codegen_err(self.db, format!("unknown block bb{}", target.0));
                        return None;
                    }
                };
                let fail_block = builder.create_block();

                if *expected {
                    // assert(cond == true): if true go to success, if false trap
                    builder
                        .ins()
                        .brif(cond_val, success_block, &[], fail_block, &[]);
                } else {
                    // assert(cond == false): if false (0) go to success, if true trap
                    builder
                        .ins()
                        .brif(cond_val, fail_block, &[], success_block, &[]);
                }

                builder.switch_to_block(fail_block);

                // Call __scrap_panic(msg_ptr, msg_len) to print the error and exit
                let msg_text = Self::assert_msg_str(msg);
                let msg_bytes = msg_text.as_bytes();
                let msg_len_val = msg_bytes.len();

                // Create a data section for the message string
                let id = self.next_data_id.get();
                self.next_data_id.set(id + 1);
                let data_name = format!(".Lpanic_msg.{id}");

                let data_id = module
                    .declare_data(&data_name, Linkage::Local, false, false)
                    .or_emit(self.db)?;
                let mut desc = DataDescription::new();
                desc.define(msg_bytes.to_vec().into_boxed_slice());
                module.define_data(data_id, &desc).or_emit(self.db)?;

                let gv = module.declare_data_in_func(data_id, builder.func);
                let msg_ptr = builder.ins().global_value(types::I64, gv);
                let msg_len = builder.ins().iconst(types::I64, msg_len_val as i64);

                // Call __scrap_panic
                if let Some(&panic_func_id) = self.functions.get("__scrap_panic") {
                    let panic_ref = module.declare_func_in_func(panic_func_id, builder.func);
                    builder.ins().call(panic_ref, &[msg_ptr, msg_len]);
                }

                // Trap as fallback (panic function diverges via ExitProcess)
                builder.ins().trap(TrapCode::user(2).unwrap());

                Some(())
            }
            ir::Terminator::Unreachable => {
                builder.ins().trap(TrapCode::user(1).unwrap());
                Some(())
            }
        }
    }

    /// Get the human-readable message string for an AssertMessage.
    fn assert_msg_str(msg: &ir::AssertMessage) -> &'static str {
        match msg {
            ir::AssertMessage::Overflow(ir::IntrinsicOp::AddWithOverflow) => {
                "attempt to add with overflow\n"
            }
            ir::AssertMessage::Overflow(ir::IntrinsicOp::SubWithOverflow) => {
                "attempt to subtract with overflow\n"
            }
            ir::AssertMessage::Overflow(ir::IntrinsicOp::MulWithOverflow) => {
                "attempt to multiply with overflow\n"
            }
            ir::AssertMessage::Overflow(_) => "arithmetic overflow\n",
            ir::AssertMessage::DivisionByZero => "attempt to divide by zero\n",
            ir::AssertMessage::RemainderByZero => {
                "attempt to calculate remainder with zero divisor\n"
            }
            ir::AssertMessage::ShiftOverflow => "attempt to shift with overflow\n",
        }
    }

    /// Check if an operand refers to a float type.
    fn is_float_operand(&self, operand: &ir::Operand<'db>) -> bool {
        if let ir::Operand::Place(ir::Place::Local(id)) = operand {
            if let Some(decl) = self.local_decls.get(id.0) {
                return matches!(decl.ty(self.db), ir::Ty::Float(_));
            }
        }
        if let ir::Operand::Constant(c) = operand {
            return matches!(c, ir::Constant::Float(_));
        }
        false
    }

    /// Lower a `Rvalue::Box(inner_ty, value)`: allocate GC memory and store the value.
    fn lower_box_alloc(
        &self,
        inner_ty: &ir::Ty<'db>,
        value_op: &ir::Operand<'db>,
        builder: &mut FunctionBuilder,
        module: &mut ObjectModule,
    ) -> Option<Value> {
        // 1. Get or create GcShape for inner_ty
        let shape_data_id = {
            let key = format!("{:?}", inner_ty);
            let mut shapes = self.gc_shapes.borrow_mut();
            if let Some(&id) = shapes.get(&key) {
                id
            } else {
                // Compute the shape inline (mirrors CodegenContext::compute_type_layout)
                let (size, align, pointer_offsets) = compute_type_layout(self.db, inner_ty);
                let num_pointers = pointer_offsets.len() as u64;
                let mut data = Vec::new();
                data.extend_from_slice(&size.to_le_bytes());
                data.extend_from_slice(&align.to_le_bytes());
                data.extend_from_slice(&num_pointers.to_le_bytes());
                for offset in &pointer_offsets {
                    data.extend_from_slice(&offset.to_le_bytes());
                }

                let name = format!(".Lgcshape.{}", shapes.len());
                let data_id = module
                    .declare_data(&name, Linkage::Local, false, false)
                    .or_emit(self.db)?;
                let mut desc = DataDescription::new();
                desc.define(data.into_boxed_slice());
                desc.set_align(8);
                module.define_data(data_id, &desc).or_emit(self.db)?;

                shapes.insert(key, data_id);
                data_id
            }
        };

        // 2. Load GcShape address
        let gv = module.declare_data_in_func(shape_data_id, builder.func);
        let shape_addr = builder.ins().global_value(types::I64, gv);

        // 3. Call __scrap_gc_alloc(shape_addr) → pointer to user data
        let alloc_func_id = match self.functions.get("__scrap_gc_alloc") {
            Some(&id) => id,
            None => {
                emit_codegen_err(self.db, "__scrap_gc_alloc not declared");
                return None;
            }
        };
        let alloc_ref = module.declare_func_in_func(alloc_func_id, builder.func);
        let call_inst = builder.ins().call(alloc_ref, &[shape_addr]);
        let ptr = builder.inst_results(call_inst)[0];

        // 4. Lower the value operand
        let value = self.lower_operand(value_op, builder, module)?;

        // 5. Store value at the allocated pointer
        builder.ins().store(MemFlags::new(), value, ptr, 0);

        // 6. Return the pointer
        Some(ptr)
    }

    /// Determine the Cranelift type of a dereferenced place.
    /// Given Place::Deref(inner), looks at the inner local's type to figure out what
    /// the reference points to (e.g., `&mut i32` → `I32`).
    fn deref_result_type(&self, inner_place: &ir::Place<'db>) -> Option<types::Type> {
        use super::ty::ir_ty_to_cl_required;

        match inner_place {
            ir::Place::Local(local_id) => {
                let decl = self.local_decls.get(local_id.0)?;
                match decl.ty(self.db) {
                    ir::Ty::Ref(inner_ty, _) => ir_ty_to_cl_required(self.db, &inner_ty),
                    ir::Ty::Ptr(inner_ty) => ir_ty_to_cl_required(self.db, &inner_ty),
                    _ => {
                        // Fallback: assume pointer-sized
                        Some(types::I64)
                    }
                }
            }
            _ => Some(types::I64), // fallback
        }
    }

    /// Check if an operand refers to a signed integer type.
    fn is_signed_operand(&self, operand: &ir::Operand<'db>) -> bool {
        if let ir::Operand::Place(ir::Place::Local(id)) = operand {
            if let Some(decl) = self.local_decls.get(id.0) {
                return matches!(decl.ty(self.db), ir::Ty::Int(_));
            }
        }
        if let ir::Operand::Constant(c) = operand {
            return matches!(c, ir::Constant::Int(_));
        }
        false
    }
}
