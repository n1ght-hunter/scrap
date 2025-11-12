use crate::PResult;
use scrap_ast::fndef::{FnDef, Param};
use scrap_lexer::Token;
use scrap_span::Span;
use thin_vec::ThinVec;

impl<'a> super::Parser<'a> {
    /// Check if the current token is a function definition
    pub fn check_fn_def(&mut self) -> bool {
        self.check(Token::Fn)
    }

    pub fn parse_fn_def(&mut self) -> PResult<'a, FnDef> {
        let start_span = self.token.span;
        self.expect(Token::Fn)?;
        let ident = self.parse_ident()?;
        let params = self.parse_fn_params()?;
        let body = self.parse_block()?;

        Ok(FnDef {
            span: Span::new(start_span.start..body.span.end),
            id: self.state.new_node_id(),
            ident,
            args: params,
            ret_type: None,
            body: body,
        })
    }

    pub fn parse_fn_params(&mut self) -> PResult<'a, ThinVec<Param>> {
        self.expect(Token::LParen)?;
        let mut params = ThinVec::new();

        while !self.check(Token::RParen) {
            let param_ident = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let param_type = self.parse_type()?;

            params.push(Param {
                span: Span::new(param_ident.span.start..param_type.span.end),
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
        let mut parser = parse_with(source);
        let fn_def = parser.parse_fn_def().unwrap_or_render();
        assert_eq!(parser.resolve_symbol(fn_def.ident.name), "my_function");
        assert_eq!(fn_def.ident.span.to_range(), 3..14);
    }

    #[test]
    fn fn_with_params() {
        let source = "fn add(a: i32, b: i32) {}";
        let mut parser = parse_with(source);
        let fn_def = parser.parse_fn_def().unwrap_or_render();
        assert_eq!(parser.resolve_symbol(fn_def.ident.name), "add");
        assert_eq!(fn_def.args.len(), 2);
        assert_eq!(parser.resolve_symbol(fn_def.args[0].ident.name), "a");
        assert_eq!(parser.resolve_symbol(fn_def.args[1].ident.name), "b");
    }
}
