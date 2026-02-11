//! Function declaration and definition (two-pass compilation).

use cranelift::codegen::ir::StackSlot;
use cranelift::prelude::*;
use cranelift_module::{Linkage, Module};
use scrap_ir as ir;
use std::collections::HashSet;

use super::context::CodegenContext;
use super::cranelift_ir::FuncTranslator;
use super::emit_codegen_err;
use super::ty::{build_cl_signature, build_cl_signature_with_layouts, ir_ty_to_cl, ir_ty_to_cl_required};
use super::ResultExt;

/// Scan a function body for locals that are targets of `Rvalue::Ref`.
/// These locals need to be stack-spilled so we can take their address.
fn collect_referenced_locals<'db>(db: &'db dyn scrap_shared::Db, body: ir::Body<'db>) -> HashSet<usize> {
    let mut referenced = HashSet::new();
    for block in body.blocks(db) {
        for stmt in block.statements(db) {
            if let ir::StatementKind::Assign(_, ir::Rvalue::Ref(_, ref place)) = stmt.kind(db) {
                if let ir::Place::Local(id) = place {
                    referenced.insert(id.0);
                }
            }
        }
    }
    referenced
}

impl<'db> CodegenContext<'db> {
    /// Pass 1: Declare all functions and extern functions in the module.
    /// Collects struct/enum layouts first so function signatures can reference ADT types.
    pub fn declare_items(&mut self, module: ir::Module<'db>) -> Option<()> {
        // First sub-pass: collect struct and enum layouts
        for item in module.items(self.db) {
            match item {
                ir::Items::Struct(s) => {
                    let name = s.name(self.db).text(self.db).to_string();
                    let field_tys: Vec<ir::Ty<'db>> =
                        s.fields(self.db).iter().map(|(_, ty)| ty.clone()).collect();
                    self.struct_layouts.insert(name, field_tys);
                }
                ir::Items::Enum(e) => {
                    let name = e.name(self.db).text(self.db).to_string();
                    let mut variant_layouts = Vec::new();
                    for variant in e.variants(self.db) {
                        let field_tys: Vec<ir::Ty<'db>> = match variant {
                            ir::EnumVariant::Unit(_) => vec![],
                            ir::EnumVariant::Tuple(_, tys) => tys.clone(),
                            ir::EnumVariant::Struct(_, fields) => {
                                fields.iter().map(|(_, ty)| ty.clone()).collect()
                            }
                        };
                        variant_layouts.push(field_tys);
                    }
                    self.enum_layouts.insert(name, variant_layouts);
                }
                _ => {}
            }
        }

