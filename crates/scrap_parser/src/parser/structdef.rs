use scrap_ast::{field::FieldDef, structdef::StructDef};
use scrap_lexer::Token;
use scrap_span::Span;

use crate::PResult;

impl<'a> super::Parser<'a> {
    pub fn check_struct_def(&mut self) -> bool {
        self.check(Token::Struct)
    }

    pub fn parse_struct_def(&mut self) -> PResult<'a, StructDef> {
        self.expect(Token::Struct)?;
        let ident = self.parse_ident()?;
        
        let var_data = self.parse_variant_data(Token::Semicolon)?;

       
        Ok(scrap_ast::structdef::StructDef {
            id: self.state.new_node_id(),
            ident,
            data: var_data,
        })
    }

    pub fn parse_variant_data(
        &mut self,
        term: Token,
    ) -> PResult<'a, scrap_ast::enumdef::VariantData> {
        if self.eat(Token::LBrace) {
            let mut fields = thin_vec::ThinVec::new();
            while !self.check(Token::RBrace) {
                let vis = self.parse_visibility()?;
                let field_ident = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let field_type = self.parse_type()?;
                fields.push(FieldDef {
                    id: self.state.new_node_id(),
                    span: Span::new(field_ident.span.start..field_type.span.end),
                    vis,
                    ident: Some(field_ident),
                    ty: Box::new(field_type),
                });
                if !self.eat(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::RBrace)?;
            Ok(scrap_ast::enumdef::VariantData::Struct { fields })
        } else if self.eat(Token::LParen) {
            let mut fields = thin_vec::ThinVec::new();
            while !self.check(Token::RParen) {
                let vis = self.parse_visibility()?;
                let field_type = self.parse_type()?;
                fields.push(FieldDef {
                    id: self.state.new_node_id(),
                    span: field_type.span,
                    vis,
                    ident: None,
                    ty: Box::new(field_type),
                });
                if !self.eat(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::RParen)?;
            Ok(scrap_ast::enumdef::VariantData::Tuple(fields, self.state.new_node_id()))
        } else if self.check(term) {
            Ok(scrap_ast::enumdef::VariantData::Unit(self.state.new_node_id()))
        } else {
            Err(self.unexpected_token_error(&[term, Token::LBrace, Token::LParen]))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_test_utils::{ExtendRes, parse_with};

    use super::*;

    #[test]
    fn test_parse_struct_def() {
        let mut parser = parse_with("struct MyStruct { pub field1: i32, field2: bool }");
        let struct_def = parser.parse_struct_def().unwrap_or_render();
        assert_eq!(parser.resolve_symbol(struct_def.ident.name), "MyStruct");
        let data = struct_def.data.unwrap_struct();
        assert_eq!(data.len(), 2);
        assert_eq!(parser.resolve_symbol(data[0].ident.as_ref().unwrap().name), "field1");
        assert_eq!(parser.resolve_symbol(data[1].ident.as_ref().unwrap().name), "field2");
    }

    #[test]
    #[should_panic]
    fn test_parse_struct_def_missing_brace() {
        let mut parser = parse_with("struct MyStruct { field1: i32, field2: bool ");
        parser.parse_struct_def().should_panic();
    }

    #[test]
    #[should_panic]
    fn test_parse_struct_def_missing_colon() {
        let mut parser = parse_with("struct MyStruct,");
        parser.parse_struct_def().unwrap_or_render();
    }
}
