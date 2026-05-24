//! The cross-process metadata schema shared between the `scrap-rustc` driver
//! (which links `rustc_private` and emits this) and scrapc (which reads it).
//!
//! It is intentionally serde-only with no compiler dependencies: the driver is
//! built by the pinned nightly with `rustc-dev`, scrapc by an ordinary toolchain,
//! and this crate is the one thing both link. The driver derives everything here
//! from a single `TyCtxt`, so the layouts/ABI scrapc mirrors and the symbols it
//! links cannot disagree.

use serde::{Deserialize, Serialize};

/// Schema version. Bumped when the shape below changes incompatibly so scrapc
/// can reject a stale dump rather than misread it.
pub const SCHEMA_VERSION: u32 = 5;

/// The full dump emitted for one anchor compilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustMetadata {
    pub schema_version: u32,
    /// The target triple the anchor was compiled for.
    pub target: String,
    /// One entry per requested dependency crate.
    pub crates: Vec<RustCrate>,
}

/// The public API surface of one dependency crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustCrate {
    pub name: String,
    pub fns: Vec<RustFn>,
    pub types: Vec<RustType>,
}

/// A public free function or associated function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustFn {
    /// Fully-qualified path, e.g. `regex::Regex::new`.
    pub path: String,
    /// Names of the function's own generic type parameters; empty = monomorphic.
    pub generic_params: Vec<String>,
    pub params: Vec<RustTyRef>,
    pub ret: RustTyRef,
    /// Whether this is a method with a `self` receiver (`params[0]` is the
    /// receiver). `false` for free functions and associated fns.
    pub has_self: bool,
    /// The concrete symbol + ABI, present only for non-generic functions (a
    /// generic function has no symbol until instantiated — see Phase 4).
    pub mono: Option<MonoFn>,
}

/// The codegen-facing facts about a concrete (monomorphic) function instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonoFn {
    /// The exact mangled symbol the archive exports (v0 mangling).
    pub symbol: String,
    pub abi: FnAbiInfo,
}

/// The per-argument / return ABI of a concrete function instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnAbiInfo {
    /// The calling convention (e.g. `Rust`, `C`), as rustc names it.
    pub conv: String,
    pub args: Vec<ArgAbi>,
    pub ret: ArgAbi,
}

/// How one argument or the return value is passed across the ABI boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgAbi {
    pub ty: RustTyRef,
    pub mode: PassMode,
}

/// A scalar ABI component, mappable to a Cranelift type. Pointers are recorded
/// as `Ptr` so codegen can use the target's pointer width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scalar {
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Ptr,
}

/// The ABI pass mode, mirroring rustc's `PassMode`. `Direct`/`Pair` carry their
/// scalar component types so codegen can build the exact Cranelift signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassMode {
    /// Zero-sized; passed as nothing.
    Ignore,
    /// A single scalar in a register.
    Direct(Scalar),
    /// Two scalars in two registers (`ScalarPair`).
    Pair(Scalar, Scalar),
    /// Passed/returned through memory by pointer (`sret` for returns). `size` is
    /// the value's byte size, needed to build a `StructArgument` ABI param.
    Indirect { on_stack: bool, size: u64 },
    /// Coerced to a differently-shaped scalar/array before passing.
    Cast,
}

/// A reference to a Rust type, by display string. Phase 4 will grow this into a
/// structured form; for now the display name is enough to mirror and to match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustTyRef {
    pub display: String,
}

/// The kind of an algebraic data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdtKind {
    Struct,
    Enum,
    Union,
}

/// A public struct, enum, or union.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustType {
    pub path: String,
    pub kind: AdtKind,
    /// The `repr` as rustc reports it (e.g. `Rust`, `C`, `transparent`).
    pub repr: String,
    pub generic_params: Vec<String>,
    /// Fields of a struct/union, or the empty list for an enum (see `variants`).
    pub fields: Vec<RustField>,
    /// Variants of an enum, or the empty list otherwise.
    pub variants: Vec<RustVariant>,
    /// Inherent methods (associated fns with a `self` receiver), by full path
    /// `crate::Type::method`. Associated fns *without* a receiver live in the
    /// crate's `fns` list instead (importable by path like a free fn).
    pub methods: Vec<RustFn>,
    /// Whether the type (or its field/variant list) is `#[non_exhaustive]` —
    /// field-by-field construction from Scrap is forbidden when set.
    pub non_exhaustive: bool,
    /// Concrete layout, present only for non-generic types.
    pub layout: Option<LayoutInfo>,
}

/// One field of a struct, union, or enum variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustField {
    pub name: String,
    /// Whether the field is visible (`pub`) at the crate boundary — gates
    /// field-by-field construction from Scrap (§5 of the plan).
    pub public: bool,
    pub ty: RustTyRef,
}

/// One enum variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustVariant {
    pub name: String,
    pub fields: Vec<RustField>,
}

/// The concrete in-memory layout of a non-generic type, exactly as the archive
/// was compiled with: scrapc mirrors this to place fields at the right offsets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutInfo {
    pub size: u64,
    pub align: u64,
    /// Byte offset of each field, in declaration order (repr(Rust) may reorder
    /// the underlying storage, so these need not be ascending).
    pub field_offsets: Vec<u64>,
    /// Whether the type is `Copy` (never dropped by Scrap's RAII model).
    pub is_copy: bool,
    /// Whether the type has non-trivial drop glue (`Drop` impl or a droppable
    /// field). Only such types get a drop wrapper + RAII drop in Scrap.
    pub needs_drop: bool,
}
