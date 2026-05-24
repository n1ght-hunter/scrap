# scrap-rustc

The Rust-interop **metadata driver**: a `rustc_private` wrapper that compiles the generated anchor
crate and extracts the [`scrap_rmeta`](../../crates/scrap_rmeta) metadata the Scrap compiler needs to
mirror Rust types and call Rust functions with the real Rust ABI.

## How it's used

`scrap_interop` sets this binary as cargo's `RUSTC_WORKSPACE_WRAPPER` when building the anchor. cargo
then invokes it as `scrap-rustc <real-rustc> <rustc-args…>`:

- **Anchor compile** (`crate_name == scrap_anchor`): runs an in-process `rustc_driver` that compiles
  the staticlib *and*, in `after_analysis`, walks the `TyCtxt` (`extract.rs`) to dump the catalog —
  per-crate public fns/types, v0-mangled symbols, `FnAbiInfo` (the real `PassMode` per arg/return),
  and concrete `LayoutInfo` — as JSON to `$SCRAP_RMETA_OUT`.
- **Every other crate** (std, the user's dependencies, version probes): transparently forwards to the
  real rustc. Only the anchor's `TyCtxt` is dumped, and it already sees every dependency through it.

## Toolchain

Built with the pinned nightly + `rustc-dev` component in `rust-toolchain.toml`. That channel must
match `scrap_interop::PINNED_TOOLCHAIN`. When cargo doesn't pass `--sysroot`, the driver injects one
(a `rustc_private` tool can't locate its sysroot relative to its own exe).

This is an internal build tool, not part of the published compiler.
