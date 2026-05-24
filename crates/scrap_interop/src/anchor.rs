//! Generation and cargo-build of the anchor crate.
//!
//! The anchor is the single `crate-type = ["staticlib"]` that depends on the
//! user's Rust crates and on `scrap_rt` (consumed as an rlib). Building it
//! produces one archive that bundles `std` exactly once — the fix for the
//! duplicate-`std` linker clash — which the driver links into the Scrap exe in
//! place of the standalone `scrap_rt.lib`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};
use target_lexicon::{BinaryFormat, Triple};

use crate::cache::cache_key;
use crate::schema::RustDeps;

/// A generated drop wrapper to emit into the anchor: a v0-mangled `pub fn` that
/// forces `drop_in_place::<T>` glue into the archive and gives it an externally
/// linkable symbol (resolved from the v2 metadata, not `#[no_mangle]`).
pub struct DropWrapper {
    /// Unique sanitized suffix derived from the full type path.
    pub sanitized: String,
    /// The full Rust type path, e.g. `rmeta_fixture::Wrapper`.
    pub full_path: String,
}

/// Inputs for one anchor build.
pub struct AnchorRequest<'a> {
    /// The user's declared Rust dependencies. Empty → no anchor is built.
    pub rust_deps: &'a RustDeps,
    /// Per-type drop wrappers to generate (empty on the first/metadata-only build;
    /// populated on the second build once the droppable type set is known).
    pub drop_wrappers: &'a [DropWrapper],
    /// Absolute path to the `scrap_rt` crate directory (depended on as an rlib).
    pub scrap_rt_crate_dir: &'a Path,
    /// Absolute path to the `tools/scrap-rustc` driver crate directory. The
    /// driver is built on demand and run as cargo's `RUSTC_WORKSPACE_WRAPPER` to
    /// dump interop metadata for the anchor.
    pub scrap_rustc_crate_dir: &'a Path,
    /// The target triple the Scrap program is being compiled for.
    pub target: &'a Triple,
    /// The pinned toolchain channel (e.g. `nightly-2026-02-10`).
    pub toolchain_channel: &'a str,
    /// The compiler output root (`target/scrap`); the anchor lives under it.
    pub out_root: &'a Path,
    /// Whether to build the anchor in release mode.
    pub release: bool,
}

/// The product of a successful anchor build.
pub struct AnchorArtifact {
    /// Path to the produced static archive.
    pub archive: PathBuf,
    /// The interop metadata the driver dumped for the requested dep crates, if
    /// the dump was produced and read back successfully.
    pub metadata: Option<scrap_rmeta::RustMetadata>,
}

/// Build the anchor crate for `req`, reusing a cached archive when the inputs
/// are unchanged. Returns `Ok(None)` when no Rust dependencies are declared (the
/// driver then keeps the plain `scrap_rt.lib` link path).
pub fn build_anchor(req: &AnchorRequest) -> anyhow::Result<Option<AnchorArtifact>> {
    if req.rust_deps.is_empty() {
        return Ok(None);
    }

    let key = cache_key(
        req.rust_deps,
        &req.target.to_string(),
        req.toolchain_channel,
        req.release,
        req.drop_wrappers,
    );
    // Absolute so cargo's `current_dir` + `--manifest-path` stay consistent and
    // the generated `rust-toolchain.toml` is discovered from the anchor dir.
    let abs_out_root = if req.out_root.is_absolute() {
        req.out_root.to_path_buf()
    } else {
        std::env::current_dir()?.join(req.out_root)
    };
    let anchor_dir = abs_out_root.join("anchor").join(&key);
    let target_dir = anchor_dir.join(".cargo-target");
    let stamp_path = anchor_dir.join("stamp.json");
    let metadata_path = anchor_dir.join("metadata.json");

    if let Some(archive) = read_stamp(&stamp_path, &key) {
        return Ok(Some(AnchorArtifact {
            archive,
            metadata: read_metadata(&metadata_path),
        }));
    }

    let driver_bin = crate::driver::ensure_driver(req.scrap_rustc_crate_dir)?;

    generate_files(req, &anchor_dir).with_context(|| {
        format!(
            "failed to generate anchor crate at {}",
            anchor_dir.display()
        )
    })?;

    let archive = run_cargo(req, &anchor_dir, &target_dir, &driver_bin, &metadata_path)?;
    write_stamp(&stamp_path, &key, &archive)?;

    Ok(Some(AnchorArtifact {
        archive,
        metadata: read_metadata(&metadata_path),
    }))
}

