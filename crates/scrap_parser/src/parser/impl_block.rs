use crate::PResult;
use scrap_ast::fndef::{FnDef, Param};
use scrap_ast::impl_block::ImplBlock;
use scrap_ast::typedef::{Ty, TyKind};
use scrap_lexer::Token;
use scrap_shared::path::Path;
use scrap_span::Span;
use thin_vec::ThinVec;

impl<'a, 'db> super::Parser<'a, 'db> {
    /// Check if the current token starts an impl block.
    pub fn check_impl_block(&mut self) -> bool {
        self.check(Token::Impl)
    }

    /// Parse an `impl TypeName { fn method(...) { ... } ... }` block.
    pub fn parse_impl_block(&mut self) -> PResult<'a, ImplBlock<'db>> {
        let start_span = self.token.span;
        self.expect(Token::Impl)?;
        let type_name = self.parse_ident()?;
        self.expect(Token::LBrace)?;

        let mut methods = Vec::new();
        while !self.check(Token::RBrace) {
            methods.push(self.parse_method_def(&type_name)?);
        }

        let end_span = self.token.span;
        self.expect(Token::RBrace)?;

        Ok(ImplBlock {
            id: self.state.new_node_id(),
            type_name,
            methods,
            span: Span::new(self.db, start_span.start(self.db), end_span.end(self.db)),
        })
    }

    /// Parse a method definition inside an impl block.
    /// Methods look like regular functions but their first param can be
    /// `self`, `&self`, or `&mut self`.
    fn parse_method_def(
        &mut self,
        type_name: &scrap_shared::ident::Ident<'db>,
    ) -> PResult<'a, FnDef<'db>> {
        let start_span = self.token.span;
        self.expect(Token::Fn)?;
        let ident = self.parse_ident()?;
        let params = self.parse_method_params(type_name)?;
        let ret_type = if self.eat(Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        let span = Span::new(self.db, start_span.start(self.db), body.span.end(self.db));

        Ok(FnDef::new(
            self.db,
            self.state.new_node_id(),
            ident,
            params,
            ret_type,
            body,
            span,
        ))
    }

    /// Parse method parameters, handling `self`, `&self`, and `&mut self` as the first parameter.
    /// All three forms desugar to a parameter of the bare struct type (pass-by-value).
    fn parse_method_params(
        &mut self,
        type_name: &scrap_shared::ident::Ident<'db>,
    ) -> PResult<'a, ThinVec<Param<'db>>> {
        self.expect(Token::LParen)?;
        let mut params = ThinVec::new();

        // Check for self parameter as the first param
        if !self.check(Token::RParen)
            && let Some(self_param) = self.try_parse_self_param(type_name)?
        {
            params.push(self_param);
            let _ = self.eat(Token::Comma);
        }

        // Parse remaining regular parameters
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
                pat: Box::new(self.parse_pat_empty()?),
            });

            if !self.eat(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RParen)?;
        Ok(params)
    }

    /// Try to parse a self parameter. Returns Some(Param) if the current position
    /// starts with `self`, `&self`, or `&mut self`.
    /// All forms produce a Param with type = TypeName (by-value).
    fn try_parse_self_param(
        &mut self,
        type_name: &scrap_shared::ident::Ident<'db>,
    ) -> PResult<'a, Option<Param<'db>>> {
        let start_pos = self.position();

        // Case 1: `self` (bare identifier)
        if self.check(Token::Ident) {
            let text = &self.source[self.token.span.to_range(self.db)];
            if text == "self" {
                // Check next is `,` or `)` (confirming this is a self param, not `self: Type`)
                if self.check_ahead(1, Token::Comma) || self.check_ahead(1, Token::RParen) {
                    let self_ident = self.parse_ident()?;
                    return Ok(Some(self.make_self_param(self_ident, type_name)));
                }
            }
        }

        // Case 2: `&self` or `&mut self`
        if self.check(Token::BitAnd) {
            // Copy lookahead info to avoid borrow conflicts
            let ahead1_info = self.look_ahead(1).map(|t| (t.node, t.span));
            if let Some((ahead1_tok, ahead1_span)) = ahead1_info {
                let ahead1_text = &self.source[ahead1_span.to_range(self.db)];

                if ahead1_tok == Token::Ident && ahead1_text == "self" {
                    // `&self` — consume & and self
                    self.bump(); // consume &
                    let self_ident = self.parse_ident()?;
                    return Ok(Some(self.make_self_param(self_ident, type_name)));
                }

                if ahead1_tok == Token::Ident && ahead1_text == "mut" {
                    // Check if it's `&mut self`
                    let ahead2_info = self.look_ahead(2).map(|t| (t.node, t.span));
                    if let Some((ahead2_tok, ahead2_span)) = ahead2_info {
                        let ahead2_text = &self.source[ahead2_span.to_range(self.db)];
                        if ahead2_tok == Token::Ident && ahead2_text == "self" {
                            // `&mut self` — consume &, mut, self
                            self.bump(); // consume &
                            self.bump(); // consume mut
                            let self_ident = self.parse_ident()?;
                            return Ok(Some(self.make_self_param(self_ident, type_name)));
                        }
                    }
                }
            }
        }

        // Not a self parameter — restore position
        self.set_position(start_pos);
        Ok(None)
    }

    /// Create a self parameter with the bare struct type.
    fn make_self_param(
        &mut self,
        self_ident: scrap_shared::ident::Ident<'db>,
        type_name: &scrap_shared::ident::Ident<'db>,
    ) -> Param<'db> {
        let ty = Ty {
            id: self.state.new_node_id(),
            span: type_name.span,
            kind: TyKind::Path(Path::from_ident(*type_name)),
        };
        Param {
            span: self_ident.span,
            id: self.state.new_node_id(),
            ident: self_ident,
            ty: Box::new(ty),
            pat: Box::new(self.parse_pat_empty().unwrap()),
        }
    }
}
