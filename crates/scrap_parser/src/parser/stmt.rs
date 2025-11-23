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

        if self.check_item() {
            let item = self.parse_item()?;
            return Ok(Stmt {
                id: self.state.new_node_id(),
                span: item.span,
                kind: StmtKind::Item(item),
            });
        }

        if self.eat(Token::Semicolon) {
            return Ok(Stmt {
                id: self.state.new_node_id(),
                kind: StmtKind::Empty,
                span: self.token.span,
            });
        }

        let expr = self.parse_expr()?;
        return Ok(Stmt {
            id: self.state.new_node_id(),
            span: expr.span,
            kind: if self.eat(Token::Semicolon) {
                StmtKind::Semi(Box::new(expr))
            } else {
                StmtKind::Expr(Box::new(expr))
            },
        });

        // Err(Level::ERROR
        //     .primary_title("unexpected token while parsing statement")
        //     .element(
        //         scrap_diagnostics::Snippet::source(self.source)
        //             .path(self.state.file_name)
        //             .annotation(
        //                 scrap_diagnostics::AnnotationKind::Primary
        //                     .span(self.token.span.to_range(self.db))
        //                     .label("expected a statement here".to_string()),
        //             ),
        //     ))
    }
}
