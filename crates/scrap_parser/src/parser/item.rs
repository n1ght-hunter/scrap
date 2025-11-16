use scrap_ast::{
    item::{Item, ItemKind, ItemKindDiscriminants, UseTree, UseTreeKind},
    path::{Path, PathSegment},
};
use scrap_diagnostics::{AnnotationKind, Level, Snippet};
use scrap_lexer::Token;
use scrap_span::Span;
use strum::IntoEnumIterator;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_item(&mut self) -> crate::PResult<'a, Box<Item<'db>>> {
        let start_span = self.token.span;
        let vis = self.parse_visibility()?;
        let item = self.parse_item_kind()?;
        let span = Span::new(
            self.db,
            start_span.start(self.db),
            self.token.span.end(self.db),
        );
        let id = self.state.new_node_id();
        Ok(Box::new(Item {
            kind: item,
            span,
            vis,
            id,
        }))
    }

    pub fn parse_item_kind(&mut self) -> crate::PResult<'a, ItemKind<'db>> {
        if self.check_fn_def() {
            return Ok(ItemKind::Fn(self.parse_fn_def()?));
        }
        if self.check_module() {
            return self.parse_module();
        }
        if self.check_struct_def() {
            return Ok(ItemKind::Struct(self.parse_struct_def()?));
        }
        if self.check_enum_def() {
            return Ok(ItemKind::Enum(self.parse_enum_def()?));
        }

        if self.check(Token::Use) {
            let path = self.parse_use_item()?;
            return Ok(ItemKind::Use(path));
        }

        Err(Level::ERROR
            .primary_title("expected a top-level item")
            .element(
                Snippet::source(self.source)
                    .path(self.state.file_name)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.token.span.to_range(self.db))
                            .label(format!(
                                "expected one of {} found {}",
                                ItemKindDiscriminants::iter()
                                    .map(|d| format!("{:?}", d))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                self.token.node
                            )),
                    ),
            ))
    }

    pub fn parse_use_item(&mut self) -> crate::PResult<'a, UseTree<'db>> {
        let use_span = self.token.span;
        self.expect(Token::Use)?;
        let mut segments = thin_vec::ThinVec::new();

        while !self.check(Token::Semicolon) {
            let ident = self.parse_ident()?;
            segments.push(PathSegment {
                id: self.state.new_node_id(),
                ident,
            });

            if self.eat(Token::Semicolon) || !self.eat(Token::DoubleColon) {
                break;
            }
        }
        let path = Path {
            segments,
            span: Span::new(
                self.db,
                use_span.start(self.db),
                self.token.span.end(self.db),
            ),
        };
        Ok(UseTree {
            prefix: path,
            kind: UseTreeKind::Simple(None),
            span: Span::new(
                self.db,
                use_span.start(self.db),
                self.token.span.end(self.db),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use scrap_ast::module::{Inline, Module};

    use crate::parser::parse_test_utils::ExtendRes;

    use super::*;

    #[test]
    fn test_parse_item_fn() {
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = crate::parser::parse_test_utils::parse_with(&db, "fn my_function() {}");
        let item = parser.parse_item().unwrap_or_render();
        match item.kind {
            ItemKind::Fn(fndef) => {
                assert_eq!(fndef.ident(&db).name.text(&db), "my_function");
                assert_eq!(fndef.span(&db).to_range(&db), 0..19);
            }
            _ => panic!("Expected function item"),
        }
    }

    #[test]
    #[should_panic]
    fn test_parse_item_invalid() {
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = crate::parser::parse_test_utils::parse_with(&db, "invalid_item");
        let _item = parser.parse_item().unwrap_or_render();
    }

    #[test]
    fn test_parse_item_struct() {
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = crate::parser::parse_test_utils::parse_with(
            &db,
            "struct MyStruct { field1: i32, field2: String }",
        );
        let item = parser.parse_item().unwrap_or_render();
        match item.kind {
            ItemKind::Struct(struct_def) => {
                assert_eq!(struct_def.ident.name.text(&db), "MyStruct");
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_item_module_loaded() {
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = crate::parser::parse_test_utils::parse_with(&db, "mod my_module { }");
        let item = parser.parse_item().unwrap_or_render();
        match item.kind {
            ItemKind::Module(ident, module) => {
                assert_eq!(ident.segments.last().unwrap().ident.name.text(&db), "my_module");
                match module {
                    Module::Loaded(_, inline, span) => {
                        assert_eq!(inline, Inline::Yes);
                        assert_eq!(span.to_range(&db), 0..17);
                    }
                    _ => panic!("Expected loaded module"),
                }
            }
            _ => panic!("Expected module item"),
        }
    }

    #[test]
    fn test_parse_item_module_unloaded() {
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = crate::parser::parse_test_utils::parse_with(&db, "mod my_module;");
        let item = parser.parse_item().unwrap_or_render();
        match item.kind {
            ItemKind::Module(ident, module) => {
                assert_eq!(ident.segments.last().unwrap().ident.name.text(&db), "my_module");
                match module {
                    Module::Unloaded => {}
                    _ => panic!("Expected loaded module"),
                }
            }
            _ => panic!("Expected module item"),
        }
    }
}
