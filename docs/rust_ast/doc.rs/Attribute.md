# Struct Attribute

[Source](../../src/rustc_ast/ast.rs.html#3257-3264)

```
pub struct Attribute {
    pub kind: AttrKind,
    pub id: AttrId,
    pub style: AttrStyle,
    pub span: Span,
}
```



A syntax-level representation of an attribute.

## Fields

`kind: AttrKind`

`id: AttrId`

`style: AttrStyle`

Denotes if the attribute decorates the following construct (outer)
or the construct this attribute is contained within (inner).

`span: Span`

## Implementations

[Source](../../src/rustc_ast/attr/mod.rs.html#55-69)

### impl [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/attr/mod.rs.html#56-61)

#### pub fn [get\_normal\_item](#method.get_normal_item)(&self) -> &[AttrItem](struct.AttrItem.html "struct rustc_ast::ast::AttrItem")

[Source](../../src/rustc_ast/attr/mod.rs.html#63-68)

#### pub fn [unwrap\_normal\_item](#method.unwrap_normal_item)(self) -> [AttrItem](struct.AttrItem.html "struct rustc_ast::ast::AttrItem")

[Source](../../src/rustc_ast/attr/mod.rs.html#225-263)

### impl [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/attr/mod.rs.html#226-228)

#### pub fn [style](#method.style)(&self) -> [AttrStyle](enum.AttrStyle.html "enum rustc_ast::ast::AttrStyle")

[Source](../../src/rustc_ast/attr/mod.rs.html#230-232)

#### pub fn [may\_have\_doc\_links](#method.may_have_doc_links)(&self) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#235-240)

#### pub fn [meta](#method.meta)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[MetaItem](struct.MetaItem.html "struct rustc_ast::ast::MetaItem")>

Extracts the MetaItem from inside this Attribute.

[Source](../../src/rustc_ast/attr/mod.rs.html#242-247)

#### pub fn [meta\_kind](#method.meta_kind)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[MetaItemKind](enum.MetaItemKind.html "enum rustc_ast::ast::MetaItemKind")>

[Source](../../src/rustc_ast/attr/mod.rs.html#249-262)

#### pub fn [token\_trees](#method.token_trees)(&self) -> [Vec](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html "struct alloc::vec::Vec")<[TokenTree](../tokenstream/enum.TokenTree.html "enum rustc_ast::tokenstream::TokenTree")>

[Source](../../src/rustc_ast/attr/mod.rs.html#837-907)

### impl [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/attr/mod.rs.html#838-840)

#### pub fn [id](#method.id)(&self) -> [AttrId](struct.AttrId.html "struct rustc_ast::ast::AttrId")

[Source](../../src/rustc_ast/attr/mod.rs.html#842-844)

#### pub fn [name](#method.name)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")>

[Source](../../src/rustc_ast/attr/mod.rs.html#846-848)

#### pub fn [meta\_item\_list](#method.meta_item_list)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[ThinVec](https://docs.rs/thin-vec/0.2.14/thin_vec/struct.ThinVec.html "struct thin_vec::ThinVec")<[MetaItemInner](enum.MetaItemInner.html "enum rustc_ast::ast::MetaItemInner")>>

[Source](../../src/rustc_ast/attr/mod.rs.html#850-852)

#### pub fn [value\_str](#method.value_str)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")>

[Source](../../src/rustc_ast/attr/mod.rs.html#854-856)

#### pub fn [value\_span](#method.value_span)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span")>

[Source](../../src/rustc_ast/attr/mod.rs.html#858-860)

#### pub fn [ident](#method.ident)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Ident](../../rustc_span/symbol/struct.Ident.html "struct rustc_span::symbol::Ident")>

[Source](../../src/rustc_ast/attr/mod.rs.html#862-864)

#### pub fn [path\_matches](#method.path_matches)(&self, name: &[[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")]) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#866-868)

#### pub fn [is\_doc\_comment](#method.is_doc_comment)(&self) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#871-873)

#### pub fn [has\_name](#method.has_name)(&self, name: [Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#876-878)

#### pub fn [has\_any\_name](#method.has_any_name)(&self, names: &[[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")]) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#880-882)

#### pub fn [span](#method.span)(&self) -> [Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span")

[Source](../../src/rustc_ast/attr/mod.rs.html#884-886)

#### pub fn [is\_word](#method.is_word)(&self) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#888-890)

#### pub fn [path](#method.path)(&self) -> [SmallVec](https://docs.rs/smallvec/1.15.1/smallvec/struct.SmallVec.html "struct smallvec::SmallVec")<[[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol"); [1](https://doc.rust-lang.org/nightly/std/primitive.array.html)]>

[Source](../../src/rustc_ast/attr/mod.rs.html#892-894)

#### pub fn [ident\_path](#method.ident_path)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[SmallVec](https://docs.rs/smallvec/1.15.1/smallvec/struct.SmallVec.html "struct smallvec::SmallVec")<[[Ident](../../rustc_span/symbol/struct.Ident.html "struct rustc_span::symbol::Ident"); [1](https://doc.rust-lang.org/nightly/std/primitive.array.html)]>>

[Source](../../src/rustc_ast/attr/mod.rs.html#896-898)

#### pub fn [doc\_str](#method.doc_str)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")>

[Source](../../src/rustc_ast/attr/mod.rs.html#900-902)

#### pub fn [is\_proc\_macro\_attr](#method.is_proc_macro_attr)(&self) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#904-906)

#### pub fn [doc\_str\_and\_comment\_kind](#method.doc_str_and_comment_kind)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<([Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol"), [CommentKind](../token/enum.CommentKind.html "enum rustc_ast::token::CommentKind"))>

## Trait Implementations

[Source](../../src/rustc_ast/attr/mod.rs.html#71-223)

### impl [AttributeExt](../attr/trait.AttributeExt.html "trait rustc_ast::attr::AttributeExt") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/attr/mod.rs.html#89-94)

#### fn [is\_doc\_comment](../attr/trait.AttributeExt.html#tymethod.is_doc_comment)(&self) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

Returns `true` if it is a sugared doc comment (`///` or `//!` for example).
So `#[doc = "doc"]` (which is a doc comment) and `#[doc(...)]` (which is not
a doc comment) will return `false`.

[Source](../../src/rustc_ast/attr/mod.rs.html#97-108)

#### fn [ident](../attr/trait.AttributeExt.html#tymethod.ident)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Ident](../../rustc_span/symbol/struct.Ident.html "struct rustc_span::symbol::Ident")>

For a single-segment attribute, returns its name; otherwise, returns `None`.

[Source](../../src/rustc_ast/attr/mod.rs.html#152-157)

#### fn [meta\_item\_list](../attr/trait.AttributeExt.html#tymethod.meta_item_list)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[ThinVec](https://docs.rs/thin-vec/0.2.14/thin_vec/struct.ThinVec.html "struct thin_vec::ThinVec")<[MetaItemInner](enum.MetaItemInner.html "enum rustc_ast::ast::MetaItemInner")>>

Returns a list of meta items if the attribute is delimited with parenthesis:

```
#[attr(a, b = "c")] // Returns `Some()`.
#[attr = ""] // Returns `None`.
#[attr] // Returns `None`.
```

[Source](../../src/rustc_ast/attr/mod.rs.html#174-179)

#### fn [value\_str](../attr/trait.AttributeExt.html#tymethod.value_str)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")>

Returns the string value in:

```
#[attribute = "value"]
              ^^^^^^^
```

It returns `None` in any other cases, including doc comments if they
are not under the form `#[doc = "..."]`.

It also returns `None` for:

```
#[attr("value")]
```

[Source](../../src/rustc_ast/attr/mod.rs.html#186-194)

#### fn [doc\_str\_and\_comment\_kind](../attr/trait.AttributeExt.html#tymethod.doc_str_and_comment_kind)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<([Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol"), [CommentKind](../token/enum.CommentKind.html "enum rustc_ast::token::CommentKind"))>

Returns the documentation and its kind if this is a doc comment or a sugared doc comment.

* `///doc` returns `Some(("doc", CommentKind::Line))`.
* `/** doc */` returns `Some(("doc", CommentKind::Block))`.
* `#[doc = "doc"]` returns `Some(("doc", CommentKind::Line))`.
* `#[doc(...)]` returns `None`.

[Source](../../src/rustc_ast/attr/mod.rs.html#200-206)

#### fn [doc\_str](../attr/trait.AttributeExt.html#tymethod.doc_str)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")>

Returns the documentation if this is a doc comment or a sugared doc comment.

* `///doc` returns `Some("doc")`.
* `#[doc = "doc"]` returns `Some("doc")`.
* `#[doc(...)]` returns `None`.

[Source](../../src/rustc_ast/attr/mod.rs.html#72-74)

#### fn [id](../attr/trait.AttributeExt.html#tymethod.id)(&self) -> [AttrId](struct.AttrId.html "struct rustc_ast::ast::AttrId")

[Source](../../src/rustc_ast/attr/mod.rs.html#76-84)

#### fn [value\_span](../attr/trait.AttributeExt.html#tymethod.value_span)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span")>

Gets the span of the value literal, as string, when using `#[attr = value]`

[Source](../../src/rustc_ast/attr/mod.rs.html#110-115)

#### fn [ident\_path](../attr/trait.AttributeExt.html#tymethod.ident_path)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[SmallVec](https://docs.rs/smallvec/1.15.1/smallvec/struct.SmallVec.html "struct smallvec::SmallVec")<[[Ident](../../rustc_span/symbol/struct.Ident.html "struct rustc_span::symbol::Ident"); [1](https://doc.rust-lang.org/nightly/std/primitive.array.html)]>>

Returns None for doc comments

[Source](../../src/rustc_ast/attr/mod.rs.html#117-131)

#### fn [path\_matches](../attr/trait.AttributeExt.html#tymethod.path_matches)(&self, name: &[[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")]) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

Checks whether the path of this attribute matches the name. [Read more](../attr/trait.AttributeExt.html#tymethod.path_matches)

[Source](../../src/rustc_ast/attr/mod.rs.html#133-135)

#### fn [span](../attr/trait.AttributeExt.html#tymethod.span)(&self) -> [Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span")

get the span of the entire attribute

[Source](../../src/rustc_ast/attr/mod.rs.html#137-143)

#### fn [is\_word](../attr/trait.AttributeExt.html#tymethod.is_word)(&self) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#208-218)

#### fn [doc\_resolution\_scope](../attr/trait.AttributeExt.html#tymethod.doc_resolution_scope)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[AttrStyle](enum.AttrStyle.html "enum rustc_ast::ast::AttrStyle")>

Returns outer or inner if this is a doc attribute or a sugared doc
comment, otherwise None. [Read more](../attr/trait.AttributeExt.html#tymethod.doc_resolution_scope)

[Source](../../src/rustc_ast/attr/mod.rs.html#220-222)

#### fn [is\_automatically\_derived\_attr](../attr/trait.AttributeExt.html#tymethod.is_automatically_derived_attr)(&self) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#755-757)

#### fn [name](../attr/trait.AttributeExt.html#method.name)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")>

For a single-segment attribute (i.e., `#[attr]` and not `#[path::atrr]`),
return the name of the attribute; otherwise, returns `None`.

[Source](../../src/rustc_ast/attr/mod.rs.html#782-784)

#### fn [has\_name](../attr/trait.AttributeExt.html#method.has_name)(&self, name: [Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#787-789)

#### fn [has\_any\_name](../attr/trait.AttributeExt.html#method.has_any_name)(&self, names: &[[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol")]) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/attr/mod.rs.html#796-800)

#### fn [path](../attr/trait.AttributeExt.html#method.path)(&self) -> [SmallVec](https://docs.rs/smallvec/1.15.1/smallvec/struct.SmallVec.html "struct smallvec::SmallVec")<[[Symbol](../../rustc_span/symbol/struct.Symbol.html "struct rustc_span::symbol::Symbol"); [1](https://doc.rust-lang.org/nightly/std/primitive.array.html)]>

[Source](../../src/rustc_ast/attr/mod.rs.html#811-815)

#### fn [is\_proc\_macro\_attr](../attr/trait.AttributeExt.html#method.is_proc_macro_attr)(&self) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)

[Source](../../src/rustc_ast/ast.rs.html#3256)

### impl [Clone](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html "trait core::clone::Clone") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/ast.rs.html#3256)

#### fn [clone](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone)(&self) -> [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

Returns a duplicate of the value. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone)

1.0.0 Â· [Source](https://doc.rust-lang.org/nightly/src/core/clone.rs.html#246-248)

#### fn [clone\_from](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from)(&mut self, source: &Self)

Performs copy-assignment from `source`. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from)

[Source](../../src/rustc_ast/ast.rs.html#3256)

### impl [Debug](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html "trait core::fmt::Debug") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/ast.rs.html#3256)

#### fn [fmt](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)(&self, f: &mut [Formatter](https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html "struct core::fmt::Formatter")<'\_>) -> [Result](https://doc.rust-lang.org/nightly/core/fmt/type.Result.html "type core::fmt::Result")

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

[Source](../../src/rustc_ast/ast.rs.html#3256)

### impl<\_\_D: [SpanDecoder](../../rustc_span/trait.SpanDecoder.html "trait rustc_span::SpanDecoder")> [Decodable](../../rustc_serialize/serialize/trait.Decodable.html "trait rustc_serialize::serialize::Decodable")<\_\_D> for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/ast.rs.html#3256)

#### fn [decode](../../rustc_serialize/serialize/trait.Decodable.html#tymethod.decode)(\_\_decoder: [&mut \_\_D](https://doc.rust-lang.org/nightly/std/primitive.reference.html)) -> Self

[Source](../../src/rustc_ast/ast.rs.html#3256)

### impl<\_\_E: [SpanEncoder](../../rustc_span/trait.SpanEncoder.html "trait rustc_span::SpanEncoder")> [Encodable](../../rustc_serialize/serialize/trait.Encodable.html "trait rustc_serialize::serialize::Encodable")<\_\_E> for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/ast.rs.html#3256)

#### fn [encode](../../rustc_serialize/serialize/trait.Encodable.html#tymethod.encode)(&self, \_\_encoder: [&mut \_\_E](https://doc.rust-lang.org/nightly/std/primitive.reference.html))

[Source](../../src/rustc_ast/ast_traits.rs.html#245)

### impl [HasAttrs](../ast_traits/trait.HasAttrs.html "trait rustc_ast::ast_traits::HasAttrs") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/ast_traits.rs.html#245)

#### const [SUPPORTS\_CUSTOM\_INNER\_ATTRS](../ast_traits/trait.HasAttrs.html#associatedconstant.SUPPORTS_CUSTOM_INNER_ATTRS): [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html) = false

This is `true` if this `HasAttrs` might support âcustomâ (proc-macro) inner
attributes. Attributes like `#![cfg]` and `#![cfg_attr]` are not
considered âcustomâ attributes. [Read more](../ast_traits/trait.HasAttrs.html#associatedconstant.SUPPORTS_CUSTOM_INNER_ATTRS)

[Source](../../src/rustc_ast/ast_traits.rs.html#245)

#### fn [attrs](../ast_traits/trait.HasAttrs.html#tymethod.attrs)(&self) -> &[[Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")]

[Source](../../src/rustc_ast/ast_traits.rs.html#245)

#### fn [visit\_attrs](../ast_traits/trait.HasAttrs.html#tymethod.visit_attrs)(&mut self, \_f: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&mut [AttrVec](type.AttrVec.html "type rustc_ast::ast::AttrVec")))

[Source](../../src/rustc_ast/ast_traits.rs.html#160-177)

### impl [HasTokens](../ast_traits/trait.HasTokens.html "trait rustc_ast::ast_traits::HasTokens") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/ast_traits.rs.html#161-168)

#### fn [tokens](../ast_traits/trait.HasTokens.html#tymethod.tokens)(&self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<&[LazyAttrTokenStream](../tokenstream/struct.LazyAttrTokenStream.html "struct rustc_ast::tokenstream::LazyAttrTokenStream")>

[Source](../../src/rustc_ast/ast_traits.rs.html#169-176)

#### fn [tokens\_mut](../ast_traits/trait.HasTokens.html#tymethod.tokens_mut)(&mut self) -> [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<&mut [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[LazyAttrTokenStream](../tokenstream/struct.LazyAttrTokenStream.html "struct rustc_ast::tokenstream::LazyAttrTokenStream")>>

[Source](../../src/rustc_ast/mut_visit.rs.html#262)

### impl<V: [MutVisitor](../mut_visit/trait.MutVisitor.html "trait rustc_ast::mut_visit::MutVisitor")> [MutVisitable](../mut_visit/trait.MutVisitable.html "trait rustc_ast::mut_visit::MutVisitable")<V> for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/mut_visit.rs.html#262)

#### type [Extra](../mut_visit/trait.MutVisitable.html#associatedtype.Extra) = [()](https://doc.rust-lang.org/nightly/std/primitive.unit.html)

[Source](../../src/rustc_ast/mut_visit.rs.html#262)

#### fn [visit\_mut](../mut_visit/trait.MutVisitable.html#tymethod.visit_mut)(&mut self, visitor: [&mut V](https://doc.rust-lang.org/nightly/std/primitive.reference.html), extra: Self::[Extra](../mut_visit/trait.MutVisitable.html#associatedtype.Extra "type rustc_ast::mut_visit::MutVisitable::Extra")) -> V::[Result](../mut_visit/sealed/trait.MutVisitorResult.html#associatedtype.Result "type rustc_ast::mut_visit::sealed::MutVisitorResult::Result")

[Source](../../src/rustc_ast/ast.rs.html#3256)

### impl<\_\_V> [MutWalkable](../mut_visit/trait.MutWalkable.html "trait rustc_ast::mut_visit::MutWalkable")<\_\_V> for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute") where \_\_V: [MutVisitor](../mut_visit/trait.MutVisitor.html "trait rustc_ast::mut_visit::MutVisitor"),

[Source](../../src/rustc_ast/ast.rs.html#3256)

#### fn [walk\_mut](../mut_visit/trait.MutWalkable.html#tymethod.walk_mut)(&mut self, \_\_visitor: [&mut \_\_V](https://doc.rust-lang.org/nightly/std/primitive.reference.html))

[Source](../../src/rustc_ast/visit.rs.html#1117)

### impl<'a, V: [Visitor](../visit/trait.Visitor.html "trait rustc_ast::visit::Visitor")<'a>> [Visitable](../visit/trait.Visitable.html "trait rustc_ast::visit::Visitable")<'a, V> for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

[Source](../../src/rustc_ast/visit.rs.html#1117)

#### type [Extra](../visit/trait.Visitable.html#associatedtype.Extra) = [()](https://doc.rust-lang.org/nightly/std/primitive.unit.html)

[Source](../../src/rustc_ast/visit.rs.html#1117)

#### fn [visit](../visit/trait.Visitable.html#tymethod.visit)(&'a self, visitor: [&mut V](https://doc.rust-lang.org/nightly/std/primitive.reference.html), extra: Self::[Extra](../visit/trait.Visitable.html#associatedtype.Extra "type rustc_ast::visit::Visitable::Extra")) -> V::[Result](../visit/trait.Visitor.html#associatedtype.Result "type rustc_ast::visit::Visitor::Result")

[Source](../../src/rustc_ast/ast.rs.html#3256)

### impl<'\_\_ast, \_\_V> [Walkable](../visit/trait.Walkable.html "trait rustc_ast::visit::Walkable")<'\_\_ast, \_\_V> for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute") where \_\_V: [Visitor](../visit/trait.Visitor.html "trait rustc_ast::visit::Visitor")<'\_\_ast>,

[Source](../../src/rustc_ast/ast.rs.html#3256)

#### fn [walk\_ref](../visit/trait.Walkable.html#tymethod.walk_ref)(&'\_\_ast self, \_\_visitor: [&mut \_\_V](https://doc.rust-lang.org/nightly/std/primitive.reference.html)) -> \_\_V::[Result](../visit/trait.Visitor.html#associatedtype.Result "type rustc_ast::visit::Visitor::Result")

## Auto Trait Implementations

### impl [DynSend](../../rustc_data_structures/marker/trait.DynSend.html "trait rustc_data_structures::marker::DynSend") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

### impl [DynSync](../../rustc_data_structures/marker/trait.DynSync.html "trait rustc_data_structures::marker::DynSync") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

### impl [Freeze](https://doc.rust-lang.org/nightly/core/marker/trait.Freeze.html "trait core::marker::Freeze") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

### impl [RefUnwindSafe](https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

### impl [Send](https://doc.rust-lang.org/nightly/core/marker/trait.Send.html "trait core::marker::Send") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

### impl [Sync](https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html "trait core::marker::Sync") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

### impl [Unpin](https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html "trait core::marker::Unpin") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

### impl [UnwindSafe](https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe") for [Attribute](struct.Attribute.html "struct rustc_ast::ast::Attribute")

## Blanket Implementations

[Source](../../src/rustc_data_structures/aligned.rs.html#26)

### impl<T> [Aligned](../../rustc_data_structures/aligned/trait.Aligned.html "trait rustc_data_structures::aligned::Aligned") for T

[Source](../../src/rustc_data_structures/aligned.rs.html#27)

#### const [ALIGN](../../rustc_data_structures/aligned/trait.Aligned.html#associatedconstant.ALIGN): [Alignment](https://doc.rust-lang.org/nightly/core/ptr/alignment/struct.Alignment.html "struct core::ptr::alignment::Alignment")

Alignment of `Self`.

[Source](https://doc.rust-lang.org/nightly/src/core/any.rs.html#138)

### impl<T> [Any](https://doc.rust-lang.org/nightly/core/any/trait.Any.html "trait core::any::Any") for T where T: 'static + ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

[Source](https://doc.rust-lang.org/nightly/src/core/any.rs.html#139)

#### fn [type\_id](https://doc.rust-lang.org/nightly/core/any/trait.Any.html#tymethod.type_id)(&self) -> [TypeId](https://doc.rust-lang.org/nightly/core/any/struct.TypeId.html "struct core::any::TypeId")

Gets the `TypeId` of `self`. [Read more](https://doc.rust-lang.org/nightly/core/any/trait.Any.html#tymethod.type_id)

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#209)

### impl<T> [Borrow](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html "trait core::borrow::Borrow")<T> for T where T: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#211)

#### fn [borrow](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html#tymethod.borrow)(&self) -> [&T](https://doc.rust-lang.org/nightly/std/primitive.reference.html)

Immutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html#tymethod.borrow)

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#217)

### impl<T> [BorrowMut](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut")<T> for T where T: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#218)

#### fn [borrow\_mut](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)(&mut self) -> [&mut T](https://doc.rust-lang.org/nightly/std/primitive.reference.html)

Mutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)

[Source](https://doc.rust-lang.org/nightly/src/core/clone.rs.html#516)

### impl<T> [CloneToUninit](https://doc.rust-lang.org/nightly/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit") for T where T: [Clone](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html "trait core::clone::Clone"),

[Source](https://doc.rust-lang.org/nightly/src/core/clone.rs.html#518)

#### unsafe fn [clone\_to\_uninit](https://doc.rust-lang.org/nightly/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)(&self, dest: [\*mut](https://doc.rust-lang.org/nightly/std/primitive.pointer.html) [u8](https://doc.rust-lang.org/nightly/std/primitive.u8.html))

ð¬This is a nightly-only experimental API. (`clone_to_uninit`)

Performs copy-assignment from `self` to `dest`. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#791)

### impl<T> [From](https://doc.rust-lang.org/nightly/core/convert/trait.From.html "trait core::convert::From")<T> for T

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#794)

#### fn [from](https://doc.rust-lang.org/nightly/core/convert/trait.From.html#tymethod.from)(t: T) -> T

Returns the argument unchanged.

[Source](https://docs.rs/tracing/0.1.37/src/tracing/instrument.rs.html#276)

### impl<T> [Instrument](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.Instrument.html "trait tracing::instrument::Instrument") for T

[Source](https://docs.rs/tracing/0.1.37/src/tracing/instrument.rs.html#82)

#### fn [instrument](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.Instrument.html#method.instrument)(self, span: [Span](https://docs.rs/tracing/0.1.37/tracing/span/struct.Span.html "struct tracing::span::Span")) -> [Instrumented](https://docs.rs/tracing/0.1.37/tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented")<Self>

Instruments this type with the provided [`Span`](https://docs.rs/tracing/0.1.37/tracing/span/struct.Span.html "struct tracing::span::Span"), returning an
`Instrumented` wrapper. [Read more](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.Instrument.html#method.instrument)

[Source](https://docs.rs/tracing/0.1.37/src/tracing/instrument.rs.html#121)

#### fn [in\_current\_span](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.Instrument.html#method.in_current_span)(self) -> [Instrumented](https://docs.rs/tracing/0.1.37/tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented")<Self>

Instruments this type with the [current](https://docs.rs/tracing/0.1.37/tracing/span/struct.Span.html#method.current "associated function tracing::span::Span::current") [`Span`](https://docs.rs/tracing/0.1.37/tracing/span/struct.Span.html "struct tracing::span::Span"), returning an
`Instrumented` wrapper. [Read more](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.Instrument.html#method.in_current_span)

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#773-775)

### impl<T, U> [Into](https://doc.rust-lang.org/nightly/core/convert/trait.Into.html "trait core::convert::Into")<U> for T where U: [From](https://doc.rust-lang.org/nightly/core/convert/trait.From.html "trait core::convert::From")<T>,

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#783)

#### fn [into](https://doc.rust-lang.org/nightly/core/convert/trait.Into.html#tymethod.into)(self) -> U

Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
`From<T> for U` chooses to do.

[Source](https://docs.rs/either/1/src/either/into_either.rs.html#64)

### impl<T> [IntoEither](https://docs.rs/either/1/either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither") for T

[Source](https://docs.rs/either/1/src/either/into_either.rs.html#29)

#### fn [into\_either](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either)(self, into\_left: [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html)) -> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")<Self, Self>

Converts `self` into a [`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left") variant of [`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left` is `true`.
Converts `self` into a [`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right") variant of [`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either)

[Source](https://docs.rs/either/1/src/either/into_either.rs.html#55-57)

#### fn [into\_either\_with](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with)<F>(self, into\_left: F) -> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")<Self, Self> where F: [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&Self) -> [bool](https://doc.rust-lang.org/nightly/std/primitive.bool.html),

Converts `self` into a [`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left") variant of [`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`.
Converts `self` into a [`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right") variant of [`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with)

[Source](../../src/rustc_ast/mut_visit.rs.html#31-33)

### impl<T> [MutVisitorResult](../mut_visit/sealed/trait.MutVisitorResult.html "trait rustc_ast::mut_visit::sealed::MutVisitorResult") for T

[Source](../../src/rustc_ast/mut_visit.rs.html#32)

#### type [Result](../mut_visit/sealed/trait.MutVisitorResult.html#associatedtype.Result) = [()](https://doc.rust-lang.org/nightly/std/primitive.unit.html)

[Source](https://docs.rs/crossbeam-epoch/0.9.18/src/crossbeam_epoch/atomic.rs.html#194)

### impl<T> [Pointable](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable") for T

[Source](https://docs.rs/crossbeam-epoch/0.9.18/src/crossbeam_epoch/atomic.rs.html#195)

#### const [ALIGN](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#associatedconstant.ALIGN): [usize](https://doc.rust-lang.org/nightly/std/primitive.usize.html)

The alignment of pointer.

[Source](https://docs.rs/crossbeam-epoch/0.9.18/src/crossbeam_epoch/atomic.rs.html#197)

#### type [Init](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init) = T

The type for initializers.

[Source](https://docs.rs/crossbeam-epoch/0.9.18/src/crossbeam_epoch/atomic.rs.html#199)

#### unsafe fn [init](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init)(init: <T as [Pointable](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable")>::[Init](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init "type crossbeam_epoch::atomic::Pointable::Init")) -> [usize](https://doc.rust-lang.org/nightly/std/primitive.usize.html)

Initializes a with the given initializer. [Read more](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init)

[Source](https://docs.rs/crossbeam-epoch/0.9.18/src/crossbeam_epoch/atomic.rs.html#203)

#### unsafe fn [deref](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref)<'a>(ptr: [usize](https://doc.rust-lang.org/nightly/std/primitive.usize.html)) -> [&'a T](https://doc.rust-lang.org/nightly/std/primitive.reference.html)

Dereferences the given pointer. [Read more](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref)

[Source](https://docs.rs/crossbeam-epoch/0.9.18/src/crossbeam_epoch/atomic.rs.html#207)

#### unsafe fn [deref\_mut](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut)<'a>(ptr: [usize](https://doc.rust-lang.org/nightly/std/primitive.usize.html)) -> [&'a mut T](https://doc.rust-lang.org/nightly/std/primitive.reference.html)

Mutably dereferences the given pointer. [Read more](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut)

[Source](https://docs.rs/crossbeam-epoch/0.9.18/src/crossbeam_epoch/atomic.rs.html#211)

#### unsafe fn [drop](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop)(ptr: [usize](https://doc.rust-lang.org/nightly/std/primitive.usize.html))

Drops the object pointed to by the given pointer. [Read more](https://docs.rs/crossbeam-epoch/0.9.18/crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop)

[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#34)

### impl<T> [Same](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same") for T

[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#35)

#### type [Output](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html#associatedtype.Output) = T

Should always be `Self`

[Source](https://doc.rust-lang.org/nightly/src/alloc/borrow.rs.html#82-84)

### impl<T> [ToOwned](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned") for T where T: [Clone](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html "trait core::clone::Clone"),

[Source](https://doc.rust-lang.org/nightly/src/alloc/borrow.rs.html#86)

#### type [Owned](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html#associatedtype.Owned) = T

The resulting type after obtaining ownership.

[Source](https://doc.rust-lang.org/nightly/src/alloc/borrow.rs.html#87)

#### fn [to\_owned](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)(&self) -> T

Creates owned data from borrowed data, usually by cloning. [Read more](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)

[Source](https://doc.rust-lang.org/nightly/src/alloc/borrow.rs.html#91)

#### fn [clone\_into](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html#method.clone_into)(&self, target: [&mut T](https://doc.rust-lang.org/nightly/std/primitive.reference.html))

Uses borrowed data to replace owned data, usually by cloning. [Read more](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html#method.clone_into)

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#833-835)

### impl<T, U> [TryFrom](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html "trait core::convert::TryFrom")<U> for T where U: [Into](https://doc.rust-lang.org/nightly/core/convert/trait.Into.html "trait core::convert::Into")<T>,

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#837)

#### type [Error](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html#associatedtype.Error) = [Infallible](https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html "enum core::convert::Infallible")

The type returned in the event of a conversion error.

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#840)

#### fn [try\_from](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html#tymethod.try_from)(value: U) -> [Result](https://doc.rust-lang.org/nightly/core/result/enum.Result.html "enum core::result::Result")<T, <T as [TryFrom](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html "trait core::convert::TryFrom")<U>>::[Error](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error")>

Performs the conversion.

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#817-819)

### impl<T, U> [TryInto](https://doc.rust-lang.org/nightly/core/convert/trait.TryInto.html "trait core::convert::TryInto")<U> for T where U: [TryFrom](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html "trait core::convert::TryFrom")<T>,

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#821)

#### type [Error](https://doc.rust-lang.org/nightly/core/convert/trait.TryInto.html#associatedtype.Error) = <U as [TryFrom](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html "trait core::convert::TryFrom")<T>>::[Error](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error")

The type returned in the event of a conversion error.

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#824)

#### fn [try\_into](https://doc.rust-lang.org/nightly/core/convert/trait.TryInto.html#tymethod.try_into)(self) -> [Result](https://doc.rust-lang.org/nightly/core/result/enum.Result.html "enum core::result::Result")<U, <U as [TryFrom](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html "trait core::convert::TryFrom")<T>>::[Error](https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error")>

Performs the conversion.

[Source](https://docs.rs/tracing/0.1.37/src/tracing/instrument.rs.html#334)

### impl<T> [WithSubscriber](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.WithSubscriber.html "trait tracing::instrument::WithSubscriber") for T

[Source](https://docs.rs/tracing/0.1.37/src/tracing/instrument.rs.html#169-171)

#### fn [with\_subscriber](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.WithSubscriber.html#method.with_subscriber)<S>(self, subscriber: S) -> [WithDispatch](https://docs.rs/tracing/0.1.37/tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")<Self> where S: [Into](https://doc.rust-lang.org/nightly/core/convert/trait.Into.html "trait core::convert::Into")<[Dispatch](https://docs.rs/tracing-core/0.1.22/tracing_core/dispatcher/struct.Dispatch.html "struct tracing_core::dispatcher::Dispatch")>,

Attaches the provided [`Subscriber`](https://docs.rs/tracing-core/0.1.22/tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber") to this type, returning a
[`WithDispatch`](https://docs.rs/tracing/0.1.37/tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch") wrapper. [Read more](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.WithSubscriber.html#method.with_subscriber)

[Source](https://docs.rs/tracing/0.1.37/src/tracing/instrument.rs.html#221)

#### fn [with\_current\_subscriber](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber)(self) -> [WithDispatch](https://docs.rs/tracing/0.1.37/tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")<Self>

Attaches the current [default](https://docs.rs/tracing/0.1.37/tracing/dispatcher/index.html#setting-the-default-subscriber "mod tracing::dispatcher") [`Subscriber`](https://docs.rs/tracing-core/0.1.22/tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber") to this type, returning a
[`WithDispatch`](https://docs.rs/tracing/0.1.37/tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch") wrapper. [Read more](https://docs.rs/tracing/0.1.37/tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber)

## Layout

**Note:** Most layout information is **completely unstable** and may even differ between compilations. The only exception is types with certain `repr(...)` attributes. Please see the Rust Reference's [âType Layoutâ](https://doc.rust-lang.org/nightly/reference/type-layout.html) chapter for details on type layout guarantees.

**Size:** 32 bytes

---

