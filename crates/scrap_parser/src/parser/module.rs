use scrap_ast::{
    item::{Item, ItemKind},
    module::{Inline, Module, ModuleKind},
};
use scrap_diagnostics::{AnnotationKind, Level, Snippet};
use scrap_lexer::Token;
use scrap_shared::id::ModuleId;
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
            Ok(ItemKind::Module(Module::new(
                self.db,
                ModuleId::from_path(self.db, &self.current_module_path()),
                ModuleKind::Unloaded,
            )))
        } else if self.eat(Token::LBrace) {
            let items = self.parse_module_inner(Token::RBrace)?;
            let module = Module::new(
                self.db,
                ModuleId::from_path(self.db, &self.current_module_path()),
                ModuleKind::Loaded(
                    items,
                    Inline::Yes,
                    Span::new(self.db, start_span, self.token.span.end(self.db)),
                ),
            );
            self.modules.push(module);

            Ok(ItemKind::Module(module))
        } else {
            Err(self.db.dcx().emit_err(
                Level::ERROR
                    .primary_title("expected module body or semicolon")
                    .element(
                        Snippet::source(self.source)
                            .path(self.state.file_name)
                            .annotation(
                                AnnotationKind::Primary
                                    .span(self.token.span.to_range(self.db))
                                    .label("expected `{` or `;`".to_string()),
                            ),
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
    use crate::parser::parse_test_utils::ExtendRes;

    use super::*;

    #[scrap_macros::salsa_test]
    fn test_parse_module_loaded(db: &dyn scrap_shared::Db) {
        let mut parser = crate::parser::parse_test_utils::parse_with(
            db,
            "mod my_module { fn my_function() {} }",
        );
        let item = parser.parse_module().unwrap_or_render();
        match item {
            ItemKind::Module(module) => {
                assert!(
                    module.id(db).path_str(db).ends_with("my_module"),
                    "Expected path to end with 'my_module', got '{}'",
                    module.id(db).path_str(db)
                );
                match module.kind(db) {
                    scrap_ast::module::ModuleKind::Loaded(items, inline, span) => {
                        assert_eq!(*inline, Inline::Yes);
                        assert_eq!(span.to_range(db), 0..37);
                        assert_eq!(items.len(), 1);
                        match &items[0].kind {
                            ItemKind::Fn(fndef) => {
                                assert_eq!(fndef.ident(db).name.text(db), "my_function");
                                assert_eq!(fndef.span(db).to_range(db), 16..35);
                            }
                            _ => panic!("Expected function item inside module"),
                        }
                    }
                    scrap_ast::module::ModuleKind::Unloaded => panic!("Expected loaded module"),
                }
            }
            _ => panic!("Expected module item"),
        }
    }
}
