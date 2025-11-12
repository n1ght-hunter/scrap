use crate::PResult;
use scrap_ast::{
    path::Path,
    typedef::{Ty, TyKind},
};

impl<'a> super::Parser<'a> {
    pub fn parse_type(&mut self) -> PResult<'a, Ty> {
        // For now, we only support identifier types
        let ident = self.parse_ident()?;
        Ok(Ty {
            id: self.state.new_node_id(),
            span: ident.span,
            kind: TyKind::Path(Path::from_ident(ident)),
        })
    }
}