/// Comma-separated crate names (Cargo dep keys, hyphens normalized to
/// underscores) the driver should dump metadata for.
fn dep_crate_list(req: &AnchorRequest) -> String {
    req.rust_deps
        .0
        .keys()
        .map(|k| k.replace('-', "_"))
        .collect::<Vec<_>>()
        .join(",")
}

/// Read back a metadata dump, returning `None` (rather than failing the build)
/// if it is absent or unreadable. A dump whose `schema_version` doesn't match the
/// one we compile against is rejected (treated as absent) rather than misread —
/// fields may have shifted meaning between versions.
fn read_metadata(path: &Path) -> Option<scrap_rmeta::RustMetadata> {
    let text = std::fs::read_to_string(path).ok()?;
    let metadata: scrap_rmeta::RustMetadata = serde_json::from_str(&text).ok()?;
    if metadata.schema_version != scrap_rmeta::SCHEMA_VERSION {
        return None;
    }
    Some(metadata)
}

/// The anchor's static-archive filename for the target format.
fn archive_name(target: &Triple) -> &'static str {
    match target.binary_format {
        BinaryFormat::Coff => "scrap_anchor.lib",
        _ => "libscrap_anchor.a",
    }
}

fn generate_files(req: &AnchorRequest, anchor_dir: &Path) -> anyhow::Result<()> {
    let src_dir = anchor_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    std::fs::write(anchor_dir.join("Cargo.toml"), cargo_toml(req))?;
    std::fs::write(
        anchor_dir.join("rust-toolchain.toml"),
        format!("[toolchain]\nchannel = \"{}\"\n", req.toolchain_channel),
    )?;
    std::fs::write(src_dir.join("lib.rs"), lib_rs(req))?;
    Ok(())
}

fn cargo_toml(req: &AnchorRequest) -> String {
    let mut out = String::new();
    out.push_str("[package]\n");
    out.push_str("name = \"scrap_anchor\"\n");
    out.push_str("version = \"0.0.0\"\n");
    out.push_str("edition = \"2024\"\n");
    out.push_str("publish = false\n\n");
    // Detach from the surrounding scrap workspace.
    out.push_str("[workspace]\n\n");
    out.push_str("[lib]\n");
    out.push_str("crate-type = [\"staticlib\"]\n");
    out.push_str("path = \"src/lib.rs\"\n\n");
    // No landing pads / SEH in Scrap frames → unwinding across the boundary is
    // UB; abort sidesteps it. Set in both profiles since `--release` is per-build.
    out.push_str("[profile.dev]\npanic = \"abort\"\n\n");
    out.push_str("[profile.release]\npanic = \"abort\"\n\n");
    out.push_str("[dependencies]\n");
    out.push_str(&format!(
        "scrap_rt = {{ path = '{}' }}\n",
        req.scrap_rt_crate_dir.display()
    ));
    for (name, spec) in &req.rust_deps.0 {
        out.push_str(&spec.render_line(name));
        out.push('\n');
    }
    out
}

fn lib_rs(req: &AnchorRequest) -> String {
    let mut out = String::new();
    out.push_str("#![allow(unused, dead_code, unused_extern_crates)]\n\n");
    out.push_str("extern crate scrap_rt;\n");
    for name in req.rust_deps.0.keys() {
        // `extern crate` wants the crate's lib name (hyphens → underscores).
        out.push_str(&format!("extern crate {};\n", name.replace('-', "_")));
    }
    // Keep at least one symbol so the staticlib is never empty.
    out.push_str("\n#[used]\nstatic _ANCHOR_KEEP: extern \"C\" fn() = scrap_anchor_keep;\n");
    out.push_str("extern \"C\" fn scrap_anchor_keep() {}\n");
    // Drop wrappers force `drop_in_place::<T>` glue into the archive and export
    // it. A staticlib only exports `#[export_name]`/`#[no_mangle]`/extern symbols
    // (a plain `pub fn` is internalized), so we set an explicit export name —
    // derived from the FULL crate path, so it is unique per type and cannot
    // collide (the concern that rules out a fixed `#[no_mangle]` name).
    for w in req.drop_wrappers {
        out.push_str(&format!(
            "#[unsafe(export_name = \"__scrap_drop_in_place__{name}\")]\n\
             pub extern \"C\" fn __scrap_drop_in_place__{name}(p: *mut {path}) {{ unsafe {{ core::ptr::drop_in_place(p) }} }}\n",
            name = w.sanitized, path = w.full_path
        ));
    }
    out
}

