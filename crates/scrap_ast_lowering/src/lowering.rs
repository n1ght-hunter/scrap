//! Lowering implementations for AST to IR conversion

pub mod expr;
pub mod module;
pub mod ty;

pub use module::{lower_body, lower_function, lower_module, lower_signature};
pub use ty::lower_type;
