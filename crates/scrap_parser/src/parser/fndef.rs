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
        let params = self.parse_fn_params()?;
        let body = self.parse_block()?;
        let span = Span::new(self.db, start_span.start(self.db), body.span.end(self.db));

        Ok(FnDef::new(
            self.db,
            self.state.new_node_id(),
            ident,
            params,
            None,
            body,
            span,
        ))
    }

    pub fn parse_fn_params(&mut self) -> PResult<'a, ThinVec<Param<'db>>> {
        self.expect(Token::LParen)?;
        let mut params = ThinVec::new();

        while !self.check(Token::RParen) {
            let param_ident = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let param_type = self.parse_type()?;

            params.push(Param {
                span: Span::new(
                    self.db,
                    param_ident.span.start(self.db),
                    param_type.span.end(self.db),
                ),
                id: self.state.new_node_id(),
                ident: param_ident,
                ty: Box::new(param_type),
                pat: Box::new(self.parse_pat()?),
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

    #[test]
    fn empty_fn() {
        let source = "fn my_function() {}";
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = parse_with(&db, source);
        let fn_def = parser.parse_fn_def().unwrap_or_render();
        assert_eq!(fn_def.ident(&db).name.text(&db), "my_function");
        assert_eq!(fn_def.ident(&db).span.to_range(&db), 3..14);
    }

    #[test]
    fn fn_with_params() {
        let source = "fn add(a: i32, b: i32) {}";
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = parse_with(&db, source);
        let fn_def = parser.parse_fn_def().unwrap_or_render();
        assert_eq!(fn_def.ident(&db).name.text(&db), "add");
        assert_eq!(fn_def.args(&db).len(), 2);
        assert_eq!(fn_def.args(&db)[0].ident.name.text(&db), "a");
        assert_eq!(fn_def.args(&db)[1].ident.name.text(&db), "b");
    }
}
