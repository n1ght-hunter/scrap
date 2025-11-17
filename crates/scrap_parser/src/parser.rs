use std::cell::RefCell;
use std::rc::Rc;

use scrap_ast::Can;
use scrap_ast::Recovered;
use scrap_ast::Visibility;
use scrap_ast::ident::Ident;
use scrap_ast::path::Path;
use scrap_lexer::Token;
use scrap_shared::NodeId;

use scrap_lexer::token_stream::TokenStreamCursor;
use scrap_lexer::token_stream::TokenTypeSet;
use scrap_span::Span;
use scrap_span::Spanned;

#[derive(Debug, Clone)]
pub struct State<'a> {
    id: u16,
    file_hash: u64,
    pub(crate) file_name: &'a str,
}

impl<'a> State<'a> {
    pub fn new(file_name: &'a str) -> Self {
        let file_hash = wyhash::wyhash(file_name.as_bytes(), 0);
        Self {
            id: 0,
            file_hash,
            file_name,
        }
    }

    pub fn new_node_id(&mut self) -> NodeId {
        let id = self.id;
        self.id += 1;
        NodeId::new(id, self.file_hash)
    }
}

pub mod block;
pub mod enumdef;
pub mod expr;
pub mod fndef;
pub mod ident;
pub mod item;
pub mod lit;
pub mod local;
pub mod module;
pub mod pat;
pub mod stmt;
pub mod structdef;
pub mod ty;

pub struct Parser<'a, 'db> {
    pub(crate) source: &'a str,
    pub(crate) token_stream: TokenStreamCursor<'db>,
    pub(super) expected_token_types: TokenTypeSet,
    pub(crate) token: Spanned<'db, Token>,
    pub(crate) state: State<'a>,
    pub(crate) db: &'db dyn scrap_shared::Db,
    pub(crate) current_module_path: Rc<RefCell<Path<'db>>>,
}

impl<'a, 'db> Parser<'a, 'db> {
    pub fn new(
        db: &'db dyn scrap_shared::Db,
        source: &'a str,
        token_stream: TokenStreamCursor<'db>,
        state: State<'a>,
        name: Ident<'db>,
    ) -> Self {
        Self {
            token: token_stream
                .curr()
                .unwrap_or_else(|| Spanned::new(Token::dummy(), Span::new_default(db))),
            source,
            token_stream,
            expected_token_types: TokenTypeSet::new(),
            state,
            db,
            current_module_path: Rc::new(RefCell::new(Path::from_ident(name))),
        }
    }

    #[inline]
    pub fn check(&mut self, expected: Token) -> bool {
        let is_present = self.token.node == expected;
        if !is_present {
            self.expected_token_types.insert(expected);
        }
        is_present
    }

    #[inline]
    pub fn check_ahead(&mut self, n: usize, expected: Token) -> bool {
        let lookahead_token = self.token_stream.look_ahead(n);
        match lookahead_token {
            Some(tok) => {
                let is_present = tok.node == expected;
                if !is_present {
                    self.expected_token_types.insert(expected);
                }
                is_present
            }
            None => false,
        }
    }

