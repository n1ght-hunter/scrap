# Enum ExprKind

[Source](../../src/rustc_ast/ast.rs.html#1707-1884)

```
pub enum ExprKind {
Array(ThinVec<Box<Expr>>),
    ConstBlock(AnonConst),
    Call(Box<Expr>, ThinVec<Box<Expr>>),
    MethodCall(Box<MethodCall>),
    Tup(ThinVec<Box<Expr>>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Lit(Lit),
    Cast(Box<Expr>, Box<Ty>),
    Type(Box<Expr>, Box<Ty>),
    Let(Box<Pat>, Box<Expr>, Span, Recovered),
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    While(Box<Expr>, Box<Block>, Option<Label>),
    ForLoop {
        pat: Box<Pat>,
        iter: Box<Expr>,
        body: Box<Block>,
        label: Option<Label>,
        kind: ForLoopKind,
    },
    Loop(Box<Block>, Option<Label>, Span),
    Match(Box<Expr>, ThinVec<Arm>, MatchKind),
    Closure(Box<Closure>),
    Block(Box<Block>, Option<Label>),
    Gen(CaptureBy, Box<Block>, GenBlockKind, Span),
    Await(Box<Expr>, Span),
    Use(Box<Expr>, Span),
    TryBlock(Box<Block>),
    Assign(Box<Expr>, Box<Expr>, Span),
    AssignOp(AssignOp, Box<Expr>, Box<Expr>),
    Field(Box<Expr>, Ident),
    Index(Box<Expr>, Box<Expr>, Span),
    Range(Option<Box<Expr>>, Option<Box<Expr>>, RangeLimits),
    Underscore,
    Path(Option<Box<QSelf>>, Path),
    AddrOf(BorrowKind, Mutability, Box<Expr>),
    Break(Option<Label>, Option<Box<Expr>>),
    Continue(Option<Label>),
    Ret(Option<Box<Expr>>),
    InlineAsm(Box<InlineAsm>),
    OffsetOf(Box<Ty>, Vec<Ident>),
    MacCall(Box<MacCall>),
    Struct(Box<StructExpr>),
    Repeat(Box<Expr>, AnonConst),
    Paren(Box<Expr>),
    Try(Box<Expr>),
    Yield(YieldKind),
    Yeet(Option<Box<Expr>>),
    Become(Box<Expr>),
    IncludedBytes(ByteSymbol),
    FormatArgs(Box<FormatArgs>),
    UnsafeBinderCast(UnsafeBinderCastKind, Box<Expr>, Option<Box<Ty>>),
    Err(ErrorGuaranteed),
    Dummy,
}
```

## Variants

### Array([ThinVec](https://docs.rs/thin-vec/0.2.14/thin_vec/struct.ThinVec.html "struct thin_vec::ThinVec")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>>)

An array (e.g, `[a, b, c, d]`).

### ConstBlock([AnonConst](struct.AnonConst.html "struct rustc_ast::ast::AnonConst"))

Allow anonymous constants from an inline `const` block.

### Call([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [ThinVec](https://docs.rs/thin-vec/0.2.14/thin_vec/struct.ThinVec.html "struct thin_vec::ThinVec")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>>)

A function call.

The first field resolves to the function itself,
and the second field is the list of arguments.
This also represents calling the constructor of
tuple-like ADTs such as tuple structs and enum variants.

### MethodCall([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[MethodCall](struct.MethodCall.html "struct rustc_ast::ast::MethodCall")>)

A method call (e.g., `x.foo::<Bar, Baz>(a, b, c)`).

### Tup([ThinVec](https://docs.rs/thin-vec/0.2.14/thin_vec/struct.ThinVec.html "struct thin_vec::ThinVec")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>>)

A tuple (e.g., `(a, b, c, d)`).

### Binary([BinOp](type.BinOp.html "type rustc_ast::ast::BinOp"), [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>)

A binary operation (e.g., `a + b`, `a * b`).

### Unary([UnOp](enum.UnOp.html "enum rustc_ast::ast::UnOp"), [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>)

A unary operation (e.g., `!x`, `*x`).

### Lit([Lit](../token/struct.Lit.html "struct rustc_ast::token::Lit"))

A literal (e.g., `1`, `"foo"`).

### Cast([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Ty](struct.Ty.html "struct rustc_ast::ast::Ty")>)

A cast (e.g., `foo as f64`).

### Type([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Ty](struct.Ty.html "struct rustc_ast::ast::Ty")>)

A type ascription (e.g., `builtin # type_ascribe(42, usize)`).

Usually not written directly in user code but
indirectly via the macro `type_ascribe!(...)`.

### Let([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Pat](struct.Pat.html "struct rustc_ast::ast::Pat")>, [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span"), [Recovered](enum.Recovered.html "enum rustc_ast::ast::Recovered"))

A `let pat = expr` expression that is only semantically allowed in the condition
of `if` / `while` expressions. (e.g., `if let 0 = x { .. }`).

`Span` represents the whole `let pat = expr` statement.

### If([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Block](struct.Block.html "struct rustc_ast::ast::Block")>, [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>>)

An `if` block, with an optional `else` block.

`if expr { block } else { expr }`

If present, the âelseâ expr is always `ExprKind::Block` (for `else`) or
`ExprKind::If` (for `else if`).

### While([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Block](struct.Block.html "struct rustc_ast::ast::Block")>, [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Label](struct.Label.html "struct rustc_ast::ast::Label")>)

A while loop, with an optional label.

`'label: while expr { block }`

### ForLoop

A `for` loop, with an optional label.

`'label: for await? pat in iter { block }`

This is desugared to a combination of `loop` and `match` expressions.

#### Fields

`pat: Box<Pat>`

`iter: Box<Expr>`

`body: Box<Block>`

`label: Option<Label>`

`kind: ForLoopKind`

### Loop([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Block](struct.Block.html "struct rustc_ast::ast::Block")>, [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Label](struct.Label.html "struct rustc_ast::ast::Label")>, [Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span"))

Conditionless loop (can be exited with `break`, `continue`, or `return`).

`'label: loop { block }`

### Match([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [ThinVec](https://docs.rs/thin-vec/0.2.14/thin_vec/struct.ThinVec.html "struct thin_vec::ThinVec")<[Arm](struct.Arm.html "struct rustc_ast::ast::Arm")>, [MatchKind](enum.MatchKind.html "enum rustc_ast::ast::MatchKind"))

A `match` block.

### Closure([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Closure](struct.Closure.html "struct rustc_ast::ast::Closure")>)

A closure (e.g., `move |a, b, c| a + b + c`).

### Block([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Block](struct.Block.html "struct rustc_ast::ast::Block")>, [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Label](struct.Label.html "struct rustc_ast::ast::Label")>)

A block (`'label: { ... }`).

### Gen([CaptureBy](enum.CaptureBy.html "enum rustc_ast::ast::CaptureBy"), [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Block](struct.Block.html "struct rustc_ast::ast::Block")>, [GenBlockKind](enum.GenBlockKind.html "enum rustc_ast::ast::GenBlockKind"), [Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span"))

An `async` block (`async move { ... }`),
or a `gen` block (`gen move { ... }`).

The span is the âdeclâ, which is the header before the body `{ }`
including the `asyng`/`gen` keywords and possibly `move`.

### Await([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span"))

An await expression (`my_future.await`). Span is of await keyword.

### Use([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span"))

A use expression (`x.use`). Span is of use keyword.

### TryBlock([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Block](struct.Block.html "struct rustc_ast::ast::Block")>)

A try block (`try { ... }`).

### Assign([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span"))

An assignment (`a = foo()`).
The `Span` argument is the span of the `=` token.

### AssignOp([AssignOp](type.AssignOp.html "type rustc_ast::ast::AssignOp"), [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>)

An assignment with an operator.

E.g., `a += 1`.

### Field([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Ident](../../rustc_span/symbol/struct.Ident.html "struct rustc_span::symbol::Ident"))

Access of a named (e.g., `obj.foo`) or unnamed (e.g., `obj.0`) struct field.

### Index([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Span](../../rustc_span/span_encoding/struct.Span.html "struct rustc_span::span_encoding::Span"))

An indexing operation (e.g., `foo[2]`).
The span represents the span of the `[2]`, including brackets.

### Range([Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>>, [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>>, [RangeLimits](enum.RangeLimits.html "enum rustc_ast::ast::RangeLimits"))

A range (e.g., `1..2`, `1..`, `..2`, `1..=2`, `..=2`; and `..` in destructuring assignment).

### Underscore

An underscore, used in destructuring assignment to ignore a value.

### Path([Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[QSelf](struct.QSelf.html "struct rustc_ast::ast::QSelf")>>, [Path](struct.Path.html "struct rustc_ast::ast::Path"))

Variable reference, possibly containing `::` and/or type
parameters (e.g., `foo::bar::<baz>`).

Optionally âqualifiedâ (e.g., `<Vec<T> as SomeTrait>::SomeType`).

### AddrOf([BorrowKind](enum.BorrowKind.html "enum rustc_ast::ast::BorrowKind"), [Mutability](enum.Mutability.html "enum rustc_ast::ast::Mutability"), [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>)

A referencing operation (`&a`, `&mut a`, `&raw const a` or `&raw mut a`).

### Break([Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Label](struct.Label.html "struct rustc_ast::ast::Label")>, [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>>)

A `break`, with an optional label to break, and an optional expression.

### Continue([Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Label](struct.Label.html "struct rustc_ast::ast::Label")>)

A `continue`, with an optional label.

### Ret([Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>>)

A `return`, with an optional value to be returned.

### InlineAsm([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[InlineAsm](struct.InlineAsm.html "struct rustc_ast::ast::InlineAsm")>)

Output of the `asm!()` macro.

### OffsetOf([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Ty](struct.Ty.html "struct rustc_ast::ast::Ty")>, [Vec](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html "struct alloc::vec::Vec")<[Ident](../../rustc_span/symbol/struct.Ident.html "struct rustc_span::symbol::Ident")>)

An `offset_of` expression (e.g., `builtin # offset_of(Struct, field)`).

Usually not written directly in user code but
indirectly via the macro `core::mem::offset_of!(...)`.

### MacCall([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[MacCall](struct.MacCall.html "struct rustc_ast::ast::MacCall")>)

A macro invocation; pre-expansion.

### Struct([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[StructExpr](struct.StructExpr.html "struct rustc_ast::ast::StructExpr")>)

A struct literal expression.

E.g., `Foo {x: 1, y: 2}`, or `Foo {x: 1, .. rest}`.

### Repeat([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [AnonConst](struct.AnonConst.html "struct rustc_ast::ast::AnonConst"))

An array literal constructed from one repeated element.

E.g., `[1; 5]`. The expression is the element to be
repeated; the constant is the number of times to repeat it.

### Paren([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>)

No-op: used solely so we can pretty-print faithfully.

### Try([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>)

A try expression (`expr?`).

### Yield([YieldKind](enum.YieldKind.html "enum rustc_ast::ast::YieldKind"))

A `yield`, with an optional value to be yielded.

### Yeet([Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>>)

A `do yeet` (aka `throw`/`fail`/`bail`/`raise`/whatever),
with an optional value to be returned.

### Become([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>)

A tail call return, with the value to be returned.

While `.0` must be a function call, we check this later, after parsing.

### IncludedBytes([ByteSymbol](../../rustc_span/symbol/struct.ByteSymbol.html "struct rustc_span::symbol::ByteSymbol"))

Bytes included via `include_bytes!`

Added for optimization purposes to avoid the need to escape
large binary blobs - should always behave like [`ExprKind::Lit`](enum.ExprKind.html#variant.Lit "variant rustc_ast::ast::ExprKind::Lit")
with a `ByteStr` literal.

The value is stored as a `ByteSymbol`. Itâs unfortunate that we need to
intern (hash) the bytes because theyâre likely to be large and unique.
But itâs necessary because this will eventually be lowered to
`LitKind::ByteStr`, which needs a `ByteSymbol` to impl `Copy` and avoid
arena allocation.

### FormatArgs([Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[FormatArgs](../format/struct.FormatArgs.html "struct rustc_ast::format::FormatArgs")>)

A `format_args!()` expression.

### UnsafeBinderCast([UnsafeBinderCastKind](enum.UnsafeBinderCastKind.html "enum rustc_ast::ast::UnsafeBinderCastKind"), [Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Expr](struct.Expr.html "struct rustc_ast::ast::Expr")>, [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<[Box](https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html "struct alloc::boxed::Box")<[Ty](struct.Ty.html "struct rustc_ast::ast::Ty")>>)

### Err([ErrorGuaranteed](../../rustc_span/struct.ErrorGuaranteed.html "struct rustc_span::ErrorGuaranteed"))

Placeholder for an expression that wasnât syntactically well formed in some way.

### Dummy

Acts as a null expression. Lowering it will always emit a bug.

## Trait Implementations

[Source](../../src/rustc_ast/ast.rs.html#1706)

### impl [Clone](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html "trait core::clone::Clone") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

[Source](../../src/rustc_ast/ast.rs.html#1706)

#### fn [clone](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone)(&self) -> [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

Returns a duplicate of the value. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone)

1.0.0 Â· [Source](https://doc.rust-lang.org/nightly/src/core/clone.rs.html#246-248)

#### fn [clone\_from](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from)(&mut self, source: &Self)

Performs copy-assignment from `source`. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from)

[Source](../../src/rustc_ast/ast.rs.html#1706)

### impl [Debug](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html "trait core::fmt::Debug") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

[Source](../../src/rustc_ast/ast.rs.html#1706)

#### fn [fmt](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)(&self, f: &mut [Formatter](https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html "struct core::fmt::Formatter")<'\_>) -> [Result](https://doc.rust-lang.org/nightly/core/fmt/type.Result.html "type core::fmt::Result")

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

[Source](../../src/rustc_ast/ast.rs.html#1706)

### impl<\_\_D: [SpanDecoder](../../rustc_span/trait.SpanDecoder.html "trait rustc_span::SpanDecoder")> [Decodable](../../rustc_serialize/serialize/trait.Decodable.html "trait rustc_serialize::serialize::Decodable")<\_\_D> for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

[Source](../../src/rustc_ast/ast.rs.html#1706)

#### fn [decode](../../rustc_serialize/serialize/trait.Decodable.html#tymethod.decode)(\_\_decoder: [&mut \_\_D](https://doc.rust-lang.org/nightly/std/primitive.reference.html)) -> Self

[Source](../../src/rustc_ast/ast.rs.html#1706)

### impl<\_\_E: [SpanEncoder](../../rustc_span/trait.SpanEncoder.html "trait rustc_span::SpanEncoder")> [Encodable](../../rustc_serialize/serialize/trait.Encodable.html "trait rustc_serialize::serialize::Encodable")<\_\_E> for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

[Source](../../src/rustc_ast/ast.rs.html#1706)

#### fn [encode](../../rustc_serialize/serialize/trait.Encodable.html#tymethod.encode)(&self, \_\_encoder: [&mut \_\_E](https://doc.rust-lang.org/nightly/std/primitive.reference.html))

## Auto Trait Implementations

### impl [DynSend](../../rustc_data_structures/marker/trait.DynSend.html "trait rustc_data_structures::marker::DynSend") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

### impl [DynSync](../../rustc_data_structures/marker/trait.DynSync.html "trait rustc_data_structures::marker::DynSync") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

### impl [Freeze](https://doc.rust-lang.org/nightly/core/marker/trait.Freeze.html "trait core::marker::Freeze") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

### impl [RefUnwindSafe](https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

### impl [Send](https://doc.rust-lang.org/nightly/core/marker/trait.Send.html "trait core::marker::Send") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

### impl [Sync](https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html "trait core::marker::Sync") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

### impl [Unpin](https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html "trait core::marker::Unpin") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

### impl [UnwindSafe](https://doc.rust-lang.org/nightly/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe") for [ExprKind](enum.ExprKind.html "enum rustc_ast::ast::ExprKind")

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

**Size:** 40 bytes

**Size for each variant:**

* `Array`: 15 bytes
* `ConstBlock`: 23 bytes
* `Call`: 23 bytes
* `MethodCall`: 15 bytes
* `Tup`: 15 bytes
* `Binary`: 31 bytes
* `Unary`: 15 bytes
* `Lit`: 15 bytes
* `Cast`: 23 bytes
* `Type`: 23 bytes
* `Let`: 31 bytes
* `If`: 31 bytes
* `While`: 31 bytes
* `ForLoop`: 39 bytes
* `Loop`: 31 bytes
* `Match`: 23 bytes
* `Closure`: 15 bytes
* `Block`: 23 bytes
* `Gen`: 31 bytes
* `Await`: 23 bytes
* `Use`: 23 bytes
* `TryBlock`: 15 bytes
* `Assign`: 31 bytes
* `AssignOp`: 31 bytes
* `Field`: 23 bytes
* `Index`: 31 bytes
* `Range`: 23 bytes
* `Underscore`: 0 bytes
* `Path`: 39 bytes
* `AddrOf`: 15 bytes
* `Break`: 23 bytes
* `Continue`: 15 bytes
* `Ret`: 15 bytes
* `InlineAsm`: 15 bytes
* `OffsetOf`: 39 bytes
* `MacCall`: 15 bytes
* `Struct`: 15 bytes
* `Repeat`: 31 bytes
* `Paren`: 15 bytes
* `Try`: 15 bytes
* `Yield`: 23 bytes
* `Yeet`: 15 bytes
* `Become`: 15 bytes
* `IncludedBytes`: 7 bytes
* `FormatArgs`: 15 bytes
* `UnsafeBinderCast`: 23 bytes
* `Err`: 0 bytes
* `Dummy`: 0 bytes

---

