# scrap_interop

Build orchestration for Scrap's native Rust interop. Given a Scrap manifest's `[rust.dependencies]`
table, it produces a linkable archive plus the [`scrap_rmeta`](../scrap_rmeta) metadata the compiler
needs to call into those Rust crates.

## How it works

1. **Parse** the `[rust.dependencies]` table (`schema`): version/path/git dep specs, rendered back
   into the generated anchor's `Cargo.toml`.
2. **Generate the anchor crate** (`anchor`): a single `staticlib` crate that depends on the user's
   Rust crates and on `scrap_rt` (as an `rlib`). It also emits per-type drop wrappers
   (`__scrap_drop_in_place__<sanitized_path>`, via `#[unsafe(export_name = …)]`) so the GC and stack
   RAII can run Rust destructors.
3. **Build it** with the pinned nightly toolchain through the `scrap-rustc` driver (`driver`
   bootstraps that tool), which compiles the anchor *and* dumps `scrap_rmeta` metadata from its
   `TyCtxt`.
4. **Cache** (`cache`): the build is content-addressed over the dependency set, target triple,
   toolchain channel, profile, drop-wrapper set, and a generator version, so an unchanged input reuses
   the previously built archive instead of re-running cargo.

The resulting `AnchorArtifact` (archive + metadata) is handed back to `scrap_driver`, which links it
into the compiled Scrap executable.

## Notes

- `PINNED_TOOLCHAIN` must match the workspace-root `rust-toolchain.toml` and the `scrap-rustc`
  driver's `rustc_private` build.
- The cache key currently does not hash path-dependency *source* content; after editing a local
  path-dep, clear `target/scrap/anchor` to force a rebuild.
