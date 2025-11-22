//! Atomic (primary) expression parsing.

use scrap_ast::expr::{Expr, ExprKind};
use scrap_lexer::Token;
use scrap_shared::path::Path;
use scrap_span::Span;

impl<'a, 'db> crate::parser::Parser<'a, 'db> {
    /// Parse an atomic (primary) expression.
    ///
    /// Atomic expressions include literals, paths, blocks, arrays, parenthesized
    /// expressions, if expressions, and return expressions.
    pub fn parse_atom(&mut self) -> crate::PResult<'a, Expr<'db>> {
        let start_pos = self.token.span.start(self.db);
        let kind = self.parse_atom_kind()?;
        let end_pos = self.token.span.end(self.db);

        Ok(Expr {
            id: self.state.new_node_id(),
            kind,
            span: Span::new(self.db, start_pos, end_pos),
        })
    }

    /// Parse the kind of an atomic expression.
    pub fn parse_atom_kind(&mut self) -> crate::PResult<'a, ExprKind<'db>> {
        // Check for block expression
        if self.check(Token::LBrace) {
            let block = self.parse_block()?;
            return Ok(ExprKind::Block(Box::new(block)));
        }

        // Check for array expression
        if self.check(Token::LBracket) {
            return self.parse_array_expr();
        }

        // Check for parenthesized expression
        if self.eat(Token::LParen) {
            let expr = self.parse_expr()?;
            self.expect(Token::RParen)?;
            return Ok(ExprKind::Paren(Box::new(expr)));
        }

        // Check for function call or path
        if self.check_path(Token::LParen) {
            return self.parse_function_call_expr();
        }

        // Check for return expression
        if self.eat(Token::Return) {
            let expr = if !self.check(Token::Semicolon) && !self.check(Token::RBrace) {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };
            return Ok(ExprKind::Return(expr));
        }

        // Check for if expression
        if self.check(Token::If) {
            return self.parse_if_expr();
        }

        // Check for literal or path
        if self.token.node.is_literal() {
            if self.check(Token::Ident) {
                let path = self.parse_path(Token::Eof)?;
                return Ok(ExprKind::Path(path));
            } else {
                let lit = self.parse_lit()?;
                return Ok(ExprKind::Lit(lit));
            }
        }

        Err(self.raise_unexpected_token_error())
    }

    fn parse_array_expr(&mut self) -> crate::PResult<'a, ExprKind<'db>> {
        self.expect(Token::LBracket)?;
        let mut elements = thin_vec::ThinVec::new();

        while !self.check(Token::RBracket) {
            elements.push(Box::new(self.parse_expr()?));
            if !self.eat(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBracket)?;
        Ok(ExprKind::Array(elements))
    }

    fn parse_function_call_expr(&mut self) -> crate::PResult<'a, ExprKind<'db>> {
        let path = self.parse_path(Token::LParen)?;
        self.expect(Token::LParen)?;

        let mut args = thin_vec::ThinVec::new();
        while !self.check(Token::RParen) {
            args.push(Box::new(self.parse_expr()?));
            if !self.eat(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RParen)?;

        let path_expr = Expr {
            id: self.state.new_node_id(),
            span: path.span,
            kind: ExprKind::Path(path),
        };

        Ok(ExprKind::Call(Box::new(path_expr), args))
    }

    fn parse_if_expr(&mut self) -> crate::PResult<'a, ExprKind<'db>> {
        self.expect(Token::If)?;
        let condition = Box::new(self.parse_expr()?);
        let then_block = Box::new(self.parse_block()?);

        let else_block = if self.eat(Token::Else) {
            let else_start = self.token.span.start(self.db);
            let block = self.parse_block()?;
            let else_end = self.token.span.end(self.db);

            Some(Box::new(Expr {
                id: self.state.new_node_id(),
                kind: ExprKind::Block(Box::new(block)),
                span: Span::new(self.db, else_start, else_end),
            }))
        } else {
            None
        };

        Ok(ExprKind::If(condition, then_block, else_block))
    }

    /// Check if the current token starts a path followed by the terminator.
    ///
    /// Returns true if we see `Ident` followed by either `::` or the terminator token.
    pub fn check_path(&mut self, term: Token) -> bool {
        self.check(Token::Ident)
            && (self.check_ahead(1, Token::DoubleColon) || self.check_ahead(1, term))
    }

    /// Parse a path (e.g., `foo`, `foo::bar::baz`).
    ///
    /// Continues parsing segments until the terminator token is reached.
    pub fn parse_path(&mut self, term: Token) -> crate::PResult<'a, Path<'db>> {
        let mut segments = thin_vec::ThinVec::new();

        while !self.check(term) {
            let ident = self.parse_ident()?;
            segments.push(scrap_shared::path::PathSegment {
                id: self.state.new_node_id(),
                ident,
            });

            if !self.eat(Token::DoubleColon) || self.check(term) {
                break;
            }
        }

        if segments.is_empty() {
            return Err(self.unexpected_token_error(&[Token::Ident]));
        }

        let span = Span::new(
            self.db,
            segments.first().unwrap().ident.span.start(self.db),
            segments.last().unwrap().ident.span.end(self.db),
        );

        Ok(Path { span, segments })
    }
}
