#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, salsa::Update, serde::Serialize, serde::Deserialize)]
pub enum IntTy {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
}

impl IntTy {
    pub fn name_str(&self) -> &'static str {
        match *self {
            IntTy::Isize => "isize",
            IntTy::I8 => "i8",
            IntTy::I16 => "i16",
            IntTy::I32 => "i32",
            IntTy::I64 => "i64",
            IntTy::I128 => "i128",
        }
    }

    // pub fn name(self) -> Symbol {
    //     match self {
    //         IntTy::Isize => sym::isize,
    //         IntTy::I8 => sym::i8,
    //         IntTy::I16 => sym::i16,
    //         IntTy::I32 => sym::i32,
    //         IntTy::I64 => sym::i64,
    //         IntTy::I128 => sym::i128,
    //     }
    // }

    pub fn bit_width(&self) -> Option<u64> {
        Some(match *self {
            IntTy::Isize => return None,
            IntTy::I8 => 8,
            IntTy::I16 => 16,
            IntTy::I32 => 32,
            IntTy::I64 => 64,
            IntTy::I128 => 128,
        })
    }

    pub fn normalize(&self, target_width: u16) -> Self {
        match self {
            IntTy::Isize => match target_width {
                16 => IntTy::I16,
                32 => IntTy::I32,
                64 => IntTy::I64,
                _ => unreachable!(),
            },
            _ => *self,
        }
    }

    pub fn to_unsigned(self) -> UintTy {
        match self {
            IntTy::Isize => UintTy::Usize,
            IntTy::I8 => UintTy::U8,
            IntTy::I16 => UintTy::U16,
            IntTy::I32 => UintTy::U32,
            IntTy::I64 => UintTy::U64,
            IntTy::I128 => UintTy::U128,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, salsa::Update, serde::Serialize, serde::Deserialize)]
pub enum UintTy {
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl UintTy {
    pub fn name_str(&self) -> &'static str {
        match *self {
            UintTy::Usize => "usize",
            UintTy::U8 => "u8",
            UintTy::U16 => "u16",
            UintTy::U32 => "u32",
            UintTy::U64 => "u64",
            UintTy::U128 => "u128",
        }
    }

    // pub fn name(self) -> Symbol {
    //     match self {
    //         UintTy::Usize => sym::usize,
    //         UintTy::U8 => sym::u8,
    //         UintTy::U16 => sym::u16,
    //         UintTy::U32 => sym::u32,
    //         UintTy::U64 => sym::u64,
    //         UintTy::U128 => sym::u128,
    //     }
    // }

    pub fn bit_width(&self) -> Option<u64> {
        Some(match *self {
            UintTy::Usize => return None,
            UintTy::U8 => 8,
            UintTy::U16 => 16,
            UintTy::U32 => 32,
            UintTy::U64 => 64,
            UintTy::U128 => 128,
        })
    }

    pub fn normalize(&self, target_width: u16) -> Self {
        match self {
            UintTy::Usize => match target_width {
                16 => UintTy::U16,
                32 => UintTy::U32,
                64 => UintTy::U64,
                _ => unreachable!(),
            },
            _ => *self,
        }
    }

    pub fn to_signed(self) -> IntTy {
        match self {
            UintTy::Usize => IntTy::Isize,
            UintTy::U8 => IntTy::I8,
            UintTy::U16 => IntTy::I16,
            UintTy::U32 => IntTy::I32,
            UintTy::U64 => IntTy::I64,
            UintTy::U128 => IntTy::I128,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, salsa::Update, serde::Serialize, serde::Deserialize)]
pub enum FloatTy {
    F16,
    F32,
    F64,
    F128,
}

impl FloatTy {
    pub fn name_str(self) -> &'static str {
        match self {
            FloatTy::F16 => "f16",
            FloatTy::F32 => "f32",
            FloatTy::F64 => "f64",
            FloatTy::F128 => "f128",
        }
    }

    // pub fn name(self) -> Symbol {
    //     match self {
    //         FloatTy::F16 => sym::f16,
    //         FloatTy::F32 => sym::f32,
    //         FloatTy::F64 => sym::f64,
    //         FloatTy::F128 => sym::f128,
    //     }
    // }

