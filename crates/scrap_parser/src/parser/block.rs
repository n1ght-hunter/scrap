use scrap_ast::block::Block;
use scrap_lexer::Token;
use scrap_span::Span;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_block(&mut self) -> crate::PResult<'a, Block<'db>> {
        let start_span = self.token.span;
        self.expect(Token::LBrace)?;
        let mut stmts = thin_vec::ThinVec::new();

        while !self.check(Token::RBrace) {
            let stamt = self.parse_stmt()?;
            stmts.push(stamt);
        }

        let end_span = self.token.span;
        self.expect(Token::RBrace)?;

        Ok(Block {
            id: self.state.new_node_id(),
            stmts,
            span: Span::new(start_span.start, end_span.end),
        })
    }
}
