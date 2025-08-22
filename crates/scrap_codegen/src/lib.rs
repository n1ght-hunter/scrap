//! Code generation backend for the Scrap programming language using Cranelift.
//!
//! This crate provides functionality to compile Scrap AST nodes into executable code
//! using the Cranelift code generator.

use anyhow::Result;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::FuncId;
use scrap_parser::parser::item::{Item, ItemKind};
use std::collections::HashMap;

pub mod error;
pub mod jit;
pub mod object;

pub use error::{CodegenError, CodegenResult};

/// The main code generator for Scrap programs.
pub struct CodeGenerator {
    /// The Cranelift module used for code generation
    module: JITModule,
    /// The function builder context
    builder_context: FunctionBuilderContext,
    /// Maps function names to their IDs in the module
    function_map: HashMap<String, FuncId>,
}

impl CodeGenerator {
    /// Creates a new code generator instance.
    pub fn new() -> Result<Self> {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        let module = JITModule::new(builder);
        
        Ok(Self {
            module,
            builder_context: FunctionBuilderContext::new(),
            function_map: HashMap::new(),
        })
    }

    /// Compiles a Scrap program and returns the JIT module.
    pub fn compile_program(&mut self, program: &[Item]) -> Result<()> {
        for item in program {
            self.compile_item(item)?;
        }
        
        self.module.finalize_definitions()?;
        Ok(())
    }

    /// Compiles a top-level program item.
    fn compile_item(&mut self, item: &Item) -> Result<()> {
        match &item.kind {
            ItemKind::Fn(func) => self.compile_function(func),
            ItemKind::Enum(_) => {
                // TODO: Implement enum compilation
                Ok(())
            }
            ItemKind::Struct(_) => {
                // TODO: Implement struct compilation
                Ok(())
            }
        }
    }

    /// Compiles a function definition.
    fn compile_function(&mut self, _func: &scrap_parser::parser::fndef::FnDef) -> Result<()> {
        // TODO: Implement function compilation
        // This is a placeholder for the actual implementation
        todo!("Function compilation not yet implemented")
    }

    /// Finalizes the module and prepares it for execution.
    pub fn finalize(&mut self) -> Result<()> {
        self.module.finalize_definitions()
            .map_err(|e| anyhow::anyhow!("Failed to finalize module: {}", e))
    }

    /// Gets a pointer to a compiled function.
    pub fn get_function_ptr(&self, name: &str) -> Option<*const u8> {
        let func_id = self.function_map.get(name)?;
        Some(self.module.get_finalized_function(*func_id))
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default CodeGenerator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_creation() {
        let codegen = CodeGenerator::new();
        assert!(codegen.is_ok());
    }
}
