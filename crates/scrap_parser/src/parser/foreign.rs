use scrap_ast::foreign::{ForeignItem, ForeignMod};
use scrap_lexer::Token;
use scrap_span::Span;
use thin_vec::ThinVec;

use crate::PResult;

impl<'a, 'db> super::Parser<'a, 'db> {
    /// Check if the current token starts an extern block
    pub fn check_extern_block(&mut self) -> bool {
        self.check(Token::Extern)
    }

    /// Parse an extern block: `extern "C" { fn foo(...) -> ...; ... }`
    pub fn parse_extern_block(&mut self) -> PResult<'a, ForeignMod> {
        let start_span = self.token.span;
        self.expect(Token::Extern)?;

        // Parse the ABI string literal (e.g. "C")
        let abi_span = self.token.span;
        self.expect(Token::Str)?;
        let abi_text = &self.source[abi_span.start..abi_span.end];
        // Strip quotes from the string literal
        let abi_inner = &abi_text[1..abi_text.len() - 1];
        let abi = scrap_shared::ident::Symbol::new(abi_inner);

        self.expect(Token::LBrace)?;

        let mut items = ThinVec::new();
        while !self.check(Token::RBrace) {
            items.push(self.parse_foreign_item()?);
        }

        let end_span = self.token.span;
        self.expect(Token::RBrace)?;
        let span = Span::new(start_span.start, end_span.end);

        Ok(ForeignMod { abi, items, span })
    }

    /// Parse a single foreign function declaration: `fn foo(a: i32) -> !;`
    fn parse_foreign_item(&mut self) -> PResult<'a, ForeignItem> {
        let start_span = self.token.span;
        self.expect(Token::Fn)?;
        let ident = self.parse_ident()?;
        let params = self.parse_fn_params()?;
        let ret_type = if self.eat(Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let end_span = self.token.span;
        self.expect(Token::Semicolon)?;
        let span = Span::new(start_span.start, end_span.end);

        Ok(ForeignItem {
            id: self.state.new_node_id(),
            ident,
            args: params,
            ret_type,
            span,
        })
    }
}
