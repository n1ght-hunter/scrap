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
    /// All tokens including trivia (used for building the tree)
    tokens: Vec<Spanned<'db, Token>>,
    /// Indices of non-trivia tokens in the tokens array
    filtered_indices: Vec<usize>,
    /// Current position in the original token stream
    pos: usize,
    /// Current position in the filtered token stream
    filtered_pos: usize,
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

        // Build filtered indices - indices of non-trivia tokens
        let filtered_indices: Vec<usize> = tokens
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.node.is_trivia())
            .map(|(i, _)| i)
            .collect();

        Parser {
            tokens,
            filtered_indices,
            pos: 0,
            filtered_pos: 0,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
            db,
            source,
        }
    }

    /// Get the current token from the original stream (may include trivia)
    #[allow(dead_code)]
    fn current(&self) -> Option<&Spanned<'db, Token>> {
        self.tokens.get(self.pos)
    }

    /// Get the current non-trivia token kind for parsing logic
    fn current_kind(&self) -> Option<Token> {
        self.filtered_indices
            .get(self.filtered_pos)
            .and_then(|&idx| self.tokens.get(idx))
            .map(|t| t.node)
    }

    /// Check if we're at a specific token (automatically skips trivia)
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

    /// Create a checkpoint for potentially wrapping nodes later
    pub(super) fn checkpoint(&mut self) -> rowan::Checkpoint {
        self.builder.checkpoint()
    }

    /// Start a node at a previous checkpoint, wrapping all content added since
    pub(super) fn start_node_at(&mut self, checkpoint: rowan::Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind.into());
    }

    /// Consume the current token and add it to the tree
    /// This consumes all trivia up to and including the next non-trivia token
    fn bump(&mut self) {
        // Get the index of the current non-trivia token we want to consume
        if let Some(&target_idx) = self.filtered_indices.get(self.filtered_pos) {
            // Consume all trivia tokens before the target
            while self.pos < target_idx {
                if let Some(token) = self.tokens.get(self.pos) {
                    let kind: SyntaxKind = token.node.into();
                    let start = token.span.start(self.db);
                    let end = token.span.end(self.db);
                    let text = &self.source[start..end];
                    self.builder.token(kind.into(), text);
                }
                self.pos += 1;
            }

            // Consume the target non-trivia token
            if let Some(token) = self.tokens.get(self.pos) {
                let kind: SyntaxKind = token.node.into();
                let start = token.span.start(self.db);
                let end = token.span.end(self.db);
                let text = &self.source[start..end];
                self.builder.token(kind.into(), text);
                self.pos += 1;
            }

            // Advance filtered position
            self.filtered_pos += 1;
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

    /// Get the token kind at offset n from current position (0 = current token)
    /// Automatically skips trivia since we use filtered indices
    pub(super) fn nth(&self, n: usize) -> Option<Token> {
        self.filtered_indices
            .get(self.filtered_pos + n)
            .and_then(|&idx| self.tokens.get(idx))
            .map(|t| t.node)
    }

    /// Check if we're at a specific token at offset n (0 = current)
    pub(super) fn nth_at(&self, n: usize, token: Token) -> bool {
        self.nth(n) == Some(token)
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
            self.parse_item();
        }

        // Consume any remaining trivia at the end of the file
        while self.pos < self.tokens.len() {
            if let Some(token) = self.tokens.get(self.pos) {
                let kind: SyntaxKind = token.node.into();
                let start = token.span.start(self.db);
                let end = token.span.end(self.db);
                let text = &self.source[start..end];
                self.builder.token(kind.into(), text);
            }
            self.pos += 1;
        }

        self.finish_node();
    }
}
