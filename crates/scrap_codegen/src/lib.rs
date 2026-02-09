//! Code generation backend for the Scrap programming language using Cranelift.
//!
//! This crate provides functionality to compile Scrap IR into executable code
//! using the Cranelift code generator.

pub mod codegen;
pub mod error;
pub mod object;

pub use error::{CodegenError, CodegenResult};

/// Compile an IR compilation unit to an object file.
///
/// This is the main entry point for code generation. It:
/// 1. Declares all functions (local + imported)
/// 2. Defines all function bodies
/// 3. Generates a `_start` entry point that calls `main`
/// 4. Returns the raw COFF object bytes, or `None` if errors were emitted
///
/// Errors are emitted through the database's diagnostic context (`db.dcx()`).
#[salsa::tracked]
pub fn compile_to_object<'db>(
    db: &'db dyn scrap_shared::Db,
    ir: scrap_ir::Can<'db>,
) -> Option<Vec<u8>> {
    let mut ctx = codegen::CodegenContext::new(db)?;

    for module in ir.modules(db) {
        ctx.compile_module(*module)?;
    }

    ctx.generate_start()?;
    ctx.finalize()
}
