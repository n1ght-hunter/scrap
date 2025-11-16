use crate::PResult;
use scrap_ast::pat::{Pat, PatKind};

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_pat_empty(&mut self) -> PResult<'a, Pat<'db>> {
        Ok(Pat {
            id: self.state.new_node_id(),
            kind: PatKind::Missing,
            span: self.token.span,
        })
    }

    pub fn parse_pat(&mut self) -> PResult<'a, Pat<'db>> {
        let ident = self.parse_ident()?;
        Ok(Pat {
            id: self.state.new_node_id(),
            kind: PatKind::Ident(
                scrap_ast::pat::BindingMode(
                    scrap_ast::pat::ByRef::No,
                    scrap_shared::Mutability::Not,
                ),
                ident,
                None,
            ),
            span: self.token.span,
        })
    }
}
