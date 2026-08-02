# scrap_rmeta

The cross-process metadata schema shared between the [`scrap-rustc`](../../tools/scrap-rustc)
driver and the Scrap compiler (`scrapc`) for native Rust interop.

`scrap-rustc` links `rustc_private` and is built by a pinned nightly with `rustc-dev`; `scrapc` is
built by an ordinary toolchain. This crate is the one thing both link, so it is deliberately
**serde-only with no compiler dependencies**. The driver derives every value here from a single
`TyCtxt`, which is why the layouts and ABI `scrapc` mirrors — and the symbols it links — cannot
disagree with what the anchor archive actually contains.

## What it describes

For one *anchor* compilation (the staticlib that depends on the user's Rust crates), the dump
(`RustMetadata`) records, per dependency crate:

- **Functions** (`RustFn`) — free functions and inherent associated fns, with their per-argument /
  return ABI (`FnAbiInfo` → `PassMode`: `Ignore` / `Direct` / `Pair` / `Indirect` / `Cast`) and the
  exact v0-mangled `symbol` for monomorphic instances.
- **Types** (`RustType`) — structs, enums, and unions: `repr`, fields, variants, `#[non_exhaustive]`,
  inherent methods, and a concrete `LayoutInfo` (size, align, field offsets, `needs_drop`, `is_copy`)
  for non-generic types so `scrapc` can mirror the in-memory layout.

`SCHEMA_VERSION` is bumped when the shape changes incompatibly, so a stale dump is rejected rather
than misread.
