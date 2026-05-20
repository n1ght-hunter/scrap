#![feature(try_blocks)]

mod args;
mod cache;
mod link;
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
    let _type_table = scrap_tycheck::check_types(db, resolved_can, entry_file.file(db));
    handle_diagnostics(db)?;

    // Phase 3: Lower to IR with type information
    let (entry_ir, other_ir) = utils::lower_input_files_to_ir(
        db,
        entry_file,
        other_files.to_vec(),
        resolved_can,
        entry_file.file(db),
    );

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

        let obj_bytes =
            scrap_codegen::compile_to_object(db, lowered_ir.can(db), args.target.clone());
        handle_diagnostics(db)?;

        let obj_bytes = obj_bytes.unwrap(); // safe: handle_diagnostics would have bailed

        let out_dir = std::path::Path::new("target/scrap");
        std::fs::create_dir_all(out_dir)?;
        let obj_path = out_dir.join(format!("{}.obj", args.crate_name));
        std::fs::write(&obj_path, &obj_bytes)?;

        let exe_path = out_dir.join(format!("{}{}", args.crate_name, exe_suffix(&args.target)));

        // Find the scrap_rt runtime archive — look for it relative to the
        // compiler binary, or in the target directory.
        let rt_lib = find_scrap_rt_lib(&args.target);

        link::link_executable(
            &args.target,
            &args.crate_name,
            &obj_path,
            &exe_path,
            rt_lib.as_deref(),
        )?;

        eprintln!("Compiled to {}", exe_path.display());
    }

    Ok(())
}

/// Executable filename suffix for the target (`.exe` for COFF/PE, empty otherwise).
fn exe_suffix(target: &target_lexicon::Triple) -> &'static str {
    match target.binary_format {
        target_lexicon::BinaryFormat::Coff => ".exe",
        _ => "",
    }
}

/// The `scrap_rt` static archive filename for the target (`scrap_rt.lib` for
/// COFF/PE, `libscrap_rt.a` otherwise).
fn rt_lib_name(target: &target_lexicon::Triple) -> &'static str {
    match target.binary_format {
        target_lexicon::BinaryFormat::Coff => "scrap_rt.lib",
        _ => "libscrap_rt.a",
    }
}

/// Find the `scrap_rt` static archive by searching common locations.
fn find_scrap_rt_lib(target: &target_lexicon::Triple) -> Option<std::path::PathBuf> {
    let name = rt_lib_name(target);

    // Build output directories, relative to the current working directory.
    let dirs = [
        // Standalone crate build output (scrap_rt may be built on its own).
        "crates/scrap_rt/target/release",
        "crates/scrap_rt/target/debug",
        // Workspace build paths.
        "target/release",
        "target/debug",
        "target/x86_64-pc-windows-msvc/release",
        "target/x86_64-pc-windows-msvc/debug",
        "target/x86_64-unknown-linux-gnu/release",
        "target/x86_64-unknown-linux-gnu/debug",
        "target/x86_64-apple-darwin/release",
        "target/x86_64-apple-darwin/debug",
    ];
    for dir in &dirs {
        let path = std::path::Path::new(dir).join(name);
        if path.exists() {
            return Some(path);
        }
    }

    // Check relative to the compiler binary.
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let rt = dir.join(name);
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
