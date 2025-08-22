//! Code generation backend for the Scrap programming language using Cranelift.
//!
//! This crate provides functionality to compile Scrap AST nodes into executable code
//! using the Cranelift code generator.

pub mod error;
pub mod jit;
pub mod object;

pub use error::{CodegenError, CodegenResult};

#[cfg(test)]
mod tests {
    use crate::jit::JitCompiler;

    #[test]
    fn test_jit_creation() {
        let jit = JitCompiler::new();
        assert!(jit.is_ok());
    }
}
