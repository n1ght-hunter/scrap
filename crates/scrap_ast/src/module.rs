use scrap_span::Span;
use thin_vec::ThinVec;

use crate::item::Item;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Inline {
    Yes,
    No,
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Module<'db> {
    /// Module with inlined definition `mod foo { ... }`,
    /// or with definition outlined to a separate file `mod foo;` and already loaded from it.
    /// The inner span is from the `mod` to the `}`,
    /// or from the first to the last token in the loaded file.
    Loaded(ThinVec<Box<Item<'db>>>, Inline, Span<'db>),

    Unloaded,
}
