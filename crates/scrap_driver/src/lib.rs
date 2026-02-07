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

pub fn run_complier<I, T>(itr: I)
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
        let status = std::process::Command::new("lld-link.exe")
            .args([
                obj_path.to_str().unwrap(),
                "kernel32.lib",
                "/SUBSYSTEM:CONSOLE",
                "/ENTRY:_start",
                &format!("/OUT:{}", exe_path.display()),
            ])
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run lld-link.exe: {e}"))?;

        if !status.success() {
            anyhow::bail!("Linking failed with exit code: {}", status.code().unwrap_or(-1));
        }

        eprintln!("Compiled to {}", exe_path.display());
    }

    Ok(())
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
