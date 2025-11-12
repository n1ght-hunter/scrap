use crate::PResult;
use scrap_ast::pat::{Pat, PatKind};

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_pat(&mut self) -> PResult<'a, Pat<'db>> {
        Ok(Pat {
            id: self.state.new_node_id(),
            kind: PatKind::Missing,
            span: self.token.span,
        })
    }
}
