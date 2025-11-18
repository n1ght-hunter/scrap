use scrap_span::Span;

use crate::node_id::NodeId;

/// A literal value with its kind and actual data.
/// This represents any literal value that appears in source code.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Lit<'db> {
    /// Unique identifier for this literal node
    pub id: NodeId,
    /// The kind of literal (determines how it should be interpreted)
    pub kind: LitKind,
    /// The span of the literal in the source code
    pub span: Span<'db>,
    // In full Rust AST, there would also be:
    // pub symbol: Symbol,        // The original source representation
    // pub suffix: Option<Symbol>, // Type suffix like "f32" in "1.0f32"
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Lit<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        match self.kind {
            LitKind::Bool => write!(f, "<bool literal>"),
            LitKind::Integer => write!(f, "<integer literal>"),
            LitKind::Float => write!(f, "<float literal>"),
            LitKind::Str => write!(f, "<string literal>"),
        }
    }
}

/// Literal kinds, following Rust AST enum structure.
/// This is a simplified subset of the full Rust LitKind enum.
///
/// Note that the entire literal (including the suffix) is considered when
/// deciding the `LitKind`. This means that float literals like `1f32` are
/// classified by this type as `Float`. This is different to `token::LitKind`
/// which does *not* consider the suffix.
#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum LitKind {
    /// A boolean literal (`true`, `false`)
    Bool,
    /// An integer literal (`1`)
    Integer,
    /// A float literal (`1.0`, `1f64` or `1E10f64`)
    Float,
    /// A string literal (`"foo"`). The symbol is unescaped, and so may differ
    /// from the original token's symbol.
    Str,
}
