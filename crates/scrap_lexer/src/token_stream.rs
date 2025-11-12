use std::{ops::Deref, sync::Arc};

use scrap_span::Spanned;

use crate::Token;

/// A bitset type designed specifically for `Parser::expected_token_types`,
/// which is very hot. `u128` is the smallest integer that will fit every
/// `TokenType` value.
#[derive(Clone, Copy)]
pub struct TokenTypeSet(u128);

impl TokenTypeSet {
    pub fn new() -> TokenTypeSet {
        TokenTypeSet(0)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn insert(&mut self, token_type: Token) {
        self.0 = self.0 | (1u128 << token_type as u32)
    }

    pub fn clear(&mut self) {
        self.0 = 0
    }

    pub fn contains(&self, token_type: Token) -> bool {
        self.0 & (1u128 << token_type as u32) != 0
    }

    pub fn iter(&self) -> TokenTypeSetIter {
        TokenTypeSetIter(*self)
    }
}

// The `TokenTypeSet` is a copy of the set being iterated. It initially holds
// the entire set. Each bit is cleared as it is returned. We have finished once
// it is all zeroes.
pub struct TokenTypeSetIter(TokenTypeSet);

impl Iterator for TokenTypeSetIter {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let num_bits: u32 = (size_of_val(&self.0.0) * 8) as u32;
        assert_eq!(num_bits, 128);
        let z = self.0.0.trailing_zeros();
        if z == num_bits {
            None
        } else {
            self.0.0 &= !(1 << z); // clear the trailing 1 bit
            Some(Token::from_u32(z))
        }
    }
}

#[derive(Clone, Debug)]
pub struct TokenStream<'db> {
    inner: Arc<Vec<Spanned<'db, Token>>>,
}

impl<'db> Deref for TokenStream<'db> {
    type Target = Vec<Spanned<'db, Token>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'db> TokenStream<'db> {
    pub fn new(mut inner: Vec<Spanned<'db, Token>>) -> Self {
        if let Some(last) = inner.last() {
            if last.node != Token::Eof {
                inner.push(Spanned::new(Token::Eof, last.span.clone()));
            }
        }
        TokenStream {
            inner: Arc::new(inner),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TokenStreamCursor<'db> {
    stream: TokenStream<'db>,
    index: usize,
}

impl<'db> TokenStreamCursor<'db> {
    #[inline]
    pub fn new(stream: TokenStream<'db>) -> Self {
        TokenStreamCursor { stream, index: 0 }
    }

    #[inline]
    /// Get the current token.
    pub fn curr(&self) -> Option<Spanned<'db, Token>> {
        self.stream.get(self.index).copied()
    }

    pub fn look_ahead(&self, n: usize) -> Option<&Spanned<'db, Token>> {
        self.stream.get(self.index + n)
    }

    #[inline]
    pub fn bump(&mut self) {
        self.index += 1;
    }

    pub fn eof(&self) -> bool {
        self.index >= self.stream.len()
    }

    // For skipping ahead in rare circumstances.
    #[inline]
    pub fn bump_to_end(&mut self) {
        self.index = self.stream.len();
    }
}
