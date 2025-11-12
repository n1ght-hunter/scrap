use scrap_ast::stmt::{Stmt, StmtKind};
use scrap_lexer::Token;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_stmt(&mut self) -> crate::PResult<'a, Stmt<'db>> {
        if self.check(Token::Let) {
            let local = self.parse_local()?;
            return Ok(Stmt {
                id: self.state.new_node_id(),
                span: local.span,
                kind: StmtKind::Let(Box::new(local)),
            });
        }

        Ok(Stmt {
            id: self.state.new_node_id(),
            kind: StmtKind::Empty,
            span: self.token.span,
        })
    }
}
