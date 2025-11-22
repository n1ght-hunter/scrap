use crate::PResult;
use scrap_ast::typedef::{Ty, TyKind};
use scrap_shared::path::Path;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_type(&mut self) -> PResult<'a, Ty<'db>> {
        // For now, we only support identifier types
        let ident = self.parse_ident()?;
        Ok(Ty {
            id: self.state.new_node_id(),
            span: ident.span,
            kind: TyKind::Path(Path::from_ident(ident)),
        })
    }
}
