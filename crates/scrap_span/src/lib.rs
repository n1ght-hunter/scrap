use std::ops::{Deref, DerefMut, Range};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    salsa::SalsaValue,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn range(self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct Spanned<T: salsa::SalsaValue> {
    pub node: T,
    pub span: Span,
}

impl<T: salsa::SalsaValue> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn into_inner(self) -> T {
        self.node
    }
}

impl<T: salsa::SalsaValue> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T: salsa::SalsaValue> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}
