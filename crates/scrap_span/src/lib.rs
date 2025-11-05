use std::{
    fmt,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span<T = usize> {
    /// The start of the span.
    pub start: T,
    /// The end of the span (exclusive).
    pub end: T,
}

impl<T> Span<T> {
    /// Create a new span.
    pub fn new(range: std::ops::Range<T>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }

    pub fn to_range(&self) -> std::ops::Range<T>
    where
        T: Clone,
    {
        self.start.clone()..self.end.clone()
    }
}

impl From<std::ops::Range<usize>> for Span<usize> {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Span<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn into_inner(self) -> T {
        self.node
    }

    pub fn into_parts(self) -> (T, Span) {
        (self.node, self.span)
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
/// A symbol represents an interned string.
pub struct Symbol(pub lasso::Spur);
