use scrap_ast::{
    item::{Item, ItemKind},
    module::{Inline, Module},
};
use scrap_diagnostics::{AnnotationKind, Level, Snippet};
use scrap_lexer::Token;
use scrap_span::Span;
use thin_vec::ThinVec;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn check_module(&mut self) -> bool {
        self.check(Token::Mod)
    }

    pub fn parse_module(&mut self) -> crate::PResult<'a, ItemKind<'db>> {
        let start_span = self.token.span.start(self.db);
        self.expect(Token::Mod)?;
        let ident = self.parse_ident()?;
        let _guard = self.guard_current_module_path(ident);

        if self.eat(Token::Semicolon) {
            Ok(ItemKind::Module(
                self.current_module_path().to_owned(),
                Module::Unloaded,
            ))
        } else if self.eat(Token::LBrace) {
            let items = self.parse_module_inner(Token::RBrace)?;

            Ok(ItemKind::Module(
                self.current_module_path().to_owned(),
                Module::Loaded(
                    items,
                    Inline::Yes,
                    Span::new(self.db, start_span, self.token.span.end(self.db)),
                ),
            ))
        } else {
            Err(Level::ERROR
                .primary_title("expected module body or semicolon")
                .element(
                    Snippet::source(self.source)
                        .path(self.state.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.token.span.to_range(self.db))
                                .label("expected `{` or `;`".to_string()),
                        ),
                ))
        }
    }

    pub fn parse_module_inner(
        &mut self,
        term: Token,
    ) -> crate::PResult<'a, ThinVec<Box<Item<'db>>>> {
        let mut items = ThinVec::new();
        while !self.check(term) && !self.token_stream.eof() {
            let item = self.parse_item()?;
            items.push(item);
        }
        self.expect(term)?;
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_test_utils::{ExtendRes, parse_with};

    use super::*;

    #[test]
    fn test_parse_module_loaded() {
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = crate::parser::parse_test_utils::parse_with(
            &db,
            "mod my_module { fn my_function() {} }",
        );
        let item = parser.parse_module().unwrap_or_render();
        match item {
            ItemKind::Module(ident, module) => {
                assert_eq!(ident.segments.last().unwrap().ident.name.text(&db), "my_module");
                match module {
                    Module::Loaded(items, inline, span) => {
                        assert_eq!(inline, Inline::Yes);
                        assert_eq!(span.to_range(&db), 0..37);
                        assert_eq!(items.len(), 1);
                        match &items[0].kind {
                            ItemKind::Fn(fndef) => {
                                assert_eq!(fndef.ident(&db).name.text(&db), "my_function");
                                assert_eq!(fndef.span(&db).to_range(&db), 16..35);
                            }
                            _ => panic!("Expected function item inside module"),
                        }
                    }
                    _ => panic!("Expected loaded module"),
                }
            }
            _ => panic!("Expected module item"),
        }
    }
}
