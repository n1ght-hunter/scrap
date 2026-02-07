//! CodegenContext — holds the Cranelift module and compilation state.

use cranelift::prelude::*;
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use scrap_ir as ir;
use std::collections::HashMap;
use std::str::FromStr;
use target_lexicon::Triple;

use super::emit_codegen_err;
use super::ResultExt;

/// The main code generation context.
pub struct CodegenContext<'db> {
    pub(crate) db: &'db dyn scrap_shared::Db,
    pub(crate) module: ObjectModule,
    pub(crate) ctx: codegen::Context,
    pub(crate) func_ctx: FunctionBuilderContext,
    /// Maps function name → Cranelift FuncId.
    pub(crate) functions: HashMap<String, FuncId>,
}

impl<'db> CodegenContext<'db> {
    /// Create a new code generation context targeting x86_64-pc-windows-msvc.
    pub fn new(db: &'db dyn scrap_shared::Db) -> Option<Self> {
        let target_triple =
            Triple::from_str("x86_64-pc-windows-msvc").map_err(|e| {
                format!("failed to parse target triple: {e}")
            }).or_emit(db)?;

        let shared_builder = settings::builder();
        let shared_flags = settings::Flags::new(shared_builder);
        let isa = cranelift::codegen::isa::lookup(target_triple)
            .map_err(|e| format!("ISA lookup failed: {e}"))
            .or_emit(db)?
            .finish(shared_flags)
            .map_err(|e| format!("ISA finish failed: {e}"))
            .or_emit(db)?;

        let object_builder = ObjectBuilder::new(
            isa,
            "scrap_program",
            cranelift_module::default_libcall_names(),
        )
        .map_err(|e| format!("ObjectBuilder failed: {e}"))
        .or_emit(db)?;

        let module = ObjectModule::new(object_builder);

        Some(Self {
            db,
            module,
            ctx: codegen::Context::new(),
            func_ctx: FunctionBuilderContext::new(),
            functions: HashMap::new(),
        })
    }

    /// Compile an entire IR module (declare then define).
    pub fn compile_module(&mut self, module: ir::Module<'db>) -> Option<()> {
        self.declare_items(module)?;
        self.define_functions(module)?;
        Some(())
    }

    /// Generate the `_start` entry point that calls `main`.
    pub fn generate_start(&mut self) -> Option<()> {
        let main_func_id = match self.functions.get("main").copied() {
            Some(id) => id,
            None => {
                emit_codegen_err(self.db, "function 'main' not found");
                return None;
            }
        };

        // Declare _start: no params, no returns
        let mut start_sig = self.module.make_signature();
        start_sig.call_conv = self.module.target_config().default_call_conv;

        let start_func_id = self
            .module
            .declare_function("_start", Linkage::Export, &start_sig)
            .or_emit(self.db)?;

        self.ctx.func.signature = start_sig;

        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.func_ctx);
            let entry_block = builder.create_block();
            builder.switch_to_block(entry_block);

            // Call main
            let main_ref =
                self.module.declare_func_in_func(main_func_id, builder.func);
            builder.ins().call(main_ref, &[]);

            // If main returns (it shouldn't if it returns !), trap
            builder.ins().trap(TrapCode::user(1).unwrap());

            builder.seal_all_blocks();
            builder.finalize();
        }

        self.module
            .define_function(start_func_id, &mut self.ctx)
            .or_emit(self.db)?;
        self.module.clear_context(&mut self.ctx);

        Some(())
    }

    /// Finalize the module and return the object file bytes.
    pub fn finalize(self) -> Option<Vec<u8>> {
        let object_product = self.module.finish();
        object_product
            .emit()
            .map_err(|e| format!("failed to emit object file: {e}"))
            .or_emit(self.db)
    }
}
