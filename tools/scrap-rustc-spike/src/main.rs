//! Phase 0 feasibility spike for scrapc's Rust-interop driver.
//!
//! Validates the three rustc-private capabilities the whole interop design rests
//! on, against the *exact* pinned toolchain (see `rust-toolchain.toml`):
//!   1. building/linking a `rustc_driver`-based tool at all (the riskiest unknown
//!      is the build + runtime dylib setup, not the queries);
//!   2. `tcx.symbol_name(instance)` — the real mangled symbol for a fn;
//!   3. `tcx.fn_abi_of_instance(..)` — per-arg/return ABI pass modes;
//!   4. `tcx.layout_of(..)` — exact size/align/field offsets for a struct.
//!
//! Run it like a rustc invocation on a sample file (see `run.ps1`):
//!   scrap-rustc-spike sample.rs --crate-type lib --edition 2021
//!
//! `--sysroot` is injected automatically if absent: a rustc-private tool cannot
//! locate the sysroot relative to its own exe the way the real rustc proxy does.

#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

use rustc_driver::{Callbacks, Compilation};
use rustc_hir::def::DefKind;
use rustc_interface::interface::Compiler;
use rustc_middle::ty::{Instance, Ty, TyCtxt, TypingEnv};

struct SpikeCallbacks;

impl Callbacks for SpikeCallbacks {
    fn after_analysis<'tcx>(&mut self, _compiler: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        let items = tcx.hir_crate_items(());

        let mut generic_fns = Vec::new();
        for id in items.free_items() {
            let def_id = id.owner_id.to_def_id();
            let kind = tcx.def_kind(def_id);
            let path = tcx.def_path_str(def_id);

            match kind {
                DefKind::Fn if tcx.generics_of(def_id).own_params.is_empty() => {
                    report_fn(tcx, def_id, &path)
                }
                DefKind::Fn => generic_fns.push(def_id),
                DefKind::Struct | DefKind::Enum | DefKind::Union => {
                    report_adt_layout(tcx, def_id, &path)
                }
                _ => {}
            }
        }

        eprintln!("\n=== synthetic monomorphized instances ===");
        report_generic_instances(tcx, &generic_fns);
        report_std_type_layouts(tcx);

        Compilation::Continue
    }
}

/// Instantiate each single-type-param generic fn at `i32` and report the
/// resulting monomorphized symbol + ABI — the catalog→instantiation core of Phase 2.
fn report_generic_instances<'tcx>(tcx: TyCtxt<'tcx>, generic_fns: &[rustc_span::def_id::DefId]) {
    for &def_id in generic_fns {
        let generics = tcx.generics_of(def_id);
        if generics.own_params.len() != 1 {
            continue;
        }
        let path = tcx.def_path_str(def_id);
        let args = tcx.mk_args(&[tcx.types.i32.into()]);
        let typing_env = TypingEnv::fully_monomorphized();
        let instance = Instance::expect_resolve(tcx, typing_env, def_id, args, rustc_span::DUMMY_SP);

        eprintln!("fn  {path}::<i32>");
        eprintln!("    symbol = {}", tcx.symbol_name(instance));
        match tcx.fn_abi_of_instance(typing_env.as_query_input((instance, ty::List::empty()))) {
            Ok(fn_abi) => {
                for (i, arg) in fn_abi.args.iter().enumerate() {
                    eprintln!("    arg{i}  = {:?}", arg.mode);
                }
                eprintln!("    ret    = {:?}", fn_abi.ret.mode);
            }
            Err(e) => eprintln!("    fn_abi ERROR: {e:?}"),
        }
    }
}

/// Build generic args for `did` that assign `i32` to the first type param and
/// fall back to each remaining param's default (e.g. `Vec`'s `A = Global`).
/// Phase 2 must fill defaulted params, not just the user-visible ones.
fn args_first_type_i32<'tcx>(
    tcx: TyCtxt<'tcx>,
    did: rustc_span::def_id::DefId,
) -> ty::GenericArgsRef<'tcx> {
    use std::cell::Cell;
    let assigned_first = Cell::new(false);
    ty::GenericArgs::for_item(tcx, did, |param, args| match param.kind {
        ty::GenericParamDefKind::Lifetime => tcx.lifetimes.re_erased.into(),
        ty::GenericParamDefKind::Type { has_default, .. } => {
            if !assigned_first.get() {
                assigned_first.set(true);
                tcx.types.i32.into()
            } else if has_default {
                tcx.type_of(param.def_id).instantiate(tcx, args).into()
            } else {
                tcx.types.i32.into()
            }
        }
        ty::GenericParamDefKind::Const { .. } => {
            panic!("const generic param unsupported in spike")
        }
    })
}

