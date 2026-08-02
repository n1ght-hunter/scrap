use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{fndef::Param, node_id::NodeId, typedef::Ty};
use scrap_shared::ident::{Ident, Symbol};

/// An `extern` block: `extern "C" { fn foo(...) -> ...; }`
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct ForeignMod {
    /// The ABI string, e.g. `"C"`
    pub abi: Symbol,
    /// The foreign function declarations inside the block
    pub items: ThinVec<ForeignItem>,
    pub span: Span,
}

/// A single foreign function declaration: `fn ExitProcess(exit_code: usize) -> !;`
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct ForeignItem {
    pub id: NodeId,
    pub ident: Ident,
    pub args: ThinVec<Param>,
    pub ret_type: Option<Ty>,
    pub span: Span,
}

impl scrap_shared::pretty_print::PrettyPrint for ForeignMod {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        salsa::with_attached_database(|_db| {
            write!(f, "extern \"{}\" {{", self.abi.text())?;
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

impl scrap_shared::pretty_print::PrettyPrint for ForeignItem {
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
