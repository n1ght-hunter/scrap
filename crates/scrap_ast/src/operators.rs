use scrap_lexer::Token;
use scrap_span::Spanned;

/// A binary operator with its source location span.
/// This matches the Rust AST pattern of wrapping operator kinds with span information.
pub type BinOp<'db> = Spanned<'db, BinOpKind>;
pub type AssignOp<'db> = Spanned<'db, AssignOpKind>;

#[derive(
    Clone, Copy, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum AssocOp {
    /// A binary op.
    Binary(BinOpKind),
    /// `?=` where ? is one of the assignable BinOps
    AssignOp(AssignOpKind),
    /// `=`
    Assign,
}

/// Assignment operator kinds, following Rust AST enum structure exactly.
/// These represent the different types of assignment operations available in the language.
#[derive(
    Clone, Copy, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum AssignOpKind {
    /// The `+=` operator (addition)
    AddAssign,
    /// The `-=` operator (subtraction)
    SubAssign,
    /// The `*=` operator (multiplication)
    MulAssign,
    /// The `/=` operator (division)
    DivAssign,
    /// The `%=` operator (modulus)
    RemAssign,
    /// The `^=` operator (bitwise xor)
    BitXorAssign,
    /// The `&=` operator (bitwise and)
    BitAndAssign,
    /// The `|=` operator (bitwise or)
    BitOrAssign,
    /// The `<<=` operator (shift left)
    ShlAssign,
    /// The `>>=` operator (shift right)
    ShrAssign,
}

/// Binary operator kinds, following Rust AST enum structure exactly.
/// These represent the different types of binary operations available in the language.
#[derive(
    Clone, Copy, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum BinOpKind {
    /// The `+` operator (addition)
    Add,
    /// The `-` operator (subtraction)
    Sub,
    /// The `*` operator (multiplication)
    Mul,
    /// The `/` operator (division)
    Div,
    /// The `%` operator (modulus)
    Rem,
    /// The `&&` operator (logical and)
    And,
    /// The `||` operator (logical or)
    Or,
    /// The `^` operator (bitwise xor)
    BitXor,
    /// The `&` operator (bitwise and)
    BitAnd,
    /// The `|` operator (bitwise or)
    BitOr,
    /// The `<<` operator (shift left)
    Shl,
    /// The `>>` operator (shift right)
    Shr,
    /// The `==` operator (equality)
    Eq,
    /// The `<` operator (less than)
    Lt,
    /// The `<=` operator (less than or equal to)
    Le,
    /// The `!=` operator (not equal to)
    Ne,
    /// The `>=` operator (greater than or equal to)
    Ge,
    /// The `>` operator (greater than)
    Gt,
}

impl AssocOp {
    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Assign => Some(AssocOp::Assign),
            Token::Add => Some(AssocOp::Binary(BinOpKind::Add)),
            Token::Sub => Some(AssocOp::Binary(BinOpKind::Sub)),
            Token::Mul => Some(AssocOp::Binary(BinOpKind::Mul)),
            Token::Div => Some(AssocOp::Binary(BinOpKind::Div)),
            Token::Rem => Some(AssocOp::Binary(BinOpKind::Rem)),
            Token::And => Some(AssocOp::Binary(BinOpKind::And)),
            Token::Or => Some(AssocOp::Binary(BinOpKind::Or)),
            Token::BitXor => Some(AssocOp::Binary(BinOpKind::BitXor)),
            Token::BitAnd => Some(AssocOp::Binary(BinOpKind::BitAnd)),
            Token::BitOr => Some(AssocOp::Binary(BinOpKind::BitOr)),
            Token::Shl => Some(AssocOp::Binary(BinOpKind::Shl)),
            Token::Shr => Some(AssocOp::Binary(BinOpKind::Shr)),
            Token::Eq => Some(AssocOp::Binary(BinOpKind::Eq)),
            Token::Lt => Some(AssocOp::Binary(BinOpKind::Lt)),
            Token::Le => Some(AssocOp::Binary(BinOpKind::Le)),
            Token::Ne => Some(AssocOp::Binary(BinOpKind::Ne)),
            Token::Ge => Some(AssocOp::Binary(BinOpKind::Ge)),
            Token::Gt => Some(AssocOp::Binary(BinOpKind::Gt)),
            Token::AddAssign => Some(AssocOp::AssignOp(AssignOpKind::AddAssign)),
            Token::SubAssign => Some(AssocOp::AssignOp(AssignOpKind::SubAssign)),
            Token::MulAssign => Some(AssocOp::AssignOp(AssignOpKind::MulAssign)),
            Token::DivAssign => Some(AssocOp::AssignOp(AssignOpKind::DivAssign)),
            Token::RemAssign => Some(AssocOp::AssignOp(AssignOpKind::RemAssign)),
            Token::BitXorAssign => Some(AssocOp::AssignOp(AssignOpKind::BitXorAssign)),
            Token::BitAndAssign => Some(AssocOp::AssignOp(AssignOpKind::BitAndAssign)),
            Token::BitOrAssign => Some(AssocOp::AssignOp(AssignOpKind::BitOrAssign)),
            Token::ShlAssign => Some(AssocOp::AssignOp(AssignOpKind::ShlAssign)),
            Token::ShrAssign => Some(AssocOp::AssignOp(AssignOpKind::ShrAssign)),
            _ => None,
        }
    }
}
