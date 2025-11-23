use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{node_id::NodeId, stmt::Stmt};

/// A block expression. Following Rust AST structure.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Block<'db> {
    pub stmts: ThinVec<Stmt<'db>>,
    pub id: NodeId,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Block<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for stmt in &self.stmts {
            stmt.pretty_print_indent(f, indent + 1)?;
            writeln!(f)?;
        }
        Self::write_indent(f, indent)?;
        write!(f, "}}")
    }
}
