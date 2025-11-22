use std::ops::{Deref, DerefMut};

#[salsa::tracked(debug, persist)]
pub struct Span<'db> {
    /// The start of the span.
    #[tracked]
    pub start: usize,
    /// The end of the span (exclusive).
    #[tracked]
    pub end: usize,
}

impl<'db> Span<'db> {
    pub fn new_default(db: &'db dyn salsa::Database) -> Self {
        Self::new(db, 0, 0)
    }

    pub fn to_range(&self, db: &'db dyn salsa::Database) -> std::ops::Range<usize> {
        self.start(db)..self.end(db)
    }
}

#[salsa::tracked]
pub fn new_span<'db>(db: &'db dyn salsa::Database, start: usize, end: usize) -> Span<'db> {
    Span::new(db, start, end)
}

pub fn new_dummy_span<'db>(db: &'db dyn salsa::Database) -> Span<'db> {
    new_span(db, 0, 0)
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Spanned<'db, T: salsa::Update> {
    pub node: T,
    pub span: Span<'db>,
}

impl<'db, T: salsa::Update> Spanned<'db, T> {
    pub fn new(node: T, span: Span<'db>) -> Self {
        Self { node, span }
    }

    pub fn span(&self) -> &Span<'db> {
        &self.span
    }

    pub fn into_inner(self) -> T {
        self.node
    }

    pub fn into_parts(self) -> (T, Span<'db>) {
        (self.node, self.span)
    }
}

impl<'db, T: salsa::Update> Deref for Spanned<'db, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'db, T: salsa::Update> DerefMut for Spanned<'db, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}
