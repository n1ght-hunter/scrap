//! Code generation backend for the Scrap programming language using Cranelift.
//!
//! This crate provides functionality to compile Scrap IR into executable code
//! using the Cranelift code generator.

pub mod codegen;
pub mod error;
pub mod object;

pub use error::{CodegenError, CodegenResult};

use cranelift::prelude::types;

/// Map a Rust scalar type display name (e.g. `i32`, `usize`, `bool`) to its
/// Cranelift type, or `None` for non-scalar fields (aggregates / pointers),
/// which are addressed by `base + offset` rather than loaded as a value.
fn scalar_cl_ty(display: &str) -> Option<types::Type> {
    Some(match display {
        "i8" | "u8" | "bool" => types::I8,
        "i16" | "u16" => types::I16,
        "i32" | "u32" => types::I32,
        "i64" | "u64" | "isize" | "usize" => types::I64,
        "f32" => types::F32,
        "f64" => types::F64,
        _ => return None,
    })
}

/// Build a [`codegen::context::RustLayout`] from interop metadata: the type's
/// total size/align and each field's `(byte offset, display type name)` in
/// declaration order. Scalar fields get a Cranelift type; everything else is
/// addressed by offset. Keeping this in the codegen crate keeps the
/// display → Cranelift-type mapping next to the types it produces.
pub fn rust_layout_from_metadata(
    size: u64,
    align: u64,
    fields: &[(u64, &str)],
) -> codegen::context::RustLayout {
    codegen::context::RustLayout {
        size: size as u32,
        align: align as u32,
        fields: fields
            .iter()
            .map(|(offset, display)| codegen::context::RustFieldLayout {
                offset: *offset as u32,
                cl_ty: scalar_cl_ty(display),
            })
            .collect(),
    }
}

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
