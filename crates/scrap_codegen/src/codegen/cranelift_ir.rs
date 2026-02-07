//! Translation of IR statements, terminators, operands, and rvalues to Cranelift instructions.

use cranelift::prelude::*;
use cranelift_module::{FuncId, Module};
use cranelift_object::ObjectModule;
use scrap_ir as ir;
use scrap_shared::types::{FloatVal, IntVal, UintVal};
use std::collections::HashMap;

use super::emit_codegen_err;

/// Per-function translation context (holds only immutable/shared data).
pub struct FuncTranslator<'a, 'db> {
    pub db: &'db dyn scrap_shared::Db,
    /// IR LocalId index → Cranelift Variable
    pub variables: &'a HashMap<usize, Variable>,
    /// IR BasicBlockId → Cranelift Block
    pub block_map: &'a HashMap<usize, Block>,
    /// Function name → FuncId (for call resolution)
    pub functions: &'a HashMap<String, FuncId>,
    /// Local declarations for type lookup
    pub local_decls: &'db [ir::LocalDecl<'db>],
    /// Whether the function returns void/never
    pub returns_void: bool,
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
            ir::Place::Field(_, _) => {
                emit_codegen_err(self.db, "field assignment is not yet supported");
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
        let _ = module; // may be used by future rvalue variants
        match rvalue {
            ir::Rvalue::Use(operand) => self.lower_operand(operand, builder),
            ir::Rvalue::Constant(c) => self.lower_constant(c, builder),
            ir::Rvalue::BinaryOp(op, lhs, rhs) => {
                let lhs_val = self.lower_operand(lhs, builder)?;
                let rhs_val = self.lower_operand(rhs, builder)?;
                self.lower_binop(*op, lhs_val, rhs_val, lhs, builder)
            }
            ir::Rvalue::UnaryOp(op, operand) => {
                let val = self.lower_operand(operand, builder)?;
                self.lower_unop(*op, val, operand, builder)
            }
            ir::Rvalue::Aggregate(_, _) => {
                emit_codegen_err(self.db, "aggregate construction is not yet supported");
                None
            }
            ir::Rvalue::Array(_) => {
                emit_codegen_err(self.db, "array literal is not yet supported");
                None
            }
        }
    }

    pub fn lower_operand(
        &self,
        operand: &ir::Operand<'db>,
        builder: &mut FunctionBuilder,
    ) -> Option<Value> {
        match operand {
            ir::Operand::Place(place) => self.lower_place(place, builder),
            ir::Operand::Constant(c) => self.lower_constant(c, builder),
            ir::Operand::FunctionRef(_) => {
                emit_codegen_err(
                    self.db,
                    "function ref as value is not supported (handled in Call terminator)",
                );
                None
            }
        }
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
            ir::Place::Field(_, _) => {
                emit_codegen_err(self.db, "field access is not yet supported");
                None
            }
            ir::Place::__Phantom(_) => unreachable!(),
        }
    }

    fn lower_constant(
        &self,
        c: &ir::Constant<'db>,
        builder: &mut FunctionBuilder,
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
            ir::Constant::String(_) => {
                emit_codegen_err(self.db, "string constant in codegen is not yet supported");
                None
            }
            ir::Constant::Void => {
                emit_codegen_err(self.db, "void constant is not supported");
                None
            }
        }
    }

    fn lower_binop(
        &self,
        op: ir::BinOp,
        lhs: Value,
        rhs: Value,
        lhs_operand: &ir::Operand<'db>,
        builder: &mut FunctionBuilder,
    ) -> Option<Value> {
        let is_float = self.is_float_operand(lhs_operand);

        match op {
            ir::BinOp::Add => {
                if is_float {
                    Some(builder.ins().fadd(lhs, rhs))
                } else {
                    Some(builder.ins().iadd(lhs, rhs))
                }
            }
            ir::BinOp::Sub => {
                if is_float {
                    Some(builder.ins().fsub(lhs, rhs))
                } else {
                    Some(builder.ins().isub(lhs, rhs))
                }
            }
            ir::BinOp::Mul => {
                if is_float {
                    Some(builder.ins().fmul(lhs, rhs))
                } else {
                    Some(builder.ins().imul(lhs, rhs))
                }
            }
            ir::BinOp::Div => {
                if is_float {
                    Some(builder.ins().fdiv(lhs, rhs))
                } else if self.is_signed_operand(lhs_operand) {
                    Some(builder.ins().sdiv(lhs, rhs))
                } else {
                    Some(builder.ins().udiv(lhs, rhs))
                }
            }
            ir::BinOp::Rem => {
                if self.is_signed_operand(lhs_operand) {
                    Some(builder.ins().srem(lhs, rhs))
                } else {
                    Some(builder.ins().urem(lhs, rhs))
                }
            }
            ir::BinOp::BitAnd | ir::BinOp::And => Some(builder.ins().band(lhs, rhs)),
            ir::BinOp::BitOr | ir::BinOp::Or => Some(builder.ins().bor(lhs, rhs)),
            ir::BinOp::BitXor => Some(builder.ins().bxor(lhs, rhs)),
            ir::BinOp::Shl => Some(builder.ins().ishl(lhs, rhs)),
            ir::BinOp::Shr => {
                if self.is_signed_operand(lhs_operand) {
                    Some(builder.ins().sshr(lhs, rhs))
                } else {
                    Some(builder.ins().ushr(lhs, rhs))
                }
            }
            ir::BinOp::Eq => Some(builder.ins().icmp(IntCC::Equal, lhs, rhs)),
            ir::BinOp::Ne => Some(builder.ins().icmp(IntCC::NotEqual, lhs, rhs)),
            ir::BinOp::Lt => {
                if self.is_signed_operand(lhs_operand) {
                    Some(builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs))
                } else {
                    Some(builder.ins().icmp(IntCC::UnsignedLessThan, lhs, rhs))
                }
            }
            ir::BinOp::Le => {
                if self.is_signed_operand(lhs_operand) {
                    Some(builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs))
                } else {
                    Some(builder.ins().icmp(IntCC::UnsignedLessThanOrEqual, lhs, rhs))
                }
            }
            ir::BinOp::Gt => {
                if self.is_signed_operand(lhs_operand) {
                    Some(builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs))
                } else {
                    Some(builder.ins().icmp(IntCC::UnsignedGreaterThan, lhs, rhs))
                }
            }
            ir::BinOp::Ge => {
                if self.is_signed_operand(lhs_operand) {
                    Some(builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs))
                } else {
                    Some(builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, lhs, rhs))
                }
            }
        }
    }

    fn lower_unop(
        &self,
        op: ir::UnOp,
        val: Value,
        operand: &ir::Operand<'db>,
        builder: &mut FunctionBuilder,
    ) -> Option<Value> {
        match op {
            ir::UnOp::Neg => {
                if self.is_float_operand(operand) {
                    Some(builder.ins().fneg(val))
                } else {
                    Some(builder.ins().ineg(val))
                }
            }
            ir::UnOp::Not => Some(builder.ins().bnot(val)),
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
                let discr_val = self.lower_operand(discr, builder)?;
                // IR convention: targets[0] = false branch, targets[1] = true branch
                if targets.len() == 2 {
                    let false_block = self.block_map[&targets[0].0];
                    let true_block = self.block_map[&targets[1].0];
                    builder
                        .ins()
                        .brif(discr_val, true_block, &[], false_block, &[]);
                } else {
                    emit_codegen_err(self.db, "switchInt with != 2 targets is not supported");
                    return None;
                }
                Some(())
            }
            ir::Terminator::Call {
                func,
                args,
                destination,
                target,
            } => {
                // Resolve the function
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

                // Lower arguments
                let mut arg_vals = Vec::new();
                for a in args.iter() {
                    arg_vals.push(self.lower_operand(a, builder)?);
                }

                let call = builder.ins().call(func_ref, &arg_vals);

                // Assign result to destination if function returns a value
                let results = builder.inst_results(call);
                if !results.is_empty() {
                    let result_val = results[0];
                    self.assign_to_place(destination, result_val, builder)?;
                }

                // Jump to continuation block
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
            ir::Terminator::Unreachable => {
                builder.ins().trap(TrapCode::user(1).unwrap());
                Some(())
            }
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
