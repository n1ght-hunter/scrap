# rmeta-fixture

A small, standalone Rust crate of sample types and functions used as a fixture for the native
Rust-interop tests. It is **not** a workspace member (it declares its own empty `[workspace]`) so it
isn't pulled into the scrap build; the interop end-to-end tests under
[`tests/rust_interop_*`](../../tests) depend on it as a path dependency through a generated anchor.

It exercises the interop ABI/layout/drop paths: scalar and `#[repr(C)]` structs, a `ScalarPair`
return, a large (`sret`/`Indirect`) struct, a struct with an opaque non-scalar field, a `Drop` type
(for stack RAII and GC-finalizer tests), and types with inherent + associated methods.

When the schema, extractor, or these fixtures change, clear the anchor cache (`target/scrap/anchor`)
before re-running the interop tests — the cache key does not yet hash path-dependency source content.
