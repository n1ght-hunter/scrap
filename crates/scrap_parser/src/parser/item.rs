use scrap_ast::item::{Item, ItemKind, ItemKindDiscriminants};
use scrap_diagnostics::{AnnotationKind, Level, Snippet};
use scrap_span::Span;
use strum::IntoEnumIterator;

impl<'a> super::Parser<'a> {
    pub fn parse_item(&mut self) -> crate::PResult<'a, Box<Item>> {
        let start_span = self.token.span;
        let item = self.parse_item_kind()?;
        let span = Span::new(start_span.start..self.token.span.end);
        let id = self.state.new_node_id();
        Ok(Box::new(Item {
            kind: item,
            span,
            id,
        }))
    }

    pub fn parse_item_kind(&mut self) -> crate::PResult<'a, ItemKind> {
        if self.check_fn_def() {
            return Ok(ItemKind::Fn(self.parse_fn_def()?));
        }
        if self.check_module() {
            return self.parse_module();
        }
        if self.check_struct_def() {
            return Ok(ItemKind::Struct(self.parse_struct_def()?));
        }

        Err(Level::ERROR
            .primary_title("expected a top-level item")
            .element(
                Snippet::source(self.source)
                    .path(self.state.file_name)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.token.span.to_range())
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
}

#[cfg(test)]
mod tests {
    use scrap_ast::module::{Inline, Module};

    use crate::parser::parse_test_utils::{ExtendRes, parse_with};

    use super::*;

    #[test]
    fn test_parse_item_fn() {
        let mut parser = parse_with("fn my_function() {}");
        let item = parser.parse_item().unwrap_or_render();
        match item.kind {
            ItemKind::Fn(fndef) => {
                assert_eq!(parser.resolve_symbol(fndef.ident.name), "my_function");
                assert_eq!(fndef.span.to_range(), 0..19);
            }
            _ => panic!("Expected function item"),
        }
    }

    #[test]
    #[should_panic]
    fn test_parse_item_invalid() {
        let mut parser = parse_with("invalid_item");
        let _item = parser.parse_item().unwrap_or_render();
    }

    #[test]
    fn test_parse_item_struct() {
        let mut parser = parse_with("struct MyStruct { field1: i32, field2: String }");
        let item = parser.parse_item().unwrap_or_render();
        match item.kind {
            ItemKind::Struct(struct_def) => {
                assert_eq!(parser.resolve_symbol(struct_def.ident.name), "MyStruct");
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_item_module() {
        let mut parser = parse_with("mod my_module { }");
        let item = parser.parse_item().unwrap_or_render();
        match item.kind {
            ItemKind::Module(ident, module) => {
                assert_eq!(parser.resolve_symbol(ident.name), "my_module");
                match module {
                    Module::Loaded(_, inline, span) => {
                        assert_eq!(inline, Inline::Yes);
                        assert_eq!(span.to_range(), 0..17);
                    }
                    _ => panic!("Expected loaded module"),
                }
            }
            _ => panic!("Expected module item"),
        }
        let mut parser = parse_with("mod my_module;");
        let item = parser.parse_item().unwrap_or_render();
        match item.kind {
            ItemKind::Module(ident, module) => {
                assert_eq!(parser.resolve_symbol(ident.name), "my_module");
                match module {
                    Module::Unloaded => {}
                    _ => panic!("Expected loaded module"),
                }
            }
            _ => panic!("Expected module item"),
        }
    }
}
