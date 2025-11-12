//! JIT compilation utilities for Scrap programs.

use crate::{CodegenError, CodegenResult};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::FuncId;
use std::collections::HashMap;

/// A JIT compiler for Scrap programs that can execute code in memory.
pub struct JitCompiler {
    /// The underlying JIT module
    module: JITModule,
    /// Function builder context for reuse
    builder_context: FunctionBuilderContext,
    /// Maps function names to their compiled function IDs
    functions: HashMap<String, FuncId>,
}

impl JitCompiler {
    /// Creates a new JIT compiler instance.
    pub fn new() -> Result<Self> {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        let module = JITModule::new(builder);

        Ok(Self {
            module,
            builder_context: FunctionBuilderContext::new(),
            functions: HashMap::new(),
        })
    }

    /// Compiles and loads a function into the JIT.
    // pub fn compile_function(&mut self, func: &FnDef) -> CodegenResult<FuncId> {
    //     // Create the function signature
    //     let mut sig = self.module.make_signature();

    //     // TODO: Add parameters based on function signature
    //     // For now, assume no parameters and return i64
    //     sig.returns.push(AbiParam::new(types::I64));

    //     let func_id = self
    //         .module
    //         .declare_function(&func.ident.name, Linkage::Export, &sig)?;

    //     // Create function context and builder
    //     let mut ctx = codegen::Context::new();
    //     ctx.func.signature = sig;

    //     {
    //         let mut builder = FunctionBuilder::new(&mut ctx.func, &mut self.builder_context);
    //         let entry_block = builder.create_block();
    //         builder.append_block_params_for_function_params(entry_block);
    //         builder.switch_to_block(entry_block);
    //         builder.seal_block(entry_block);

    //         // TODO: Compile function body
    //         // For now, just return a constant
    //         let value = builder.ins().iconst(types::I64, 42);
    //         builder.ins().return_(&[value]);

    //         builder.finalize();
    //     }

    //     // Define the function in the module
    //     self.module.define_function(func_id, &mut ctx)?;

    //     self.functions.insert(func.ident.name.clone(), func_id);
    //     Ok(func_id)
    // }

    /// Finalizes all compiled functions and prepares them for execution.
    pub fn finalize(&mut self) -> CodegenResult<()> {
        self.module.finalize_definitions().map_err(Into::into)
    }

    /// Gets a function pointer for the given function name.
    pub fn get_function<T>(&self, name: &str) -> CodegenResult<*const T> {
        let func_id = self
            .functions
            .get(name)
            .ok_or_else(|| CodegenError::function_not_found(name))?;

        let ptr = self.module.get_finalized_function(*func_id);
        Ok(ptr as *const T)
    }

    /// Executes a function with no parameters that returns an i64.
    pub fn execute_function_i64(&self, name: &str) -> CodegenResult<i64> {
        let func_ptr: *const fn() -> i64 = self.get_function(name)?;

        // SAFETY: We trust that the function was compiled correctly
        let result = unsafe { (*func_ptr)() };
        Ok(result)
    }
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new().expect("Failed to create default JitCompiler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_compiler_creation() {
        let compiler = JitCompiler::new();
        assert!(compiler.is_ok());
    }
}
