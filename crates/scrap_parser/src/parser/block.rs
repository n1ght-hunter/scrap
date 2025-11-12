use scrap_ast::block::Block;
use scrap_lexer::Token;
use scrap_span::Span;

impl<'a> super::Parser<'a> {
    pub fn parse_block(&mut self) -> crate::PResult<'a, Block> {
        let start_span = self.token.span;
        self.expect(Token::LBrace)?;


        let end_span = self.token.span;
        self.expect(Token::RBrace)?;

        Ok(Block {
            id: self.state.new_node_id(),
            stmts: thin_vec::ThinVec::new(),
            span: Span::new(start_span.start..end_span.end),
        })
    }
}
