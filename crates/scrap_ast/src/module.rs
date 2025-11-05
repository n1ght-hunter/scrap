use scrap_span::Span;
use thin_vec::ThinVec;

use crate::item::Item;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub enum Module {
    /// Module with inlined definition `mod foo { ... }`,
    /// or with definition outlined to a separate file `mod foo;` and already loaded from it.
    /// The inner span is from the `mod` to the `}`,
    /// or from the first to the last token in the loaded file.
    Loaded(ThinVec<Box<Item>>, Inline, Span),
    
    Unloaded,
}