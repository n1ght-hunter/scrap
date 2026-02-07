//! Code generation from Scrap IR to native object files via Cranelift.

pub mod context;
pub mod cranelift_ir;
pub mod function;
pub mod ty;

pub use context::CodegenContext;

use scrap_diagnostics::Level;

/// Emit a code generation error diagnostic.
pub(crate) fn emit_codegen_err(db: &dyn scrap_shared::Db, msg: impl std::fmt::Display) {
    db.dcx().emit_err(
        Level::ERROR
            .primary_title("code generation error")
            .element(Level::NOTE.message(msg.to_string())),
    );
}

/// Extension trait to convert `Result<T, E>` into `Option<T>` with diagnostic emission.
pub(crate) trait ResultExt<T> {
    fn or_emit(self, db: &dyn scrap_shared::Db) -> Option<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn or_emit(self, db: &dyn scrap_shared::Db) -> Option<T> {
        self.map_err(|e| emit_codegen_err(db, e)).ok()
    }
}
