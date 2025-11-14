use scrap_span::Symbol;
use std::marker::PhantomData;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct BasicBlockId(pub usize);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct LocalId(pub usize);

#[salsa::interned(debug, persist)]
pub struct FunctionId<'db> {
    #[returns(ref)]
    pub text: String,
}

/// A unique, program-wide identifier for a user-defined type (struct or enum).
#[salsa::interned(debug, persist)]
pub struct TypeId<'db> {
    #[returns(ref)]
    pub name: String,
}

#[salsa::tracked(debug, persist)]
/// A collection of modules forming a single compilation unit.
pub struct Can<'db> {
    #[tracked]
    #[returns(ref)]
    pub modules: Vec<Module<'db>>,
}

#[salsa::tracked(debug, persist)]
/// A module containing a list of items (functions, structs, enums, etc.) in a single namespace.
pub struct Module<'db> {
    pub path: Symbol<'db>,
    #[tracked]
    #[returns(ref)]
    pub items: Vec<Items<'db>>,
}

/// An item in a module: function, struct, enum, etc.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Items<'db> {
    Function(Function<'db>),
    Struct(Struct<'db>),
    Enum(Enum<'db>),
}

#[salsa::tracked(debug, persist)]
/// The MIR for a struct
pub struct Struct<'db> {
    /// The name of the struct.
    pub name: Symbol<'db>,
    /// The fields of the struct.
    #[tracked]
    #[returns(ref)]
    pub fields: Vec<(Symbol<'db>, Ty<'db>)>,
}

#[salsa::tracked(debug, persist)]
/// The MIR for an enum
pub struct Enum<'db> {
    /// The name of the enum.
    pub name: Symbol<'db>,
    /// The variants of the enum.
    #[tracked]
    #[returns(ref)]
    pub variants: Vec<EnumVariant<'db>>,
}

/// An enum variant can be a unit, tuple, or struct variant.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum EnumVariant<'db> {
    /// A unit variant with no fields.
    Unit(Symbol<'db>),
    /// A tuple variant with unnamed fields.
    Tuple(Symbol<'db>, Vec<Ty<'db>>),
    /// A struct variant with named fields.
    Struct(Symbol<'db>, Vec<(Symbol<'db>, Ty<'db>)>),
}

#[salsa::tracked(debug, persist)]
/// The MIR for a function
pub struct Function<'db> {
    /// The signature of the function.
    pub signature: Signature<'db>,
    /// The body of the function.
    pub body: Body<'db>,
}

#[salsa::tracked(debug, persist)]
pub struct Signature<'db> {
    /// The name of the function.
    pub name: Symbol<'db>,
    /// The parameters of the function.
    #[tracked]
    #[returns(ref)]
    pub params: Vec<(Symbol<'db>, Ty<'db>)>,
    /// The return type of the function.
    pub return_ty: Option<Ty<'db>>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Ty<'db> {
    /// A primitive boolean type.
    Bool,
    /// A primitive integer type (you would add size/signedness).
    Int,
    /// A primitive string type.
    Str,
    /// A user-defined struct or enum, referenced by its unique ID.
    Adt(TypeId<'db>),
    /// Represents a type that never returns a value, like a function that always panics.
    Never,
    /// Represents a type that is not yet known or determined.
    Infer,
}

#[salsa::tracked(debug, persist)]
/// The MIR for a single function, represented as a Control Flow Graph (CFG).
pub struct Body<'db> {
    #[tracked]
    #[returns(ref)]
    pub blocks: Vec<BasicBlock<'db>>,
    #[tracked]
    #[returns(ref)]
    pub local_decls: Vec<LocalDecl<'db>>,
}

#[salsa::tracked(debug, persist)]
/// A Basic Block: a sequence of statements with a single entry and a single exit.
pub struct BasicBlock<'db> {
    #[tracked]
    #[returns(ref)]
    pub statements: Vec<Statement<'db>>,
    pub terminator: Terminator<'db>,
}

#[salsa::tracked(debug, persist)]
/// Declaration for a local variable, argument, or temporary.
pub struct LocalDecl<'db> {
    pub name: Option<Symbol<'db>>,
    pub ty: Ty<'db>,
}

