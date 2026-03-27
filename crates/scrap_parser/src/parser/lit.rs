use scrap_ast::lit::LitKind;
use scrap_diagnostics::{AnnotationKind, Level, Snippet};

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_lit(&mut self) -> crate::PResult<'a, scrap_ast::lit::Lit> {
        let lit_span = self.token.span;
        let lit: scrap_lexer::Literal = self.token.node.try_into().unwrap();

        let kind = match lit {
            scrap_lexer::Literal::Str => LitKind::Str,
            scrap_lexer::Literal::Float => LitKind::Float,
            scrap_lexer::Literal::Int => LitKind::Integer,
            scrap_lexer::Literal::Bool => LitKind::Bool,
            scrap_lexer::Literal::Ident => {
                let ident_str = &self.source[self.token.span.start..self.token.span.end];
                return Err(self.db.dcx().emit_err(
                    Level::ERROR
                        .primary_title(format!(
                            "Unexpected identifier '{}' where a literal was expected",
                            ident_str
                        ))
                        .element(
                            Snippet::source(self.source)
                                .annotation(
                                    AnnotationKind::Primary
                                        .span(self.token.span.range())
                                        .label("expected a literal here"),
                                )
                                .path(self.state.file_name),
                        ),
                ));
            }
        };

        self.bump();

        Ok(scrap_ast::lit::Lit {
            id: self.state.new_node_id(),
            kind,
            span: lit_span,
        })
    }
}
