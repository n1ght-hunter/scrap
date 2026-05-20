//! `scrap-rustc` — the Rust-interop metadata driver.
//!
//! Used as cargo's `RUSTC_WORKSPACE_WRAPPER` while building the generated anchor
//! crate. cargo invokes it as `scrap-rustc <real-rustc> <rustc-args…>`. For the
//! anchor compile it runs an in-process `rustc_driver` that compiles normally
//! *and* dumps `{catalog + per-instance symbol/fn_abi/layout}` (the schema in
//! `scrap_rmeta`) to the path in `SCRAP_RMETA_OUT`. For every other crate
//! (std, the user's deps, version probes) it transparently forwards to the real
//! rustc — only the anchor's `TyCtxt` is needed, and it sees all deps through it.

#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

mod extract;

use std::process::Command;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface::Compiler;
use rustc_middle::ty::TyCtxt;

/// The crate name of the generated anchor — the only compile we dump from.
const ANCHOR_CRATE: &str = "scrap_anchor";

struct DumpCallbacks {
    out: String,
    want_crates: Vec<String>,
}

impl Callbacks for DumpCallbacks {
    fn after_analysis<'tcx>(&mut self, _compiler: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        let target = tcx.sess.opts.target_triple.tuple().to_string();
        let metadata = extract::extract(tcx, &self.want_crates, target);
        match serde_json::to_string_pretty(&metadata) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.out, json) {
                    eprintln!("scrap-rustc: failed to write metadata to {}: {e}", self.out);
                }
            }
            Err(e) => eprintln!("scrap-rustc: failed to serialize metadata: {e}"),
        }
        // Continue so the anchor still compiles into its staticlib.
        Compilation::Continue
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();

    // Wrapper mode: argv = [scrap-rustc, <real rustc>, <rustc args…>].
    let real_rustc = argv.get(1).cloned();
    let rustc_args: Vec<String> = argv.iter().skip(2).cloned().collect();

    let crate_name = arg_value(&rustc_args, "--crate-name");
    let dump_out = std::env::var("SCRAP_RMETA_OUT").ok();
    let should_dump = dump_out.is_some() && crate_name.as_deref() == Some(ANCHOR_CRATE);

    if should_dump {
        let want_crates = std::env::var("SCRAP_RMETA_CRATES")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();

        // Build the rustc argv: program name + the flags cargo passed, with a
        // `--sysroot` injected if absent (a rustc-private tool can't locate it
        // relative to its own exe).
        let mut args: Vec<String> = std::iter::once("scrap-rustc".to_string())
            .chain(rustc_args.iter().cloned())
            .collect();
        if !args.iter().any(|a| a == "--sysroot" || a.starts_with("--sysroot=")) {
            if let Some(sysroot) = detect_sysroot() {
                args.push("--sysroot".to_string());
                args.push(sysroot);
            }
        }

        let mut callbacks = DumpCallbacks {
            out: dump_out.unwrap(),
            want_crates,
        };
        rustc_driver::catch_fatal_errors(|| {
            rustc_driver::run_compiler(&args, &mut callbacks);
        })
        .unwrap();
        return;
    }

    // Passthrough: forward to the real rustc for std/deps/probes.
    match real_rustc {
        Some(rustc) => {
            let status = Command::new(&rustc)
                .args(&rustc_args)
                .status()
                .unwrap_or_else(|e| panic!("scrap-rustc: failed to exec real rustc {rustc}: {e}"));
            std::process::exit(status.code().unwrap_or(1));
        }
        None => {
            eprintln!("scrap-rustc: no rustc provided (expected to run as RUSTC_WORKSPACE_WRAPPER)");
            std::process::exit(1);
        }
    }
}

/// Read the value of a `--flag value` / `--flag=value` pair from an arg list.
fn arg_value(args: &[String], flag: &str) -> Option<String> {
    let prefix = format!("{flag}=");
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == flag {
            return it.next().cloned();
        }
        if let Some(v) = a.strip_prefix(&prefix) {
            return Some(v.to_string());
        }
    }
    None
}

fn detect_sysroot() -> Option<String> {
    let out = Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8(out.stdout).ok()?.trim().to_string())
}
