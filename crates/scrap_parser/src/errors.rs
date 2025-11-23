use scrap_diagnostics::Level;
use scrap_errors::ErrorGuaranteed;
use scrap_lexer::Token;

impl<'a, 'db> crate::parser::Parser<'a, 'db> {
    pub fn unexpected_token_error(&mut self, expected_tokens: &[Token]) -> ErrorGuaranteed {
        #[cfg(debug_assertions)]
        if expected_tokens.is_empty() {
            panic!("expected_tokens must contain at least one token");
        }
        let expected: Vec<String> = expected_tokens.iter().map(|t| format!("`{}`", t)).collect();
        let expected_str = expected.join(", ");

        self.db.dcx().emit_err(
            Level::ERROR.primary_title("unexpected token").element(
                scrap_diagnostics::Snippet::source(self.source)
                    .path(self.state.file_name)
                    .annotation(
                        scrap_diagnostics::AnnotationKind::Primary
                            .span(self.token.span.to_range(self.db))
                            .label(if expected_tokens.len() == 1 {
                                format!("expected {} found `{}`", expected_str, self.token.node)
                            } else {
                                format!(
                                    "expected one of {} found `{}`",
                                    expected_str, self.token.node
                                )
                            }),
                    ),
            ),
        )
    }

    pub fn raise_unexpected_token_error(&mut self) -> ErrorGuaranteed {
        self.unexpected_token_error(
            self.expected_token_types
                .iter()
                .collect::<Vec<_>>()
                .as_slice(),
        )
    }
}
