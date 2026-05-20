use crate::PResult;
use scrap_ast::fndef::{FnDef, Param};
use scrap_lexer::Token;
use scrap_span::Span;
use thin_vec::ThinVec;

impl<'a, 'db> super::Parser<'a, 'db> {
    /// Check if the current token is a function definition
    pub fn check_fn_def(&mut self) -> bool {
        self.check(Token::Fn)
    }

    pub fn parse_fn_def(&mut self) -> PResult<'a, FnDef<'db>> {
        let start_span = self.token.span;
        self.expect(Token::Fn)?;
        let ident = self.parse_ident()?;
        let generics = self.parse_generics()?;
        let params = self.parse_fn_params()?;
        let ret_type = if self.eat(Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        let span = Span::new(start_span.start, body.span.end);

        Ok(FnDef::new(
            self.db,
            self.state.new_node_id(),
            ident,
            generics,
            params,
            ret_type,
            body,
            span,
        ))
    }

    pub fn parse_fn_params(&mut self) -> PResult<'a, ThinVec<Param>> {
        self.expect(Token::LParen)?;
        let mut params = ThinVec::new();

        while !self.check(Token::RParen) {
            let param_ident = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let param_type = self.parse_type()?;

            params.push(Param {
                span: Span::new(param_ident.span.start, param_type.span.end),
                id: self.state.new_node_id(),
                ident: param_ident,
                ty: Box::new(param_type),
                pat: Box::new(self.parse_pat_empty()?),
            });

            if !self.eat(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RParen)?;

        Ok(params)
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::parse_test_utils::{ExtendRes, parse_with};

    #[scrap_macros::salsa_test]
    fn empty_fn(db: &dyn scrap_shared::Db) {
        let source = "fn my_function() {}";
        let mut parser = parse_with(db, source);
        let fn_def = parser.parse_fn_def().unwrap_or_render();
        assert_eq!(fn_def.ident(db).name.text(), "my_function");
        assert_eq!(fn_def.ident(db).span.range(), 3..14);
    }

    #[scrap_macros::salsa_test]
    fn fn_with_params(db: &dyn scrap_shared::Db) {
        let source = "fn add(a: i32, b: i32) {}";
        let mut parser = parse_with(db, source);
        let fn_def = parser.parse_fn_def().unwrap_or_render();
        assert_eq!(fn_def.ident(db).name.text(), "add");
        assert_eq!(fn_def.args(db).len(), 2);
        assert_eq!(fn_def.args(db)[0].ident.name.text(), "a");
        assert_eq!(fn_def.args(db)[1].ident.name.text(), "b");
    }
}
