pub mod ir;
pub mod ir_builder;

// Re-export main types and functions for convenience
pub use ir::{
    AggregateKind, BasicBlock, BasicBlockId, BinOp, Body, Can, Constant, Enum, EnumVariant,
    Function, FunctionId, Items, LocalDecl, LocalId, Module, Operand, Place, Rvalue, Signature,
    Statement, StatementKind, Struct, Terminator, Ty, TypeId, UnOp,
};
pub use ir_builder::{BuilderError, LoweredIr, lower_to_ir};