fn run_cargo(
    req: &AnchorRequest,
    anchor_dir: &Path,
    target_dir: &Path,
    driver_bin: &Path,
    metadata_out: &Path,
) -> anyhow::Result<PathBuf> {
    let triple = req.target.to_string();
    let mut cmd = Command::new("cargo");
    cmd.current_dir(anchor_dir)
        // Authoritative over any inherited `[build] rustflags`; v0 mangling now
        // so Phase 2's symbol extraction is deterministic.
        .env("RUSTFLAGS", "-C symbol-mangling-version=v0")
        // Disable any ambient rustc wrapper (e.g. sccache from cargo config): an
        // empty value overrides config, and a second wrapper would probe
        // scrap-rustc as if it were the compiler and break. cargo composes
        // RUSTC_WRAPPER *outside* RUSTC_WORKSPACE_WRAPPER.
        .env("RUSTC_WRAPPER", "")
        // Run scrap-rustc as the wrapper for the anchor crate only; it compiles
        // the anchor and dumps interop metadata for the requested dep crates.
        .env("RUSTC_WORKSPACE_WRAPPER", driver_bin)
        .env("SCRAP_RMETA_OUT", metadata_out)
        .env("SCRAP_RMETA_CRATES", dep_crate_list(req))
        .arg("build");
    if req.release {
        cmd.arg("--release");
    }
    cmd.arg("--target")
        .arg(&triple)
        .arg("--target-dir")
        .arg(target_dir)
        .arg("--manifest-path")
        .arg(anchor_dir.join("Cargo.toml"))
        .arg("--message-format=json-render-diagnostics");

    let output = cmd
        .output()
        .context("failed to spawn `cargo` for the Rust-interop anchor build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "anchor crate build failed (cargo exit {}):\n{}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(archive) = parse_artifact_archive(&stdout, req.target) {
        return Ok(archive);
    }

    // Fall back to the conventional cargo output path.
    let profile = if req.release { "release" } else { "debug" };
    let fallback = target_dir
        .join(&triple)
        .join(profile)
        .join(archive_name(req.target));
    if fallback.exists() {
        return Ok(fallback);
    }

    bail!(
        "anchor build succeeded but no staticlib artifact was found \
         (looked for {} in cargo output and at {})",
        archive_name(req.target),
        fallback.display()
    )
}

/// Scan cargo's JSON message stream for the `scrap_anchor` staticlib artifact.
fn parse_artifact_archive(stdout: &str, target: &Triple) -> Option<PathBuf> {
    let want = archive_name(target);
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if msg.get("reason").and_then(|r| r.as_str()) != Some("compiler-artifact") {
            continue;
        }
        if msg
            .get("target")
            .and_then(|t| t.get("name"))
            .and_then(|n| n.as_str())
            != Some("scrap_anchor")
        {
            continue;
        }
        let Some(files) = msg.get("filenames").and_then(|f| f.as_array()) else {
            continue;
        };
        for f in files {
            let Some(path) = f.as_str() else { continue };
            if Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == want)
            {
                return Some(PathBuf::from(path));
            }
        }
    }
    None
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Stamp {
    key: String,
    archive: PathBuf,
}

fn read_stamp(stamp_path: &Path, key: &str) -> Option<PathBuf> {
    let text = std::fs::read_to_string(stamp_path).ok()?;
    let stamp: Stamp = serde_json::from_str(&text).ok()?;
    if stamp.key == key && stamp.archive.exists() {
        Some(stamp.archive)
    } else {
        None
    }
}

fn write_stamp(stamp_path: &Path, key: &str, archive: &Path) -> anyhow::Result<()> {
    let stamp = Stamp {
        key: key.to_string(),
        archive: archive.to_path_buf(),
    };
    std::fs::write(stamp_path, serde_json::to_string_pretty(&stamp)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp(name: &str, contents: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("scrap_rmeta_test_{name}.json"));
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn read_metadata_accepts_current_schema() {
        let json = format!(
            r#"{{"schema_version":{},"target":"x86_64-pc-windows-msvc","crates":[]}}"#,
            scrap_rmeta::SCHEMA_VERSION
        );
        let path = write_temp("current", &json);
        assert!(read_metadata(&path).is_some());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn read_metadata_rejects_stale_schema() {
        let json = format!(
            r#"{{"schema_version":{},"target":"x86_64-pc-windows-msvc","crates":[]}}"#,
            scrap_rmeta::SCHEMA_VERSION.wrapping_add(1)
        );
        let path = write_temp("stale", &json);
        assert!(read_metadata(&path).is_none());
        let _ = std::fs::remove_file(&path);
    }
}
