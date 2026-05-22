//! Build orchestration for Scrap's native Rust interop.
//!
//! Phase 1 scope: parse the `[rust.dependencies]` table from a Scrap manifest,
//! generate the *anchor crate* (the single `staticlib` that depends on the
//! user's Rust crates and on `scrap_rt` as an rlib), build it with the pinned
//! toolchain, and hand the resulting archive back to the driver to link into the
//! compiled Scrap executable. No metadata/ABI extraction yet — that is Phase 2.

mod anchor;
mod cache;
mod driver;
mod schema;

pub use anchor::{AnchorArtifact, AnchorRequest, DropWrapper, build_anchor};
pub use schema::{DetailedDep, RustDepSpec, RustDeps, parse_manifest_rust_deps};

/// The nightly toolchain the interop build pins to. Must match the workspace
/// root `rust-toolchain.toml` (and the `scrap-rustc` driver's `rustc_private`).
pub const PINNED_TOOLCHAIN: &str = "nightly-2026-02-10";