        // Second sub-pass: declare functions (now struct layouts are available)
        for item in module.items(self.db) {
            match item {
                ir::Items::Function(func) => {
                    let sig = func.signature(self.db);
                    let name = sig.name(self.db).text(self.db);
                    let cl_sig = build_cl_signature_with_layouts(
                        &self.module, sig, self.db, &self.struct_layouts,
                    )?;
                    let func_id = self
                        .module
                        .declare_function(name, Linkage::Local, &cl_sig)
                        .or_emit(self.db)?;
                    self.functions.insert(name.to_string(), func_id);
                }
                ir::Items::ExternFunction(ext) => {
                    let sig = ext.signature(self.db);
                    let name = sig.name(self.db).text(self.db);
                    let cl_sig = build_cl_signature(&self.module, sig, self.db)?;
                    let func_id = self
                        .module
                        .declare_function(name, Linkage::Import, &cl_sig)
                        .or_emit(self.db)?;
                    self.functions.insert(name.to_string(), func_id);
                }
                _ => {}
            }
        }
        Some(())
    }

    /// Pass 2: Define all function bodies.
    pub fn define_functions(&mut self, module: ir::Module<'db>) -> Option<()> {
        for item in module.items(self.db) {
            if let ir::Items::Function(func) = item {
                self.define_function(*func)?;
            }
        }
        Some(())
    }

    /// Define a single function's body.
    fn define_function(&mut self, func: ir::Function<'db>) -> Option<()> {
        let sig = func.signature(self.db);
        let name = sig.name(self.db).text(self.db).to_string();
        let body = func.body(self.db);

        let func_id = match self.functions.get(&name).copied() {
            Some(id) => id,
            None => {
                emit_codegen_err(self.db, format!("function '{name}' not found"));
                return None;
            }
        };

        // Set up the function context
        let cl_sig = build_cl_signature_with_layouts(
            &self.module, sig, self.db, &self.struct_layouts,
        )?;
        self.ctx.func.signature = cl_sig;

        let ret_ty = sig.return_ty(self.db);
        let returns_void = matches!(ret_ty, ir::Ty::Void | ir::Ty::Never);

        // Build the function body
        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.func_ctx);

            let local_decls = body.local_decls(self.db);
            let _param_count = body.param_count(self.db);
            let ir_blocks = body.blocks(self.db);

            // Create Cranelift blocks for each IR basic block
            let mut block_map = std::collections::HashMap::new();
            for i in 0..ir_blocks.len() {
                let block = builder.create_block();
                block_map.insert(i, block);
            }

            // Scan for locals that need stack spilling (targets of Rvalue::Ref)
            let referenced_locals = collect_referenced_locals(self.db, body);

            // Declare all local variables
            let mut variables = std::collections::HashMap::new();
            let mut tuple_variables = std::collections::HashMap::new();
            let mut stack_slots: std::collections::HashMap<usize, (StackSlot, types::Type)> =
                std::collections::HashMap::new();
            // Enum-specific: discriminant variable per enum local
            let mut enum_discriminants: std::collections::HashMap<usize, Variable> =
                std::collections::HashMap::new();
            // Enum-specific: (local_id, variant_idx, field_idx) → Variable
            let mut enum_variant_variables: std::collections::HashMap<(usize, usize, usize), Variable> =
                std::collections::HashMap::new();
            for (i, decl) in local_decls.iter().enumerate() {
                let ty = decl.ty(self.db);
                if let ir::Ty::Tuple(ref fields) = ty {
                    // Tuple locals are decomposed into per-field sub-variables
                    for (field_idx, field_ty) in fields.iter().enumerate() {
                        let cl_ty = ir_ty_to_cl_required(self.db, field_ty)?;
                        let var = builder.declare_var(cl_ty);
                        tuple_variables.insert((i, field_idx), var);
                    }
                } else if let ir::Ty::Adt(type_id) = &ty {
                    let adt_name = type_id.name(self.db);
                    if let Some(variant_layouts) = self.enum_layouts.get(adt_name.as_str()) {
                        // Enum local: discriminant (i64) + per-variant field variables
                        let disc_var = builder.declare_var(types::I64);
                        enum_discriminants.insert(i, disc_var);
                        for (variant_idx, field_tys) in variant_layouts.iter().enumerate() {
                            for (field_idx, field_ty) in field_tys.iter().enumerate() {
                                let cl_ty = ir_ty_to_cl_required(self.db, field_ty)?;
                                let var = builder.declare_var(cl_ty);
                                enum_variant_variables.insert((i, variant_idx, field_idx), var);
                            }
                        }
                    } else if let Some(field_tys) = self.struct_layouts.get(adt_name.as_str()) {
                        // Struct locals are decomposed into per-field sub-variables (like tuples)
                        for (field_idx, field_ty) in field_tys.iter().enumerate() {
                            let cl_ty = ir_ty_to_cl_required(self.db, field_ty)?;
                            let var = builder.declare_var(cl_ty);
                            tuple_variables.insert((i, field_idx), var);
                        }
                    } else {
                        emit_codegen_err(
                            self.db,
                            format!("ADT '{}' layout not found", adt_name),
                        );
                        return None;
                    }
                } else if referenced_locals.contains(&i) {
                    // Stack-spill: this local is referenced via & or &mut
                    let cl_ty = ir_ty_to_cl_required(self.db, &ty)?;
                    let slot = builder.create_sized_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        cl_ty.bytes(),
                        0,
                    ));
                    stack_slots.insert(i, (slot, cl_ty));
                } else if let Some(cl_ty) = ir_ty_to_cl(self.db, &ty)? {
                    // Regular scalar local
                    let var = builder.declare_var(cl_ty);
                    variables.insert(i, var);
                }
            }

            // Mark *T locals for Cranelift stack maps (GC root tracking at safepoints)
            for (i, decl) in local_decls.iter().enumerate() {
                if matches!(decl.ty(self.db), ir::Ty::Ptr(_)) {
                    if let Some(var) = variables.get(&i) {
                        builder.declare_var_needs_stack_map(*var);
                    }
                }
            }

            // Set up the entry block
            let entry_block = block_map[&0];
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);

            // Write function parameters to their variables (_1.._param_count)
            // Struct params are expanded into multiple Cranelift params (one per field),
            // so we walk the IR params and consume Cranelift params accordingly.
            let params = builder.block_params(entry_block).to_vec();
            let mut param_idx = 0;
            for ir_param_idx in 0.._param_count {
                let local_idx = ir_param_idx + 1; // _0 is return place, _1.. are params
                let decl_ty = local_decls[local_idx].ty(self.db);
                if let ir::Ty::Adt(type_id) = &decl_ty {
                    let adt_name = type_id.name(self.db);
                    if let Some(field_tys) = self.struct_layouts.get(adt_name.as_str()) {
                        // Struct param: each field is a separate Cranelift param
                        for (field_idx, _) in field_tys.iter().enumerate() {
                            if let Some(var) = tuple_variables.get(&(local_idx, field_idx)) {
                                builder.def_var(*var, params[param_idx]);
                            }
                            param_idx += 1;
                        }
                    }
                } else if let Some((slot, _)) = stack_slots.get(&local_idx) {
                    // Stack-spilled param: store incoming value to stack slot
                    builder.ins().stack_store(params[param_idx], *slot, 0);
                    param_idx += 1;
                } else if let Some(var) = variables.get(&local_idx) {
                    builder.def_var(*var, params[param_idx]);
                    param_idx += 1;
                } else {
                    param_idx += 1;
                }
            }

            // Emit yield point for cooperative scheduling (no-op if not in a coroutine)
            if let Some(&yield_id) = self.functions.get("__scrap_yield") {
                let yield_ref =
                    self.module.declare_func_in_func(yield_id, builder.func);
                builder.ins().call(yield_ref, &[]);
            }

            // Create the translator (holds only shared/immutable references)
            let data_counter = std::cell::Cell::new(self.data_id_counter);
            let gc_shapes_cell = std::cell::RefCell::new(std::collections::HashMap::new());
            // Pre-populate from context's gc_shapes
            for (k, v) in &self.gc_shapes {
                gc_shapes_cell.borrow_mut().insert(k.clone(), *v);
            }
            let translator = FuncTranslator {
                db: self.db,
                variables: &variables,
                tuple_variables: &tuple_variables,
                block_map: &block_map,
                functions: &self.functions,
                local_decls,
                returns_void,
                next_data_id: &data_counter,
                gc_shapes: &gc_shapes_cell,
                struct_layouts: &self.struct_layouts,
                enum_layouts: &self.enum_layouts,
                enum_discriminants: &enum_discriminants,
                enum_variant_variables: &enum_variant_variables,
                stack_slots: &stack_slots,
            };

            // Lower each basic block
            for (block_idx, ir_block) in ir_blocks.iter().enumerate() {
                let cl_block = block_map[&block_idx];

                // Switch to block (entry block already switched above)
                if block_idx != 0 {
                    builder.switch_to_block(cl_block);
                }

                // Lower statements
                for stmt in ir_block.statements(self.db) {
                    translator.lower_statement(*stmt, &mut builder, &mut self.module)?;
                }

                // Lower terminator
                let term = ir_block.terminator(self.db);
                translator.lower_terminator(&term, &mut builder, &mut self.module)?;
            }

            builder.seal_all_blocks();
            builder.finalize();

            // Persist the data counter back to the context
            self.data_id_counter = data_counter.get();

            // Merge any new gc_shapes back into the context
            for (k, v) in gc_shapes_cell.into_inner() {
                self.gc_shapes.entry(k).or_insert(v);
            }
        }

        // Define the function in the module
        self.module
            .define_function(func_id, &mut self.ctx)
            .or_emit(self.db)?;

        self.collect_stack_maps(func_id);
        self.collect_unwind_info(func_id);
        self.module.clear_context(&mut self.ctx);

        Some(())
    }
}
