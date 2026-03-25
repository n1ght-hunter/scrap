#![feature(try_blocks)]

mod args;
mod cache;
mod parsing;
mod pretty;
mod utils;

use std::ffi::OsString;

use clap::Parser;
use salsa::Database;
use scrap_errors::SimpleError;

#[salsa::tracked(debug)]
struct TrackedArgs<'db> {
    pub args: args::Args,
}

pub fn run_compiler<I, T>(itr: I)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = args::Args::parse_from(itr);

    if args.verbose {
        scrap_errors::set_verbose_errors(true);
    }

    let mut db = scrap_shared::salsa::ScrapDb::default();

    if let Some(cache_path) = args.cache.as_ref() {
        cache::load_cache(&mut db, cache_path);
    }

    let res = run(&args, &mut db);
    handle_diagnostics(&db).sexpect("Compilation failed");
    res.sexpect("Compilation failed");

    if let Some(cache_path) = args.cache.as_ref() {
        cache::save_cache(&mut db, cache_path);
    }
}

fn run(args: &args::Args, db_mut: &mut scrap_shared::salsa::ScrapDb) -> anyhow::Result<()> {
    let db = &*db_mut;

    // Phase 1: Parse files
    let mut files = parsing::parse_input_files(args, db);
    handle_diagnostics(db)?;

    let entry_file = files
        .pop()
        .ok_or_else(|| anyhow::anyhow!("No entry file found"))?;
    let other_files = files;

    // Phase 1.5: Module resolution
    let modules = utils::collect_modules(db, entry_file, other_files.clone());
    let resolved_can = parsing::resolve_modules(db, &modules, entry_file);

    let mode = pretty::PpMode::determine_pp_mode(args);

    // Pretty print AST if requested
    if let Some(mode) = mode
        && mode.needs_ast()
    {
        db.attach(|db| {
            pretty::print(db, mode, pretty::CompilationOutput::Ast(resolved_can));
        });
    }

    // Phase 2: Type checking
    let type_table = scrap_tycheck::check_types(db, resolved_can, entry_file.file(db));
    handle_diagnostics(db)?;

    // Phase 3: Lower to IR with type information
    let (entry_ir, other_ir) =
        utils::lower_input_files_to_ir(db, entry_file, other_files.to_vec(), type_table);

    handle_diagnostics(db)?;

    if let Some(mode) = mode
        && mode.needs_ir()
    {
        let lowered_ir = utils::create_lowered_ir(db, entry_ir, other_ir.clone());

        db.attach(|db| {
            pretty::print(db, mode, pretty::CompilationOutput::Ir(lowered_ir));
        });
    }

    // Phase 4: Code generation (when no pretty-print mode is active)
    if mode.is_none() {
        let lowered_ir = utils::create_lowered_ir(db, entry_ir, other_ir);

        let obj_bytes = scrap_codegen::compile_to_object(db, lowered_ir.can(db));
        handle_diagnostics(db)?;

        let obj_bytes = obj_bytes.unwrap(); // safe: handle_diagnostics would have bailed

        let out_dir = std::path::Path::new("target/scrap");
        std::fs::create_dir_all(out_dir)?;
        let obj_path = out_dir.join(format!("{}.obj", args.crate_name));
        std::fs::write(&obj_path, &obj_bytes)?;

        // Link with lld-link
        let exe_path = out_dir.join(format!("{}.exe", args.crate_name));

        // Find scrap_rt.lib — look for it relative to the compiler binary,
        // or in the target directory
        let rt_lib = find_scrap_rt_lib();

        let mut link_args: Vec<String> = vec![
            obj_path.to_str().unwrap().to_string(),
            "kernel32.lib".to_string(),
            "/SUBSYSTEM:CONSOLE".to_string(),
            "/ENTRY:_start".to_string(),
            format!("/OUT:{}", exe_path.display()),
        ];
        if let Some(ref rt) = rt_lib {
            link_args.push(rt.to_str().unwrap().to_string());
            // System libraries required by Rust's std (bundled in the staticlib)
            link_args.extend(
                [
                    "advapi32.lib",
                    "bcrypt.lib",
                    "msvcrt.lib",
                    "ntdll.lib",
                    "userenv.lib",
                    "ws2_32.lib",
                    // Modern MSVC CRT components
                    "vcruntime.lib",
                    "ucrt.lib",
                ]
                .iter()
                .map(std::string::ToString::to_string),
            );
        }

        let status = std::process::Command::new("lld-link.exe")
            .args(&link_args)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run lld-link.exe: {e}"))?;

        if !status.success() {
            anyhow::bail!(
                "Linking failed with exit code: {}",
                status.code().unwrap_or(-1)
            );
        }

        eprintln!("Compiled to {}", exe_path.display());
    }

    Ok(())
}

/// Find `scrap_rt.lib` by searching common locations.
fn find_scrap_rt_lib() -> Option<std::path::PathBuf> {
    // Check common build output directories
    let candidates = [
        // Standalone crate build output (scrap_rt is outside the workspace)
        "crates/scrap_rt/target/release/scrap_rt.lib",
        "crates/scrap_rt/target/debug/scrap_rt.lib",
        // Legacy workspace build paths
        "target/release/scrap_rt.lib",
        "target/debug/scrap_rt.lib",
        "target/x86_64-pc-windows-msvc/release/scrap_rt.lib",
        "target/x86_64-pc-windows-msvc/debug/scrap_rt.lib",
    ];
    for candidate in &candidates {
        let path = std::path::PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }

    // Check relative to the compiler binary
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let rt = dir.join("scrap_rt.lib");
        if rt.exists() {
            return Some(rt);
        }
    }

    None
}

/// Handle diagnostics after a compilation phase
/// Renders all diagnostics and returns an error if there are any errors emitted
fn handle_diagnostics(db: &dyn scrap_shared::Db) -> anyhow::Result<()> {
    db.dcx().render_all();
    if db.dcx().has_errors() {
        let (errors, warnings, _) = db.dcx().counts();
        if warnings > 0 {
            anyhow::bail!(
                "Compilation completed with {} warnings and {} errors.",
                warnings,
                errors
            );
        } else {
            anyhow::bail!("Compilation failed with {} errors.", errors);
        }
    }
    Ok(())
}
