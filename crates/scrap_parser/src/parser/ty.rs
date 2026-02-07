use crate::PResult;
use scrap_ast::typedef::{Ty, TyKind};
use scrap_lexer::Token;
use scrap_shared::path::Path;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_type(&mut self) -> PResult<'a, Ty<'db>> {
        // Never type: `!`
        if self.check(Token::Bang) {
            let span = self.token.span;
            self.bump();
            return Ok(Ty {
                id: self.state.new_node_id(),
                span,
                kind: TyKind::Never,
            });
        }

        // Identifier types (i32, bool, MyStruct, etc.)
        let ident = self.parse_ident()?;
        Ok(Ty {
            id: self.state.new_node_id(),
            span: ident.span,
            kind: TyKind::Path(Path::from_ident(ident)),
        })
    }
}
