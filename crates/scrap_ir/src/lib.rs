pub mod ir;
pub mod pretty;

// Re-export main types for convenience
pub use ir::{
    AggregateKind, BasicBlock, BasicBlockId, BinOp, Body, Can, Constant, Enum, EnumVariant,
    ExternFn, Function, FunctionId, Items, LocalDecl, LocalId, Module, Operand, Place, Rvalue,
    Signature, Statement, StatementKind, Struct, Terminator, Ty, TypeId, UnOp,
};

pub use pretty::print_can;