    pub fn bit_width(self) -> u64 {
        match self {
            FloatTy::F16 => 16,
            FloatTy::F32 => 32,
            FloatTy::F64 => 64,
            FloatTy::F128 => 128,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, salsa::Update, serde::Serialize, serde::Deserialize)]
pub enum IntVal {
    Isize(isize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

impl IntVal {
    pub fn ty(&self) -> IntTy {
        match self {
            IntVal::Isize(_) => IntTy::Isize,
            IntVal::I8(_) => IntTy::I8,
            IntVal::I16(_) => IntTy::I16,
            IntVal::I32(_) => IntTy::I32,
            IntVal::I64(_) => IntTy::I64,
            IntVal::I128(_) => IntTy::I128,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, salsa::Update, serde::Serialize, serde::Deserialize)]
pub enum UintVal {
    Usize(usize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

impl UintVal {
    pub fn ty(&self) -> UintTy {
        match self {
            UintVal::Usize(_) => UintTy::Usize,
            UintVal::U8(_) => UintTy::U8,
            UintVal::U16(_) => UintTy::U16,
            UintVal::U32(_) => UintTy::U32,
            UintVal::U64(_) => UintTy::U64,
            UintVal::U128(_) => UintTy::U128,
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Copy, salsa::Update, serde::Serialize, serde::Deserialize)]
pub enum FloatVal {
    F32(f32),
    F64(f64),
}

impl Eq for FloatVal {}

impl std::hash::Hash for FloatVal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            FloatVal::F32(f) => f.to_bits().hash(state),
            FloatVal::F64(f) => f.to_bits().hash(state),
        }
    }
}

impl Ord for FloatVal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl FloatVal {
    pub fn ty(&self) -> FloatTy {
        match self {
            FloatVal::F32(_) => FloatTy::F32,
            FloatVal::F64(_) => FloatTy::F64,
        }
    }
}

/// A unique identifier for AST nodes. NodeIds are used throughout the compiler
/// to track and reference specific nodes during analysis and compilation.
/// Every AST node that can be referenced has a unique NodeId.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize)]
pub struct NodeId {
    id: i32,
    file_hash: u64,
}

impl NodeId {
    /// Get the raw ID value (useful for debugging and serialization)
    pub fn as_i32(self) -> i32 {
        self.id
    }

    /// Create a NodeId from a u16 ID and a file hash
    pub fn new(id: u16, file_hash: u64) -> Self {
        NodeId {
            id: id as i32,
            file_hash,
        }
    }

    /// Create a special invalid NodeId for error recovery cases
    pub fn invalid() -> Self {
        NodeId {
            id: -1,
            file_hash: 0,
        }
    }

    pub fn dummy() -> Self {
        NodeId {
            id: -2,
            file_hash: 0,
        }
    }
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Copy,
    salsa::Update,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Mutability {
    // N.B. Order is deliberate, so that Not < Mut
    Not,
    Mut,
}

impl Mutability {
    pub fn invert(self) -> Self {
        match self {
            Mutability::Mut => Mutability::Not,
            Mutability::Not => Mutability::Mut,
        }
    }

    /// Returns `""` (empty string) or `"mut "` depending on the mutability.
    pub fn prefix_str(self) -> &'static str {
        match self {
            Mutability::Mut => "mut ",
            Mutability::Not => "",
        }
    }

    /// Returns `"&"` or `"&mut "` depending on the mutability.
    pub fn ref_prefix_str(self) -> &'static str {
        match self {
            Mutability::Not => "&",
            Mutability::Mut => "&mut ",
        }
    }

    /// Returns `"const"` or `"mut"` depending on the mutability.
    pub fn ptr_str(self) -> &'static str {
        match self {
            Mutability::Not => "const",
            Mutability::Mut => "mut",
        }
    }

    /// Returns `""` (empty string) or `"mutably "` depending on the mutability.
    pub fn mutably_str(self) -> &'static str {
        match self {
            Mutability::Not => "",
            Mutability::Mut => "mutably ",
        }
    }

    /// Return `true` if self is mutable
    pub fn is_mut(self) -> bool {
        matches!(self, Self::Mut)
    }

    /// Return `true` if self is **not** mutable
    pub fn is_not(self) -> bool {
        matches!(self, Self::Not)
    }
}
