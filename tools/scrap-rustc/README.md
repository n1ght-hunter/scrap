# scrap-rustc

The Rust-interop **metadata driver**: a `rustc_private` wrapper that compiles the generated anchor
crate and extracts the [`scrap_rmeta`](../../crates/scrap_rmeta) metadata the Scrap compiler needs to
mirror Rust types and call Rust functions with the real Rust ABI.

## How it's used

`scrap_interop` depends on this crate as an *artifact dependency* (`-Zbindeps`, enabled repo-wide in
`.cargo/config.toml`), so cargo builds the binary and passes its path as `CARGO_BIN_FILE_SCRAP_RUSTC`.
Making it a real edge in the build graph is deliberate: the driver depends on `scrap_rmeta`, so a
`SCHEMA_VERSION` bump rebuilds it automatically. The previous arrangement shelled out to `cargo build`
only on an anchor cache *miss*, so a bump with a warm cache left a stale binary emitting the old
schema while every dump it produced was rejected.

The dependency is optional, behind `scrap_interop/interop-driver`, because building it requires the
heavy `rustc-dev` component. An ordinary `cargo build` — and rust-analyzer — never pull it in. The
package is excluded from the workspace (`exclude = ["tools"]` in the root manifest) rather than being
a member, since members are built by any `--workspace` command regardless of features.

cargo runs the binary as its `RUSTC_WORKSPACE_WRAPPER` when building the anchor, invoking it as
`scrap-rustc <real-rustc> <rustc-args…>`:

- **Anchor compile** (`crate_name == scrap_anchor`): runs an in-process `rustc_driver` that compiles
  the staticlib *and*, in `after_analysis`, walks the `TyCtxt` (`extract.rs`) to dump the catalog —
  per-crate public fns/types, v0-mangled symbols, `FnAbiInfo` (the real `PassMode` per arg/return),
  and concrete `LayoutInfo` — as JSON to `$SCRAP_RMETA_OUT`.
- **Every other crate** (std, the user's dependencies, version probes): transparently forwards to the
  real rustc. Only the anchor's `TyCtxt` is dumped, and it already sees every dependency through it.

## Toolchain

Built with the pinned nightly + `rustc-dev` component in `rust-toolchain.toml`. That channel must
match `scrap_interop::PINNED_TOOLCHAIN` and the workspace root's `rust-toolchain.toml` — when cargo
builds this crate as an artifact dependency from the root, the *root* toolchain applies and the local
`rust-toolchain.toml` only takes effect for commands run from this directory. `rustc-dev` is
deliberately not listed in the root pin, so it is never auto-installed; run
`rustup component add rustc-dev` before building with `--features interop-driver`.

When cargo doesn't pass `--sysroot`, the driver injects one (a `rustc_private` tool can't locate its
sysroot relative to its own exe).

This is an internal build tool, not part of the published compiler.
