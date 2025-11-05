use scrap_ast::Can;
use scrap_ast::Recovered;
use scrap_ast::Visibility;
use scrap_ast::item::Item;
use scrap_ast::node_id::NodeId;
use scrap_diagnostics::AnnotationKind;
use scrap_diagnostics::Level;
use scrap_diagnostics::Snippet;
use scrap_errors::FatalError;
use scrap_lexer::Token;

use scrap_lexer::token_stream::TokenStreamCursor;
use scrap_lexer::token_stream::TokenTypeSet;
use scrap_span::Span;
use scrap_span::Spanned;
use thin_vec::ThinVec;

#[derive(Debug, Clone)]
pub struct State<'a> {
    id: u32,
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
// pub mod enumdef;
// pub mod expr;
// pub mod field;
pub mod fndef;
pub mod ident;
pub mod item;
// pub mod lit;
// pub mod local;
// pub mod operators;
pub mod pat;
// pub mod stmt;
pub mod module;
pub mod structdef;
pub mod ty;
mod utils;
pub mod stmt;
pub mod local;
pub mod expr;

pub struct NewParser<'a> {
    pub(crate) source: &'a str,
    pub(crate) token_stream: TokenStreamCursor,
    expected_token_types: TokenTypeSet,
    pub(crate) token: Spanned<Token>,
    pub(crate) state: State<'a>,
    pub(crate) lasso: lasso::Rodeo,
}

impl<'a> NewParser<'a> {
    pub fn new(source: &'a str, token_stream: TokenStreamCursor, state: State<'a>) -> Self {
        Self {
            token: token_stream
                .curr()
                .unwrap_or_else(|| Spanned::new(Token::dummy(), Span::default())),
            source,
            token_stream,
            expected_token_types: TokenTypeSet::new(),
            state,
            lasso: lasso::Rodeo::default(),
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
    pub fn look_ahead(&mut self, n: usize) -> Option<&Spanned<Token>> {
        self.token_stream.look_ahead(n)
    }

    pub fn bump(&mut self) {
        self.token_stream.bump();
        self.token = self
            .token_stream
            .curr()
            .unwrap_or_else(|| Spanned::new(Token::dummy(), Span::default()))
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

    pub fn parse_can(&mut self) -> crate::PResult<'a, Can> {
        let items = self.parse_module_inner(Token::Eof)?;
        Ok(Can {
            items,
            id: self.state.new_node_id(),
        })
    }

    pub fn parse_visibility(&mut self) -> crate::PResult<'a, Visibility> {
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
}

#[cfg(test)]
pub mod parse_test_utils {
    use scrap_diagnostics::annotate_snippets::Group;
    use scrap_lexer::Logos;
    use scrap_lexer::token_stream::TokenStream;
    use scrap_lexer::token_stream::TokenStreamCursor;

    use super::*;
    use crate::parser::NewParser;
    use crate::parser::State;

    pub fn parse_with<'a>(source: &'a str) -> NewParser<'a> {
        let (token_iter, lex_errs) = scrap_lexer::Token::lexer(source).spanned().fold(
            (Vec::new(), Vec::new()),
            |(mut tokens, mut token_errors), (new_tok, new_span)| {
                let span = Span::from(new_span);
                match new_tok {
                    Ok(new_tok) => tokens.push((new_tok, span)),
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

        let token_stream = TokenStream::new(
            token_iter
                .into_iter()
                .map(|(t, s)| Spanned::new(t, s))
                .collect(),
        );

        let state = State::new("test.sc");
        NewParser::new(source, TokenStreamCursor::new(token_stream), state)
    }

    pub fn test_parser<'a>(source: &'a str, tokens: &'a [Spanned<Token>]) -> NewParser<'a> {
        let token_stream = TokenStreamCursor::new(TokenStream::new(tokens.to_vec()));
        let state = State::new("test.sc");
        NewParser::new(source, token_stream, state)
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
                Err(_) => FatalError.raise(),
            }
        }
    }
}