/// Build `Vec<i32>` / `Option<i32>` from std diagnostic items and report their
/// layout — proves we can compute layouts for types defined in *dependency* crates.
fn report_std_type_layouts<'tcx>(tcx: TyCtxt<'tcx>) {
    use rustc_span::sym;
    let typing_env = TypingEnv::fully_monomorphized();

    for (name, sym_name) in [("Vec<i32>", sym::Vec), ("Option<i32>", sym::Option)] {
        let Some(did) = tcx.get_diagnostic_item(sym_name) else {
            eprintln!("ty  {name}: <no diagnostic item>");
            continue;
        };
        let args = args_first_type_i32(tcx, did);
        let adt_ty = Ty::new_adt(tcx, tcx.adt_def(did), args);
        match tcx.layout_of(typing_env.as_query_input(adt_ty)) {
            Ok(layout) => eprintln!(
                "ty  {name}: size = {} align = {} fields = {:?}",
                layout.size.bytes(),
                layout.align.abi.bytes(),
                layout.fields
            ),
            Err(e) => eprintln!("ty  {name}: layout ERROR: {e:?}"),
        }
    }
}

fn report_fn<'tcx>(tcx: TyCtxt<'tcx>, def_id: rustc_span::def_id::DefId, path: &str) {
    // Non-generic functions monomorphize trivially; generic ones are skipped in
    // the spike (the real driver substitutes the concrete args scrapc resolved).
    if !tcx.generics_of(def_id).own_params.is_empty() {
        eprintln!("fn  {path}: <generic, skipped in spike>");
        return;
    }

    let instance = Instance::mono(tcx, def_id);
    let symbol = tcx.symbol_name(instance);
    eprintln!("fn  {path}");
    eprintln!("    symbol = {symbol}");

    let typing_env = TypingEnv::fully_monomorphized();
    match tcx.fn_abi_of_instance(typing_env.as_query_input((instance, ty::List::empty()))) {
        Ok(fn_abi) => {
            eprintln!("    conv   = {:?}", fn_abi.conv);
            for (i, arg) in fn_abi.args.iter().enumerate() {
                eprintln!("    arg{i}  = {:?}", arg.mode);
            }
            eprintln!("    ret    = {:?}", fn_abi.ret.mode);
        }
        Err(e) => eprintln!("    fn_abi ERROR: {e:?}"),
    }
}

fn report_adt_layout<'tcx>(tcx: TyCtxt<'tcx>, def_id: rustc_span::def_id::DefId, path: &str) {
    if !tcx.generics_of(def_id).own_params.is_empty() {
        eprintln!("ty  {path}: <generic, skipped in spike>");
        return;
    }

    let ty = tcx.type_of(def_id).instantiate_identity();
    let typing_env = TypingEnv::fully_monomorphized();
    match tcx.layout_of(typing_env.as_query_input(ty)) {
        Ok(layout) => {
            eprintln!("ty  {path}");
            eprintln!(
                "    size = {} align = {}",
                layout.size.bytes(),
                layout.align.abi.bytes()
            );
            eprintln!("    fields = {:?}", layout.fields);
            eprintln!("    variants = {:?}", layout.variants);
        }
        Err(e) => eprintln!("ty  {path}: layout ERROR: {e:?}"),
    }
}

use rustc_middle::ty;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();

    if !args.iter().any(|a| a == "--sysroot" || a.starts_with("--sysroot=")) {
        if let Some(sysroot) = detect_sysroot() {
            args.push("--sysroot".to_string());
            args.push(sysroot);
        }
    }

    rustc_driver::catch_fatal_errors(|| {
        rustc_driver::run_compiler(&args, &mut SpikeCallbacks);
    })
    .unwrap();
}

fn detect_sysroot() -> Option<String> {
    let out = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8(out.stdout).ok()?.trim().to_string())
}
