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
    target: target_lexicon::Triple,
) -> Option<Vec<u8>> {
    let mut ctx = codegen::CodegenContext::new(db, &target)?;

    for module in ir.modules(db) {
        ctx.compile_module(*module)?;
    }

    ctx.generate_start()?;
    ctx.finalize()
}

/// Like [`compile_to_object`] but with native Rust interop data installed:
/// `rust_fn_symbols` maps `extern "Rust"` names to their mangled symbols and
/// `rust_layouts` maps Rust type paths to their mirrored layouts (both from
/// interop metadata). Not salsa-tracked — the maps come from a side build.
pub fn compile_to_object_interop<'db>(
    db: &'db dyn scrap_shared::Db,
    ir: scrap_ir::Can<'db>,
    target: target_lexicon::Triple,
    rust_fn_symbols: std::collections::HashMap<String, String>,
    rust_layouts: std::collections::HashMap<String, codegen::context::RustLayout>,
) -> Option<Vec<u8>> {
    let mut ctx = codegen::CodegenContext::new(db, &target)?;
    ctx.set_rust_fn_symbols(rust_fn_symbols);
    ctx.set_rust_layouts(rust_layouts);

    for module in ir.modules(db) {
        ctx.compile_module(*module)?;
    }

    ctx.generate_start()?;
    ctx.finalize()
}
