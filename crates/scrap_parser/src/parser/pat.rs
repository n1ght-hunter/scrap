use crate::PResult;
use scrap_ast::pat::{FieldPat, Pat, PatKind};
use scrap_lexer::Token;
use scrap_span::Span;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_pat_empty(&mut self) -> PResult<'a, Pat<'db>> {
        Ok(Pat {
            id: self.state.new_node_id(),
            kind: PatKind::Missing,
            span: self.token.span,
        })
    }

    pub fn parse_pat(&mut self) -> PResult<'a, Pat<'db>> {
        let ident = self.parse_ident()?;
        Ok(Pat {
            id: self.state.new_node_id(),
            kind: PatKind::Ident(
                scrap_ast::pat::BindingMode(
                    scrap_ast::pat::ByRef::No,
                    scrap_shared::Mutability::Not,
                ),
                ident,
                None,
            ),
            span: self.token.span,
        })
    }

    /// Parse a pattern for match arms.
    ///
    /// Handles:
    /// - `_` → Wildcard
    /// - `Ident::Ident` → Path (unit variant)
    /// - `Ident::Ident(pats)` → TupleStruct
    /// - `Ident::Ident { field_pats }` → Struct
    /// - literal → Lit
    /// - `ident` → Ident (binding)
    pub fn parse_match_pat(&mut self) -> PResult<'a, Pat<'db>> {
        let start = self.token.span.start(self.db);

        // Wildcard: `_`
        if self.check(Token::Ident) && &self.source[self.token.span.to_range(self.db)] == "_" {
            self.bump();
            let end = self.token.span.end(self.db);
            return Ok(Pat {
                id: self.state.new_node_id(),
                kind: PatKind::Wildcard,
                span: Span::new(self.db, start, end),
            });
        }

        // Path-based patterns: `Ident::Ident`, `Ident::Ident(...)`, `Ident::Ident { ... }`
        if self.check(Token::Ident) && self.check_ahead(1, Token::DoubleColon) {
            let path = self.parse_pat_path()?;
            let end = self.token.span.end(self.db);

            // TupleStruct: `Option::Some(x, y)`
            if self.eat(Token::LParen) {
                let mut pats = Vec::new();
                while !self.check(Token::RParen) {
                    pats.push(self.parse_match_pat()?);
                    if !self.eat(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                let end = self.token.span.end(self.db);
                return Ok(Pat {
                    id: self.state.new_node_id(),
                    kind: PatKind::TupleStruct(path, pats),
                    span: Span::new(self.db, start, end),
                });
            }

            // Struct: `Message::Move { x, y }`
            if self.eat(Token::LBrace) {
                let mut field_pats = Vec::new();
                while !self.check(Token::RBrace) {
                    let fp_start = self.token.span.start(self.db);
                    let ident = self.parse_ident()?;
                    let fp_end = self.token.span.end(self.db);
                    // Shorthand: field name = binding name
                    let pat = Pat {
                        id: self.state.new_node_id(),
                        kind: PatKind::Ident(
                            scrap_ast::pat::BindingMode(
                                scrap_ast::pat::ByRef::No,
                                scrap_shared::Mutability::Not,
                            ),
                            ident,
                            None,
                        ),
                        span: Span::new(self.db, fp_start, fp_end),
                    };
                    field_pats.push(FieldPat {
                        ident,
                        pat,
                        span: Span::new(self.db, fp_start, fp_end),
                    });
                    if !self.eat(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RBrace)?;
                let end = self.token.span.end(self.db);
                return Ok(Pat {
                    id: self.state.new_node_id(),
                    kind: PatKind::Struct(path, field_pats),
                    span: Span::new(self.db, start, end),
                });
            }

            // Plain path (unit variant): `Option::None`
            return Ok(Pat {
                id: self.state.new_node_id(),
                kind: PatKind::Path(path),
                span: Span::new(self.db, start, end),
            });
        }

        // Literal patterns: integers, bools, strings
        if self.token.node.is_literal() && !self.check(Token::Ident) {
            let lit = self.parse_lit()?;
            let end = self.token.span.end(self.db);
            return Ok(Pat {
                id: self.state.new_node_id(),
                kind: PatKind::Lit(lit),
                span: Span::new(self.db, start, end),
            });
        }

        // Simple ident binding
        if self.check(Token::Ident) {
            let ident = self.parse_ident()?;
            let end = self.token.span.end(self.db);
            return Ok(Pat {
                id: self.state.new_node_id(),
                kind: PatKind::Ident(
                    scrap_ast::pat::BindingMode(
                        scrap_ast::pat::ByRef::No,
                        scrap_shared::Mutability::Not,
                    ),
                    ident,
                    None,
                ),
                span: Span::new(self.db, start, end),
            });
        }

        Err(self.raise_unexpected_token_error())
    }

    /// Parse a path in a pattern context (uses `=>` / `(` / `{` as terminators).
    fn parse_pat_path(&mut self) -> PResult<'a, scrap_shared::path::Path<'db>> {
        let mut segments = thin_vec::ThinVec::new();

        loop {
            let ident = self.parse_ident()?;
            segments.push(scrap_shared::path::PathSegment {
                id: self.state.new_node_id(),
                ident,
            });

            if !self.eat(Token::DoubleColon) {
                break;
            }
            // Stop if the next token is a terminator (not another ident segment)
            if !self.check(Token::Ident) {
                break;
            }
        }

        let span = Span::new(
            self.db,
            segments.first().unwrap().ident.span.start(self.db),
            segments.last().unwrap().ident.span.end(self.db),
        );

        Ok(scrap_shared::path::Path { span, segments })
    }
}
