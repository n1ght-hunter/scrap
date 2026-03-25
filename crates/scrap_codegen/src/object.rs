//! Object file compilation utilities for Scrap programs.

use crate::{CodegenError, CodegenResult};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_object::{ObjectBuilder, ObjectModule};

/// A compiler that generates object files from Scrap programs.
pub struct ObjectCompiler {
    /// The underlying object module
    module: ObjectModule,
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

        Ok(Self { module })
    }

    /// Finalizes the object file and returns the compiled bytes.
    pub fn finalize(self) -> CodegenResult<Vec<u8>> {
        let object_product = self.module.finish();
        object_product
            .emit()
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
