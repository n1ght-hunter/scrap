//! Error types for the Scrap code generator.

use cranelift::codegen;
use thiserror::Error;

/// Errors that can occur during code generation.
#[derive(Error, Debug)]
pub enum CodegenError {
    /// Cranelift module error
    #[error("Cranelift module error: {0}")]
    Module(Box<cranelift_module::ModuleError>),

    /// Cranelift codegen error
    #[error("Cranelift codegen error: {0}")]
    Codegen(#[from] codegen::CodegenError),

    /// Function not found
    #[error("Function '{name}' not found")]
    FunctionNotFound { name: String },

    /// Variable not found
    #[error("Variable '{name}' not found")]
    VariableNotFound { name: String },

    /// Type mismatch
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    /// Unsupported operation
    #[error("Unsupported operation: {operation}")]
    UnsupportedOperation { operation: String },

    /// Invalid function signature
    #[error("Invalid function signature for '{name}'")]
    InvalidFunctionSignature { name: String },

    /// Generic codegen error
    #[error("Code generation error: {message}")]
    Generic { message: String },
}

impl CodegenError {
    /// Creates a new generic error with the given message.
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Creates a new function not found error.
    pub fn function_not_found(name: impl Into<String>) -> Self {
        Self::FunctionNotFound { name: name.into() }
    }

    /// Creates a new variable not found error.
    pub fn variable_not_found(name: impl Into<String>) -> Self {
        Self::VariableNotFound { name: name.into() }
    }

    /// Creates a new type mismatch error.
    pub fn type_mismatch(expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
        }
    }

    /// Creates a new unsupported operation error.
    pub fn unsupported_operation(operation: impl Into<String>) -> Self {
        Self::UnsupportedOperation {
            operation: operation.into(),
        }
    }

    /// Creates a new invalid function signature error.
    pub fn invalid_function_signature(name: impl Into<String>) -> Self {
        Self::InvalidFunctionSignature { name: name.into() }
    }
}

/// A result type that uses `CodegenError` as the error type.
pub type CodegenResult<T> = Result<T, CodegenError>;

impl From<cranelift_module::ModuleError> for CodegenError {
    fn from(err: cranelift_module::ModuleError) -> Self {
        CodegenError::Module(Box::new(err))
    }
}