    #[inline]
    pub fn look_ahead(&mut self, n: usize) -> Option<&Spanned<'db, Token>> {
        self.token_stream.look_ahead(n)
    }

    pub fn bump(&mut self) {
        self.token_stream.bump();
        self.token = self
            .token_stream
            .curr()
            .unwrap_or_else(|| Spanned::new(Token::dummy(), Span::new_default(self.db)))
    }

    #[inline]
    #[must_use]
    pub fn eat(&mut self, expected: Token) -> bool {
        let is_present = self.check(expected);
        if is_present {
            self.bump()
        }
        is_present
    }

    /// Expects and consumes the token `t`. Signals an error if the next token is not `t`.
    pub fn expect(&mut self, expected: Token) -> crate::PResult<'a, Recovered> {
        if self.expected_token_types.is_empty() {
            if self.token.node == expected {
                self.bump();
                Ok(Recovered::No)
            } else {
                Err(self.unexpected_token_error(&[expected]))
                // self.unexpected_try_recover(&exp.tok)
            }
        } else {
            return self.expect_one_of(&[expected], &[]);
        }
    }

    /// Expect next token to be edible or inedible token. If edible,
    /// then consume it; if inedible, then return without consuming
    /// anything. Signal a fatal error if next token is unexpected.
    fn expect_one_of(
        &mut self,
        edible: &[Token],
        inedible: &[Token],
    ) -> crate::PResult<'a, Recovered> {
        if edible.contains(&self.token.node) {
            self.bump();
            Ok(Recovered::No)
        } else if inedible.contains(&self.token.node) {
            // leave it in the input
            Ok(Recovered::No)
        // } else if *self.token != Token::Eof
        //     && self.last_unexpected_token_span == Some(self.token.span)
        // {
        //     FatalError.raise();
        } else {
            Err(self.unexpected_token_error(edible))
        }
        // if edible.iter().any(|node| *node == *self.token) {
        //     self.bump();
        //     Ok(Recovered::No)
        // } else if inedible.iter().any(|node| *node == *self.token) {
        //     // leave it in the input
        //     Ok(Recovered::No)
        // } else if *self.token != Token::Eof
        //     && self.last_unexpected_token_span == Some(self.token.span)
        // {
        //     FatalError.raise();
        // } else {
        //     self.expected_one_of_not_found(edible, inedible)
        //         .map(|error_guaranteed| Recovered::Yes(error_guaranteed))
        // }
    }

    fn position(&self) -> usize {
        self.token_stream.position()
    }

    fn set_position(&mut self, pos: usize) {
        self.token_stream.set_position(pos);
        self.token = self
            .token_stream
            .curr()
            .unwrap_or_else(|| Spanned::new(Token::dummy(), Span::new_default(self.db)));
    }

    pub fn parse_can(&mut self) -> crate::PResult<'a, Can<'db>> {
        let id = self.state.new_node_id();
        let items = self.parse_module_inner(Token::Eof)?;
        Ok(Can { items, id })
    }

    pub fn parse_visibility(&mut self) -> crate::PResult<'a, Visibility<'db>> {
        if self.eat(Token::Pub) {
            Ok(Visibility {
                kind: scrap_ast::VisibilityKind::Public,
                span: self.token.span,
            })
            // TODO: handle `pub(crate)` and other visibility modifiers
        } else {
            Ok(Visibility {
                kind: scrap_ast::VisibilityKind::Inherited,
                // this might be wrong
                span: self.token.span,
            })
        }
    }

    pub fn push_current_module_path(&mut self, ident: Ident<'db>) {
        let new_path = self.current_module_path.borrow().extend(self.db, ident);
        *self.current_module_path.borrow_mut() = new_path;
    }

    pub fn guard_current_module_path(&mut self, ident: Ident<'db>) -> PopOnDrop<'db> {
        self.push_current_module_path(ident);
        PopOnDrop(Rc::clone(&self.current_module_path))
    }

    pub fn pop_current_module_path(&mut self) {
        let segments = &mut self.current_module_path.borrow_mut().segments;
        segments.pop();
    }

    pub fn current_module_path(&self) -> std::cell::Ref<'_, scrap_ast::path::Path<'db>> {
        self.current_module_path.borrow()
    }
}

pub struct PopOnDrop<'db>(std::rc::Rc<std::cell::RefCell<scrap_ast::path::Path<'db>>>);

impl Drop for PopOnDrop<'_> {
    fn drop(&mut self) {
        let segments = &mut self.0.borrow_mut().segments;
        segments.pop();
    }
}

#[cfg(test)]
pub mod parse_test_utils {
    use scrap_diagnostics::annotate_snippets::Group;
    use scrap_lexer::Logos;
    use scrap_lexer::token_stream::TokenStream;
    use scrap_lexer::token_stream::TokenStreamCursor;

    use super::*;
    use crate::parser::Parser;
    use crate::parser::State;

    pub fn parse_with<'a, 'db>(db: &'db dyn scrap_shared::Db, source: &'a str) -> Parser<'a, 'db> {
        let (token_iter, lex_errs) = scrap_lexer::Token::lexer(source).spanned().fold(
            (Vec::new(), Vec::new()),
            |(mut tokens, mut token_errors), (new_tok, new_span)| {
                let span = Span::new(db, new_span.start, new_span.end);
                match new_tok {
                    Ok(new_tok) => tokens.push(Spanned::new(new_tok, span)),
                    Err(e) => {
                        token_errors.push(e);
                    }
                }
                (tokens, token_errors)
            },
        );

        if !lex_errs.is_empty() {
            lex_errs.into_iter().for_each(|e| {
                println!("Lexing error: {:?}", e);
            });
            panic!("Lexing errors encountered");
        }

        test_parser(db, source, token_iter)
    }

    pub fn test_parser<'a, 'db>(
        db: &'db dyn scrap_shared::Db,
        source: &'a str,
        tokens: Vec<Spanned<'db, Token>>,
    ) -> Parser<'a, 'db> {
        let token_stream = TokenStreamCursor::new(TokenStream::new(tokens));
        let state = State::new("test.sc");
        Parser::new(
            db,
            source,
            token_stream,
            state,
            Ident::dummy_with_name(db, "test"),
        )
    }

    pub fn render(report: &[Group]) -> ! {
        scrap_diagnostics::DiagnosticEmitter::new().render(report);
        scrap_errors::FatalError.raise()
    }

    pub trait ExtendRes<T> {
        fn unwrap_or_render(self) -> T;
        fn should_panic(self) -> T;
    }

    impl<'a, T> ExtendRes<T> for crate::PResult<'a, T> {
        fn unwrap_or_render(self) -> T {
            match self {
                Ok(v) => v,
                Err(report) => render(&[report]),
            }
        }

        fn should_panic(self) -> T {
            match self {
                Ok(v) => v,
                Err(_) => scrap_errors::FatalError.raise(),
            }
        }
    }
}
