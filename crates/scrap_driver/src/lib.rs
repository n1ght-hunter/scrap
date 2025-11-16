#![feature(try_blocks)]

mod args;

use std::ffi::OsString;

use args::UnPrettyOut;
use clap::Parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use salsa::Database;
use scrap_diagnostics::Level;
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
        let db_cache_path = cache_path.with_extension("json");
        if db_cache_path.exists() {
            tracing::info!(
                "Loading database snapshot from '{}'",
                db_cache_path.display()
            );
            let serialized = std::fs::read_to_string(&db_cache_path)
                .sexpect("Failed to read JSON database snapshot for debugging purposes");
            <dyn salsa::Database>::deserialize(
                &mut db,
                &mut serde_json::Deserializer::from_str(&serialized),
            )
            .sunwrap();
        } else {
            tracing::info!(
                "Database snapshot file '{}' does not exist. Starting with a fresh database.",
                cache_path.display()
            );
        }
    }

    run(&args, &mut db).sexpect("Compilation failed");

    if let Some(cache_path) = args.cache.as_ref() {
        tracing::info!("Saving database snapshot to '{}'", cache_path.display());
        std::fs::create_dir_all(
            cache_path
                .parent()
                .expect("Failed to get parent directory of database snapshot path"),
        )
        .sexpect("Failed to create parent directory for database snapshot");
        let output = serde_json::to_string(&<dyn salsa::Database>::as_serialize(&mut db)).sunwrap();
        std::fs::write(cache_path.with_extension("json"), output)
            .sexpect("Failed to write JSON database snapshot");
    }
}

fn run(args: &args::Args, db: &mut scrap_shared::salsa::ScrapDb) -> anyhow::Result<()> {
    let mut base_emiter = scrap_diagnostics::DiagnosticEmitter::new().with_auto_render(true);

    let source_files = args
        .source_files
        .par_iter()
        .map(|path| (path, std::fs::metadata(path).and_then(|m| m.modified())))
        .collect_vec_list()
        .into_iter()
        .flatten()
        .flat_map(|(path, res)| match res {
            Ok(modified) => Some(scrap_shared::salsa::get_input_path(
                db,
                path.to_path_buf(),
                modified,
            )),
            Err(e) => {
                base_emiter.emit(
                    Level::ERROR
                        .primary_title(format!("Failed to read source file: {}", path.display()))
                        .element(Level::HELP.message(format!("I/O Error: {}", e))),
                );
                None
            }
        });
    let _source_files = source_files.collect::<Vec<_>>();

    let metadata = std::fs::metadata(&args.entry_source_file)?;
    let input_path = scrap_shared::salsa::get_input_path(
        db,
        args.entry_source_file.to_path_buf(),
        metadata.modified()?,
    );

    let input_file = scrap_shared::salsa::load_file(db, input_path);
    let lexed_tokens = scrap_lexer::lex_file(db, input_file);
    let parsed_file = scrap_parser::parse_tokens(db, input_file, lexed_tokens, true);

    if let Some(UnPrettyOut::Ast) = args.unpretty_out {
        db.attach(|db| {
            println!("{:#?}", parsed_file.ast(db));
        });
        return Ok(());
    }

    Ok(())
}
