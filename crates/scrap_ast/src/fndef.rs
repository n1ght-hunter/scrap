use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{block::Block, node_id::NodeId, pat::Pat, typedef::Ty};
use scrap_shared::ident::Ident;

#[salsa::tracked(debug, persist)]
pub struct FnDef<'db> {
    pub id: NodeId,
    pub ident: Ident<'db>,
    #[tracked]
    #[returns(ref)]
    pub args: ThinVec<Param<'db>>,
    #[tracked]
    #[returns(ref)]
    pub ret_type: Option<Ty<'db>>,
    #[tracked]
    #[returns(ref)]
    pub body: Block<'db>,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for FnDef<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        let res = salsa::with_attached_database(|db| {
            write!(f, "fn ")?;
            self.ident(db).pretty_print_indent(f, 0)?;
            write!(f, "(")?;
            for (i, param) in self.args(db).iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                param.pretty_print_indent(f, 0)?;
            }
            write!(f, ")")?;
            if let Some(ret_type) = &self.ret_type(db) {
                write!(f, " -> ")?;
                ret_type.pretty_print_indent(f, 0)?;
            }
            write!(f, " ")?;
            self.body(db).pretty_print_indent(f, indent)?;

            Ok(())
        });
        match res {
            Some(r) => r,
            None => write!(f, "<unable to pretty print function>"),
        }
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Param<'db> {
    pub id: NodeId,
    pub ident: Ident<'db>,
    pub ty: Box<Ty<'db>>,
    pub pat: Box<Pat<'db>>,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Param<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        self.ident.pretty_print_indent(f, 0)?;
        write!(f, ": ")?;
        self.ty.pretty_print_indent(f, 0)
    }
}