/// Terminators are instructions that end a basic block and transfer control.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Terminator<'db> {
    Goto {
        target: BasicBlockId,
    },
    SwitchInt {
        discr: Operand<'db>,
        targets: Vec<BasicBlockId>,
    },
    Return,
    Call {
        func: Operand<'db>,
        args: Vec<Operand<'db>>,
        destination: Place<'db>,
        target: BasicBlockId,
    },
    Unreachable,
}

impl<'db> Default for Terminator<'db> {
    fn default() -> Self {
        Self::Unreachable
    }
}

#[salsa::tracked(debug, persist)]
/// A statement is a simple, non-control-flow-directing instruction.
pub struct Statement<'db> {
    pub kind: StatementKind<'db>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum StatementKind<'db> {
    Assign(Place<'db>, Rvalue<'db>),
}

/// An `Rvalue` (right-hand value) is a computation that produces a value.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Rvalue<'db> {
    Use(Operand<'db>),
    BinaryOp(BinOp, Operand<'db>, Operand<'db>),
    UnaryOp(UnOp, Operand<'db>),
    Constant(Constant<'db>),
    /// Constructs a struct or enum variant.
    /// Example: `MyStruct { field1: op1, field2: op2 }`
    Aggregate(AggregateKind<'db>, Vec<Operand<'db>>),
    /// Array literal.
    Array(Vec<Operand<'db>>),
}

/// An `Operand` is an input to an `Rvalue`.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Operand<'db> {
    Place(Place<'db>),
    Constant(Constant<'db>),
    FunctionRef(FunctionId<'db>),
}

/// A `Place` is a location in memory, like a local variable or a field.
/// This is the "left-hand side" of an assignment or the base of a field access.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Place<'db> {
    /// A local variable, temporary, or argument (e.g., `x`).
    Local(LocalId),
    /// A projection into a field of a struct or enum variant.
    /// Example: `my_struct.field2` would be `Place::Field(Box::new(Place::Local(my_struct_id)), 1)`
    Field(Box<Place<'db>>, usize),
    // Future additions: Index for arrays, Deref for pointers.
    #[doc(hidden)]
    __Phantom(PhantomData<&'db ()>),
}

/// Specifies what kind of aggregate value is being constructed.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum AggregateKind<'db> {
    /// Constructing a struct, identified by its `TypeId`.
    Struct(TypeId<'db>),
    /// Constructing an enum variant. We need the `TypeId` of the whole enum
    /// and the `variant_idx` of the specific variant being constructed.
    EnumVariant(TypeId<'db>, usize),
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Constant<'db> {
    Int(i64),
    Float(u64), // Store as bits for Eq/Hash
    Bool(bool),
    String(Symbol<'db>),
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
}

impl BinOp {
    pub fn from_ast(kind: scrap_ast::operators::BinOpKind) -> Self {
        match kind {
            scrap_ast::operators::BinOpKind::Add => BinOp::Add,
            scrap_ast::operators::BinOpKind::Sub => BinOp::Sub,
            scrap_ast::operators::BinOpKind::Mul => BinOp::Mul,
            scrap_ast::operators::BinOpKind::Div => BinOp::Div,
            scrap_ast::operators::BinOpKind::Rem => BinOp::Rem,
            scrap_ast::operators::BinOpKind::And => BinOp::And,
            scrap_ast::operators::BinOpKind::Or => BinOp::Or,
            scrap_ast::operators::BinOpKind::BitXor => BinOp::BitXor,
            scrap_ast::operators::BinOpKind::BitAnd => BinOp::BitAnd,
            scrap_ast::operators::BinOpKind::BitOr => BinOp::BitOr,
            scrap_ast::operators::BinOpKind::Shl => BinOp::Shl,
            scrap_ast::operators::BinOpKind::Shr => BinOp::Shr,
            scrap_ast::operators::BinOpKind::Eq => BinOp::Eq,
            scrap_ast::operators::BinOpKind::Lt => BinOp::Lt,
            scrap_ast::operators::BinOpKind::Le => BinOp::Le,
            scrap_ast::operators::BinOpKind::Ne => BinOp::Ne,
            scrap_ast::operators::BinOpKind::Ge => BinOp::Ge,
            scrap_ast::operators::BinOpKind::Gt => BinOp::Gt,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum UnOp {
    Neg,
    Not,
}
