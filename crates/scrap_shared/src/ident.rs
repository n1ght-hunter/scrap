use scrap_span::Span;

use crate::{NodeId, pretty_print::PrettyPrint};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct Ident {
    pub id: NodeId,
    pub name: Symbol,
    pub span: Span,
}

impl PrettyPrint for Ident {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        write!(f, "{}", self.name.text())
    }
}

impl Ident {
    pub fn dummy() -> Self {
        Self {
            id: NodeId::dummy(),
            name: Symbol::new("<dummy>"),
            span: Span::default(),
        }
    }

    pub fn dummy_with_name(name: &str) -> Self {
        Self {
            id: NodeId::dummy(),
            name: Symbol::new(name),
            span: Span::default(),
        }
    }
}

use lasso::{Spur, ThreadedRodeo};
use std::sync::LazyLock;

static INTERNER: LazyLock<ThreadedRodeo> = LazyLock::new(ThreadedRodeo::default);

/// Interned string key. Backed by a global thread-safe interner (lasso).
/// `Copy`, 4 bytes (`Spur` is a `NonZeroU32`), O(1) resolution.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(Spur);

impl serde::Serialize for Symbol {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.text().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Symbol {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Ok(Symbol::new(text))
    }
}

impl Symbol {
    #[inline]
    pub fn new(text: impl AsRef<str>) -> Self {
        Self(INTERNER.get_or_intern(text.as_ref()))
    }

    #[inline]
    pub fn text(&self) -> &str {
        INTERNER.resolve(&self.0)
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol({:?})", self.text())
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// SAFETY: Symbol is Copy, `'static`, and contains only a Spur (NonZeroU32) —
/// no borrows of database storage, so an instance interned in an older revision
/// stays valid in a newer one.
#[allow(unsafe_code)]
unsafe impl salsa::SalsaValue for Symbol {}
