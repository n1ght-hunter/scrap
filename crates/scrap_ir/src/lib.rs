pub mod ir;

// Re-export main types for convenience
pub use ir::{
    AggregateKind, BasicBlock, BasicBlockId, BinOp, Body, Can, Constant, Enum, EnumVariant,
    Function, FunctionId, Items, LocalDecl, LocalId, Module, Operand, Place, Rvalue, Signature,
    Statement, StatementKind, Struct, Terminator, Ty, TypeId, UnOp,
};
