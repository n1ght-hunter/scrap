use std::marker::PhantomData;

use scrap_shared::types::{FloatTy, FloatVal, IntTy, IntVal, Mutability, UintTy, UintVal};
use scrap_shared::{id::ModuleId, ident::Symbol};

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
    pub id: ModuleId<'db>,
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
    ExternFunction(ExternFn<'db>),
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
/// An extern function declaration with its ABI and signature but no body.
pub struct ExternFn<'db> {
    /// The ABI of the extern function (e.g. "C").
    pub abi: Symbol<'db>,
    /// The signature of the extern function.
    pub signature: Signature<'db>,
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
    /// The parameter types of the function.
    #[tracked]
    #[returns(ref)]
    pub params: Vec<Ty<'db>>,
    /// The return type of the function. `Ty::Void` for functions with no return value.
    pub return_ty: Ty<'db>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Ty<'db> {
    /// The void type, for functions that return nothing.
    Void,
    /// A primitive boolean type.
    Bool,
    /// A signed integer type.
    Int(IntTy),
    /// An unsigned integer type.
    Uint(UintTy),
    /// A floating-point type.
    Float(FloatTy),
    /// A primitive string type.
    Str,
    /// A user-defined struct or enum, referenced by its unique ID.
    Adt(TypeId<'db>),
    /// Represents a type that never returns a value, like a function that always panics.
    Never,
    /// A tuple type, e.g. `(i32, bool)` for checked arithmetic results.
    Tuple(Vec<Ty<'db>>),
    /// A GC-managed reference type: `&T` or `&mut T`.
    Ref(Box<Ty<'db>>, Mutability),
    /// A GC-managed pointer type: `*T`.
    Ptr(Box<Ty<'db>>),
}

#[salsa::tracked(debug, persist)]
/// The MIR for a single function, represented as a Control Flow Graph (CFG).
/// Local layout: _0 = return place, _1.._param_count = params, rest = locals/temps.
pub struct Body<'db> {
    #[tracked]
    #[returns(ref)]
    pub blocks: Vec<BasicBlock<'db>>,
    #[tracked]
    #[returns(ref)]
    pub local_decls: Vec<LocalDecl<'db>>,
    /// Number of function parameters (locals _1 through _param_count).
    pub param_count: usize,
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

/// What to do when unwinding reaches a Call or Assert terminator.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum UnwindAction {
    /// Continue execution (abort on panic). Default for now.
    Continue,
    /// Unwind to a cleanup block (future: for drop glue).
    Cleanup(BasicBlockId),
    /// Unwinding is unreachable (e.g., inside a cleanup block).
    Unreachable,
}

/// Terminators are instructions that end a basic block and transfer control.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Terminator<'db> {
    Goto {
        target: BasicBlockId,
    },
    SwitchInt {
        discr: Operand<'db>,
        targets: SwitchTargets,
    },
    Return,
    Call {
        func: Operand<'db>,
        args: Vec<Operand<'db>>,
        destination: Place<'db>,
        /// `None` when the callee returns `!` (never returns).
        target: Option<BasicBlockId>,
        unwind: UnwindAction,
    },
    /// Assert a condition holds, otherwise panic.
    /// Used for overflow checks after checked arithmetic intrinsics.
    /// `expected` is the value `cond` must equal; if `cond != expected`, panic.
    Assert {
        cond: Operand<'db>,
        expected: bool,
        msg: AssertMessage,
        target: BasicBlockId,
        unwind: UnwindAction,
    },
    #[default]
    Unreachable,
}

/// Targets for a `SwitchInt` terminator: labeled value→block pairs + otherwise.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct SwitchTargets {
    /// (discriminant_value, target_block) pairs.
    pub values: Vec<(u128, BasicBlockId)>,
    /// Fallback block when no value matches.
    pub otherwise: BasicBlockId,
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
    /// A compiler-builtin intrinsic function call.
    /// Replaces the old BinaryOp/UnaryOp — all operations are intrinsic calls.
    Intrinsic(IntrinsicOp, Vec<Operand<'db>>),
    Constant(Constant<'db>),
    /// Constructs a struct or enum variant.
    /// Example: `MyStruct { field1: op1, field2: op2 }`
    Aggregate(AggregateKind<'db>, Vec<Operand<'db>>),
    /// Array literal.
    Array(Vec<Operand<'db>>),
    /// Heap-allocate a value and return a GC-managed reference.
    /// `Box(inner_ty, value)` — allocates space for `inner_ty`, stores `value`, returns pointer.
    Box(Ty<'db>, Operand<'db>),
    /// Read the discriminant (tag) of an enum value.
    Discriminant(Place<'db>),
    /// Take a reference to a place: `&place` or `&mut place`.
    Ref(Mutability, Place<'db>),
    /// Spawn a coroutine: `spawn func(args)`.
    /// First operand is the function reference, rest are arguments.
    Spawn(Operand<'db>, Vec<Operand<'db>>),
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
    /// Example: `my_struct.field2` would be `Place::Field(Box::new(Place::Local(my_struct_id)), 1, Some(sym))`
    Field(Box<Place<'db>>, usize, Option<Symbol<'db>>),
    /// Dereference a GC reference: `*place`.
    Deref(Box<Place<'db>>),
    /// Project an enum place to a specific variant.
    /// `(_1 as Some)` = `Place::Downcast(Local(1), 1, Some("Some"))`
    Downcast(Box<Place<'db>>, usize, Option<Symbol<'db>>),
    #[doc(hidden)]
    __Phantom(PhantomData<&'db ()>),
}

/// Specifies what kind of aggregate value is being constructed.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum AggregateKind<'db> {
    /// Constructing a struct, identified by its `TypeId` and field names.
    Struct(TypeId<'db>, Vec<Symbol<'db>>),
    /// Constructing an enum variant. We need the `TypeId` of the whole enum
    /// and the `variant_idx` of the specific variant being constructed.
    EnumVariant(TypeId<'db>, usize),
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Constant<'db> {
    Void,
    Int(IntVal),
    Uint(UintVal),
    Float(FloatVal),
    Bool(bool),
    String(Symbol<'db>),
}

/// A compiler-builtin operation represented as an inline function call.
///
/// All operations in the IR (arithmetic, comparison, bitwise, unary) are
/// represented as intrinsic calls rather than special IR constructs.
/// Checked variants (e.g. `AddWithOverflow`) return `(T, bool)` where
/// the bool indicates overflow/error; these are paired with `Assert` terminators.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum IntrinsicOp {
    // Checked integer arithmetic — return (T, bool)
    AddWithOverflow,
    SubWithOverflow,
    MulWithOverflow,
    DivWithZeroCheck,
    RemWithZeroCheck,
    ShlChecked,
    ShrChecked,

    // Unchecked arithmetic — return T (used for floats, or future wrapping ops)
    Add,
    Sub,
    Mul,
    Div,
    Rem,

    // Comparisons — return bool
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical — return bool
    And,
    Or,

    // Bitwise — return T
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,

    // Unary — return T
    Neg,
    Not,
}

/// Structured panic message for `Assert` terminators.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum AssertMessage {
    Overflow(IntrinsicOp),
    DivisionByZero,
    RemainderByZero,
    ShiftOverflow,
}
