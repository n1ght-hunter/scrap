//! Function declaration and definition (two-pass compilation).

use cranelift::prelude::*;
use cranelift_module::{Linkage, Module};
use scrap_ir as ir;

use super::context::CodegenContext;
use super::cranelift_ir::FuncTranslator;
use super::emit_codegen_err;
use super::ty::{build_cl_signature, ir_ty_to_cl, ir_ty_to_cl_required};
use super::ResultExt;

impl<'db> CodegenContext<'db> {
    /// Pass 1: Declare all functions and extern functions in the module.
    pub fn declare_items(&mut self, module: ir::Module<'db>) -> Option<()> {
        for item in module.items(self.db) {
            match item {
                ir::Items::Function(func) => {
                    let sig = func.signature(self.db);
                    let name = sig.name(self.db).text(self.db);
                    let cl_sig = build_cl_signature(&self.module, sig, self.db)?;
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
                ir::Items::Struct(_) | ir::Items::Enum(_) => {
                    // Skip type definitions for now
                }
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
        let cl_sig = build_cl_signature(&self.module, sig, self.db)?;
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

            // Declare all local variables
            let mut variables = std::collections::HashMap::new();
            let mut tuple_variables = std::collections::HashMap::new();
            for (i, decl) in local_decls.iter().enumerate() {
                let ty = decl.ty(self.db);
                if let ir::Ty::Tuple(ref fields) = ty {
                    // Tuple locals are decomposed into per-field sub-variables
                    for (field_idx, field_ty) in fields.iter().enumerate() {
                        let cl_ty = ir_ty_to_cl_required(self.db, field_ty)?;
                        let var = builder.declare_var(cl_ty);
                        tuple_variables.insert((i, field_idx), var);
                    }
                } else if let Some(cl_ty) = ir_ty_to_cl(self.db, &ty)? {
                    // Regular scalar local
                    let var = builder.declare_var(cl_ty);
                    variables.insert(i, var);
                }
            }

            // Set up the entry block
            let entry_block = block_map[&0];
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);

            // Write function parameters to their variables (_1.._param_count)
            let params = builder.block_params(entry_block).to_vec();
            for (i, param_val) in params.iter().enumerate() {
                let local_idx = i + 1; // _0 is return place, _1.. are params
                if let Some(var) = variables.get(&local_idx) {
                    builder.def_var(*var, *param_val);
                }
            }

            // Create the translator (holds only shared/immutable references)
            let data_counter = std::cell::Cell::new(0);
            let translator = FuncTranslator {
                db: self.db,
                variables: &variables,
                tuple_variables: &tuple_variables,
                block_map: &block_map,
                functions: &self.functions,
                local_decls,
                returns_void,
                next_data_id: &data_counter,
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
        }

        // Define the function in the module
        self.module
            .define_function(func_id, &mut self.ctx)
            .or_emit(self.db)?;
        self.module.clear_context(&mut self.ctx);

        Some(())
    }
}
