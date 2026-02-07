use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{fndef::Param, node_id::NodeId, typedef::Ty};
use scrap_shared::ident::{Ident, Symbol};

/// An `extern` block: `extern "C" { fn foo(...) -> ...; }`
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct ForeignMod<'db> {
    /// The ABI string, e.g. `"C"`
    pub abi: Symbol<'db>,
    /// The foreign function declarations inside the block
    pub items: ThinVec<ForeignItem<'db>>,
    pub span: Span<'db>,
}

/// A single foreign function declaration: `fn ExitProcess(exit_code: usize) -> !;`
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct ForeignItem<'db> {
    pub id: NodeId,
    pub ident: Ident<'db>,
    pub args: ThinVec<Param<'db>>,
    pub ret_type: Option<Ty<'db>>,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for ForeignMod<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        salsa::with_attached_database(|db| {
            write!(f, "extern \"{}\" {{", self.abi.text(db))?;
            for item in self.items.iter() {
                writeln!(f)?;
                Self::write_indent(f, indent + 1)?;
                item.pretty_print_indent(f, indent + 1)?;
            }
            writeln!(f)?;
            Self::write_indent(f, indent)?;
            write!(f, "}}")
        })
        .unwrap_or_else(|| write!(f, "extern <no db>"))
    }
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for ForeignItem<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        write!(f, "fn ")?;
        self.ident.pretty_print_indent(f, 0)?;
        write!(f, "(")?;
        for (i, param) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            param.pretty_print_indent(f, 0)?;
        }
        write!(f, ")")?;
        if let Some(ret_type) = &self.ret_type {
            write!(f, " -> ")?;
            ret_type.pretty_print_indent(f, 0)?;
        }
        write!(f, ";")
    }
}
