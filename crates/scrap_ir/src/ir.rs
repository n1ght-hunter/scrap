#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub usize);

#[salsa::interned(debug)]
pub struct FunctionId<'db> {
    #[returns(ref)]
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resolved<T, U> {
    Resolved(T),
    Unresolved(U),
}

/// A unique, program-wide identifier for a user-defined type (struct or enum).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, Default)]
// A collection of modules forming a single compilation unit.
pub struct Can {
    /// The modules in this compilation unit.
    pub modules: Vec<Module>,
}

#[derive(Debug, Clone, Default)]
/// A module containing a list of items (functions, structs, enums, etc.) in a single namespace.
pub struct Module {
    pub path: String,
    pub items: Vec<Items>,
}

/// An item in a module: function, struct, enum, etc.
#[derive(Debug, Clone)]
pub enum Items {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
}

/// The MIR for a strcut
#[derive(Debug, Clone)]
pub struct Struct {
    /// The name of the struct.
    pub name: String,
    /// The fields of the struct.
    pub fields: Vec<(String, Ty)>,
}

/// The MIR for an enum
#[derive(Debug, Clone)]
pub struct Enum {
    /// The name of the enum.
    pub name: String,
    /// The variant of the enum.
    pub variant: EnumVariant,
}

/// An enum variant can be a unit, tuple, or struct variant.
#[derive(Debug, Clone)]
pub enum EnumVariant {
    /// A unit variant with no fields.
    Unit,
    /// A tuple variant with unnamed fields.
    Tuple(Vec<Ty>),
    /// A struct variant with named fields.
    Struct(Vec<(String, Ty)>),
}

#[derive(Debug, Clone)]
/// The MIR for a function
pub struct Function {
    /// The signature of the function.
    pub signature: Signature,
    /// The body of the function.
    pub body: Body,
}

#[derive(Debug, Clone)]
pub struct Signature {
    /// The name of the function.
    pub name: String,
    /// The parameters of the function.
    pub params: Vec<(String, Ty)>,
    /// The return type of the function.
    pub return_ty: Option<Ty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    /// A primitive boolean type.
    Bool,
    /// A primitive integer type (you would add size/signedness).
    Int,
    /// A primitive string type.
    Str,
    /// A user-defined struct or enum, referenced by its unique ID.
    Adt(Resolved<TypeId, String>),
    /// Represents a type that never returns a value, like a function that always panics.
    Never,
    /// Represents a type that is not yet known or determined.
    Infer,
}

/// The MIR for a single function, represented as a Control Flow Graph (CFG).
#[derive(Debug, Clone, Default)]
pub struct Body {
    pub blocks: Vec<BasicBlock>,
    pub local_decls: Vec<LocalDecl>,
}

/// A Basic Block: a sequence of statements with a single entry and a single exit.
#[derive(Debug, Clone, Default)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

/// Declaration for a local variable, argument, or temporary.
#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub name: Option<String>,
    pub ty: Ty,
}

/// Terminators are instructions that end a basic block and transfer control.
#[derive(Debug, Clone, Default)]
pub enum Terminator {
    Goto {
        target: BasicBlockId,
    },
    SwitchInt {
        discr: Operand,
        targets: Vec<BasicBlockId>,
    },
    Return,
    Call {
        func: Operand,
        args: Vec<Operand>,
        destination: Place,
        target: BasicBlockId,
    },
    #[default]
    Unreachable,
}

/// A statement is a simple, non-control-flow-directing instruction.
#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Assign(Place, Rvalue),
}

/// An `Rvalue` (right-hand value) is a computation that produces a value.
#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
    Constant(Constant),
    /// Constructs a struct or enum variant.
    /// Example: `MyStruct { field1: op1, field2: op2 }`
    Aggregate(AggregateKind, Vec<Operand>),
}

/// An `Operand` is an input to an `Rvalue`.
#[derive(Debug, Clone)]
pub enum Operand {
    Place(Place),
    Constant(Constant),
    FunctionRef(FunctionId),
}

/// A `Place` is a location in memory, like a local variable or a field.
/// This is the "left-hand side" of an assignment or the base of a field access.
#[derive(Debug, Clone)]
pub enum Place {
    /// A local variable, temporary, or argument (e.g., `x`).
    Local(LocalId),
    /// A projection into a field of a struct or enum variant.
    /// Example: `my_struct.field2` would be `Place::Field(Box::new(Place::Local(my_struct_id)), 1)`
    Field(Box<Place>, usize),
    // Future additions: Index for arrays, Deref for pointers.
}

// --- Leaf Data Types ---

/// Specifies what kind of aggregate value is being constructed.
#[derive(Debug, Clone)]
pub enum AggregateKind {
    /// Constructing a struct, identified by its `TypeId`.
    Struct(TypeId),
    /// Constructing an enum variant. We need the `TypeId` of the whole enum
    /// and the `variant_idx` of the specific variant being constructed.
    EnumVariant(TypeId, usize),
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
    And,
    Or,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
}
