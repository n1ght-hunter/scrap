//! Object file compilation utilities for Scrap programs.

use crate::{CodegenError, CodegenResult};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use scrap_parser::parser::{fndef::FnDef, item::{Item, ItemKind}};
use std::collections::HashMap;

/// A compiler that generates object files from Scrap programs.
pub struct ObjectCompiler {
    /// The underlying object module
    module: ObjectModule,
    /// Function builder context for reuse
    builder_context: FunctionBuilderContext,
    /// Maps function names to their compiled function IDs
    functions: HashMap<String, FuncId>,
}

impl ObjectCompiler {
    /// Creates a new object compiler instance.
    pub fn new() -> Result<Self> {
        let isa = cranelift_native::builder()
            .map_err(|msg| anyhow::anyhow!("Failed to create ISA builder: {}", msg))?
            .finish(settings::Flags::new(settings::builder()))?;

        let builder = ObjectBuilder::new(
            isa,
            "scrap_program",
            cranelift_module::default_libcall_names(),
        )?;
        let module = ObjectModule::new(builder);

        Ok(Self {
            module,
            builder_context: FunctionBuilderContext::new(),
            functions: HashMap::new(),
        })
    }

    /// Compiles a function to object code.
    pub fn compile_function(&mut self, func: &FnDef) -> CodegenResult<FuncId> {
        // Create the function signature
        let mut sig = self.module.make_signature();
        
        // TODO: Add parameters based on function signature
        // For now, assume no parameters and return i64
        sig.returns.push(AbiParam::new(types::I64));

        let func_id = self.module.declare_function(&func.ident.name, Linkage::Export, &sig)?;

        // Create function context and builder
        let mut ctx = codegen::Context::new();
        ctx.func.signature = sig;

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut self.builder_context);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            // TODO: Compile function body
            // For now, just return a constant
            let value = builder.ins().iconst(types::I64, 42);
            builder.ins().return_(&[value]);

            builder.finalize();
        }

        // Define the function in the module
        self.module.define_function(func_id, &mut ctx)?;

        self.functions.insert(func.ident.name.clone(), func_id);
        Ok(func_id)
    }

    /// Compiles a complete program to object code.
    pub fn compile_program(&mut self, program: &[Item]) -> CodegenResult<()> {
        for item in program {
            match &item.kind {
                ItemKind::Fn(func) => {
                    self.compile_function(func)?;
                }
                ItemKind::Enum(_) => {
                    // TODO: Implement enum compilation
                }
                ItemKind::Struct(_) => {
                    // TODO: Implement struct compilation
                }
            }
        }
        Ok(())
    }

    /// Finalizes the object file and returns the compiled bytes.
    pub fn finalize(self) -> CodegenResult<Vec<u8>> {
        let object_product = self.module.finish();
        object_product.emit()
            .map_err(|e| CodegenError::generic(format!("Failed to emit object file: {e}")))
    }

    /// Writes the compiled object file to disk.
    pub fn write_object_file(self, path: &std::path::Path) -> CodegenResult<()> {
        let bytes = self.finalize()?;
        std::fs::write(path, bytes)
            .map_err(|e| CodegenError::generic(format!("Failed to write object file: {e}")))?;
        Ok(())
    }
}

impl Default for ObjectCompiler {
    fn default() -> Self {
        Self::new().expect("Failed to create default ObjectCompiler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_compiler_creation() {
        let compiler = ObjectCompiler::new();
        assert!(compiler.is_ok());
    }
}
