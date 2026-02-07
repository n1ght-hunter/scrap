pub mod ir;
pub mod pretty;

// Re-export main types for convenience
pub use ir::{
    AggregateKind, AssertMessage, BasicBlock, BasicBlockId, Body, Can, Constant, Enum,
    EnumVariant, ExternFn, Function, FunctionId, IntrinsicOp, Items, LocalDecl, LocalId, Module,
    Operand, Place, Rvalue, Signature, Statement, StatementKind, Struct, Terminator, Ty, TypeId,
    UnwindAction,
};

pub use pretty::print_can;
