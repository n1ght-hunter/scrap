#![feature(try_blocks)]

mod args;
mod cache;
mod parsing;
mod pretty;
mod utils;

use std::ffi::OsString;

use anyhow::Context;
use clap::Parser;
use salsa::Database;
use scrap_errors::SimpleError;
use scrap_shared::Db;

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

    run(&args, &mut db).sexpect("Compilation failed");

    if let Some(cache_path) = args.cache.as_ref() {
        cache::save_cache(&mut db, cache_path);
    }
}

fn run(args: &args::Args, db_mut: &mut scrap_shared::salsa::ScrapDb) -> anyhow::Result<()> {
    let db = &*db_mut;

    // Phase 1: Parse files
    let (entry_file, other_files) = parsing::parse_input_files(args, db)?;

    if db.dcx().has_errors() {
        db.dcx().render_all();
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

    let modules = utils::collect_modules(db, &entry_file, &other_files);

    let mode = pretty::PpMode::determine_pp_mode(args);

    if let Some(mode) = mode
        && mode.needs_ast()
    {
        let filled_entry_file = parsing::resolve_modules(db, &modules, entry_file)
            .context("failed to resolve modules")?;
        db.attach(|db| {
            pretty::print(db, mode, &filled_entry_file, None);
        });
        // if mode.needs_ir() {
        //     // Resolve modules, lower to IR and print
        //     let filled_entry_file =
        //         resolve_modules(args, db, &base_emiter, entry_file, &other_files)?;
        //     db.attach(|db| {
        //         let lowered_ir = lower_to_ir(args, db, entry_file, &other_files);
        //         pretty::print(db, mode, &filled_entry_file, Some(lowered_ir));
        //     });
        // } else {
        //     // Resolve modules and print AST
        //     let filled_entry_file =
        //         resolve_modules(args, db, &base_emiter, entry_file, &other_files)?;
        //     db.attach(|db| {
        //         pretty::print(db, mode, &filled_entry_file, None);
        //     });
        // }
        // return Ok(());
    }

    let (entry_ir, modules_ir) = utils::collect_modules_ir(db, &modules);

    if let Some(mode) = mode
        && mode.needs_ir()
    {
        let lowered_ir = utils::lower_to_ir(db, entry_file, other_files);
        db.attach(|db| {
            pretty::print(db, mode, &filled_entry_file, Some(&lowered_ir));
        });
    }

    Ok(())
}
