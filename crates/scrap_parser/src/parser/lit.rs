use scrap_ast::lit::LitKind;
use scrap_diagnostics::{Level, annotate_snippets::Group};

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_lit(&mut self) -> crate::PResult<'a, scrap_ast::lit::Lit<'db>> {
        let lit_span = self.token.span;
        let lit: scrap_lexer::Literal = self.token.node.try_into().unwrap();

        let kind = match lit {
            scrap_lexer::Literal::Str => LitKind::Str,
            scrap_lexer::Literal::Float => LitKind::Float,
            scrap_lexer::Literal::Int => LitKind::Integer,
            scrap_lexer::Literal::Bool => LitKind::Bool,
            scrap_lexer::Literal::Ident => {
                return Err(Group::with_title(
                    Level::ERROR.primary_title("Ident should not be parsed as literal"),
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
