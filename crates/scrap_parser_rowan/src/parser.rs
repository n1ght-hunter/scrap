mod block;
mod enumdef;
mod expr;
mod fndef;
mod item;
mod path;
mod stmt;
mod structdef;
mod ty;

use rowan::{GreenNode, GreenNodeBuilder};
use scrap_lexer::{LexedTokens, Token};
use scrap_span::Spanned;

use crate::syntax_kind::SyntaxKind;

pub struct Parser<'db> {
    tokens: Vec<Spanned<'db, Token>>,
    pos: usize,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<String>,
    db: &'db dyn scrap_shared::Db,
    source: String,
}

impl<'db> Parser<'db> {
    pub fn new(
        db: &'db dyn scrap_shared::Db,
        source: String,
        lexed_tokens: &LexedTokens<'db>,
    ) -> Self {
        let tokens: Vec<_> = lexed_tokens.tokens(db).iter().cloned().collect();
        Parser {
            tokens,
            pos: 0,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
            db,
            source,
        }
    }

    /// Get the current token
    fn current(&self) -> Option<&Spanned<'db, Token>> {
        self.tokens.get(self.pos)
    }

    /// Get the current token kind
    fn current_kind(&self) -> Option<Token> {
        self.current().map(|t| t.node)
    }

    /// Check if we're at a specific token
    fn at(&self, token: Token) -> bool {
        self.current_kind() == Some(token)
    }

    /// Check if we're at EOF
    fn at_eof(&self) -> bool {
        self.current_kind() == Some(Token::Eof) || self.pos >= self.tokens.len()
    }

    /// Start a new syntax node
    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into());
    }

    /// Finish the current syntax node
    fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    /// Consume the current token and add it to the tree
    fn bump(&mut self) {
        if let Some(token) = self.current() {
            let kind: SyntaxKind = token.node.into();
            let start = token.span.start(self.db);
            let end = token.span.end(self.db);

            // Get the text from the original source
            let text = &self.source[start..end];

            self.builder.token(kind.into(), text);
            self.pos += 1;
        }
    }

    /// Consume the current token if it matches, otherwise report an error
    fn expect(&mut self, token: Token) -> bool {
        if self.at(token) {
            self.bump();
            true
        } else {
            self.error(format!(
                "Expected {:?}, found {:?}",
                token,
                self.current_kind()
            ));
            false
        }
    }

    /// Report a parsing error
    fn error(&mut self, message: String) {
        self.errors.push(message);
        // Insert an error node
        self.start_node(SyntaxKind::ERROR);
        if !self.at_eof() {
            self.bump();
        }
        self.finish_node();
    }

    /// Finish parsing and return the green tree
    pub fn finish(self) -> (GreenNode, Vec<String>) {
        (self.builder.finish(), self.errors)
    }

    // === Parsing methods ===

    /// Parse the entire source file
    pub fn parse_source_file(&mut self) {
        self.start_node(SyntaxKind::SOURCE_FILE);

        while !self.at_eof() {
            // Skip trivia at the top level (it's already in the tree)
            if self.current_kind().map_or(false, |k| k.is_trivia()) {
                self.bump();
                continue;
            }

            self.parse_item();
        }

        self.finish_node();
    }
}
