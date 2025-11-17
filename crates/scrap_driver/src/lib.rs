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

fn run(args: &args::Args, db_mut: &mut scrap_shared::salsa::ScrapDb) -> anyhow::Result<()> {
    let base_emiter = scrap_diagnostics::DiagnosticEmitter::new().with_auto_render(true);
    let db = &*db_mut;
    let root_path = &args.entry_source_file;

    let mut modules = args
        .source_files
        .par_iter()
        .chain(rayon::iter::once(&args.entry_source_file))
        .filter_map(|file_path| {
            let db = db;

            let (is_root, root_path_segments) =
                compute_relative_path_segments(args, &base_emiter, root_path, file_path)?;

            let modifed = match std::fs::metadata(file_path).and_then(|m| m.modified()) {
                Ok(m) => m,
                Err(e) => {
                    base_emiter.render_single(
                        Level::ERROR
                            .primary_title(format!(
                                "Failed to read source file: {}",
                                file_path.display()
                            ))
                            .element(Level::HELP.message(format!("I/O Error: {}", e))),
                    );
                    return None;
                }
            };
            let input_path =
                scrap_shared::salsa::get_input_path(db, file_path.to_path_buf(), modifed);
            let input_file = scrap_shared::salsa::load_file(db, input_path);
            let lexed_tokens = scrap_lexer::lex_file(db, input_file);
            let parsed_file = scrap_parser::parse_tokens(
                db,
                input_file,
                lexed_tokens,
                is_root,
                root_path_segments,
            );
            Some(parsed_file)
        })
        .collect::<Vec<_>>();

    let entry_file = modules.pop().expect("No entry file found");

    if let Some(UnPrettyOut::Ast) = args.unpretty_out {
        let modules =
            modules
                .into_iter()
                .fold(std::collections::HashMap::new(), |mut acc, parsed_file| {
                    let (path, items) = parsed_file.ast(db).clone().into_module();
                    acc.insert(path.to_string_db(db), items);
                    acc
                });
        let filled_entry_file = entry_file.ast(db).unwrap_can().iter_modules_mut(|iter| {
            iter.filter(|m| matches!(m.1, scrap_ast::module::Module::Unloaded))
                .for_each(|(path, module)| {
                    let path = path.to_string_db(db);
                    if let Some(items) = modules.get(&path) {
                        *module = scrap_ast::module::Module::Loaded(
                            items.clone(),
                            scrap_ast::module::Inline::No,
                            scrap_span::new_dummy_span(db),
                        );
                    } else {
                        base_emiter.render_single(
                            Level::ERROR
                                .primary_title(format!("Failed to load module '{}'", path))
                                .element(Level::HELP.message(
                                    "Module not found among provided source files.".to_string(),
                                ))
                                .element(Level::NOTE.message(format!(
                                            "Available modules: {}",
                                            modules
                                                .keys()
                                                .map(|p| p.to_string())
                                                .collect::<Vec<_>>()
                                                .join(", ")
                                        ))),
                        );
                    }
                })
        });
        db.attach(|_| {
            println!("{:#?}", filled_entry_file);
        });
        return Ok(());
    }

    Ok(())
}

fn compute_relative_path_segments<'a, 'db>(
    args: &args::Args,
    base_emiter: &scrap_diagnostics::DiagnosticEmitter<'a>,
    root_path: &std::path::PathBuf,
    file_path: &std::path::PathBuf,
) -> Option<(bool, Vec<String>)> {
    if root_path == file_path {
        return Some((true, vec![args.crate_name.clone()]));
    }
    if !is_beside_or_below(root_path.as_path(), file_path.as_path()) {
        base_emiter.render_single(
            Level::ERROR
                .primary_title(format!(
                    "Source file '{}' is not beside or below the entry source file '{}'",
                    file_path.display(),
                    root_path.display()
                ))
                .element(Level::HELP.message(
                    "All source files must be located beside or below the entry source file in the filesystem hierarchy.".to_string(),
                )),
        );
        return None;
    }
    let Some(diff) = pathdiff::diff_paths(file_path, root_path) else {
        base_emiter.render_single(
            Level::ERROR
                .primary_title(format!(
                    "Failed to compute relative path for file: {}",
                    file_path.display()
                ))
                .element(
                    Level::HELP
                        .message("Ensure that all source files are on the same volume".to_string()),
                ),
        );
        return None;
    };
    let root_path_segments = std::iter::once(args.crate_name.clone())
        .chain(diff.components().skip(1).filter_map(|comp| {
            let os_str = comp.as_os_str();
            if let Some(s) = os_str.to_str() {
                if s.ends_with(".sc") {
                    Some(s[..s.len() - 3].to_string())
                } else {
                    Some(s.to_string())
                }
            } else {
                base_emiter.render_single(
                    Level::ERROR
                        .primary_title(format!(
                            "Non-unicode path segment in file path: {}",
                            file_path.display()
                        ))
                        .element(Level::HELP.message(
                            "Ensure that all source file paths are valid Unicode.".to_string(),
                        )),
                );
                None
            }
        }))
        .collect::<Vec<String>>();
    Some((false, root_path_segments))
}

fn is_beside_or_below(base_path: &std::path::Path, other_path: &std::path::Path) -> bool {
    // Get the parent directory of the base path.
    let Some(base_parent) = base_path.parent() else {
        // If base_path has no parent (e.g., "/"), nothing can be beside or below it
        // in this context.
        return false;
    };

    // Check if other_path is "beside" base_path.
    if let Some(other_parent) = other_path.parent()
        && base_parent == other_parent
    {
        return true;
    }

    // Check if other_path is "below" base_path.
    // This is true if other_path starts with the parent directory of base_path.
    let is_below = other_path.starts_with(base_parent);

    // The condition is met if it's beside OR below (and not the same file).
    // The is_below check implicitly handles the "beside" case if we check
    // that the paths are not identical.
    is_below && base_path != other_path
}
