use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use scrap_ast::{
    item::{Item, ItemKind},
    module::ModuleKind,
};
use scrap_diagnostics::Level;
use scrap_shared::id::ModuleId;

/// Parse all input files in parallel.
///
/// Each worker holds its own `ScrapDb` clone (via [`Db::fork`]); the salsa
/// storage is refcounted, so clones share state. Tracked-struct ids are
/// returned across the thread boundary; the main thread reconstructs
/// `ParsedFile<'db>` against the outer db.
pub fn parse_input_files<'db>(
    args: &crate::args::Args,
    db: &'db dyn scrap_shared::Db,
) -> Vec<scrap_parser::ParsedFile<'db>> {
    use salsa::plumbing::{AsId, FromId};

    let root_path = &args.entry_source_file;

    let ids: Vec<salsa::Id> = args
        .source_files
        .par_iter()
        .chain(rayon::iter::once(&args.entry_source_file))
        .map_with(db.fork(), |db, file_path| {
            let db: &dyn scrap_shared::Db = db;
            let (is_root, root_path_segments) =
                crate::utils::compute_relative_path_segments(args, db, root_path, file_path)?;

            let modified = match std::fs::metadata(file_path).and_then(|m| m.modified()) {
                Ok(m) => m,
                Err(e) => {
                    db.dcx().emit_err(
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
                scrap_shared::salsa::get_input_path(db, file_path.to_path_buf(), modified);
            let input_file = scrap_shared::salsa::load_file(db, input_path)?;
            let lexed_tokens = scrap_lexer::lex_file(db, input_file);
            let parsed_file = scrap_parser::parse_tokens(
                db,
                input_file,
                lexed_tokens,
                is_root,
                root_path_segments,
            )?;
            Some(parsed_file.as_id())
        })
        .flatten()
        .collect();

    ids.into_iter()
        .map(<scrap_parser::ParsedFile<'db> as FromId>::from_id)
        .collect()
}

pub type Modules<'db> =
    indexmap::IndexMap<scrap_shared::id::ModuleId<'db>, scrap_ast::module::Module<'db>>;

#[salsa::tracked(persist)]
fn create_module<'db>(
    db: &'db dyn scrap_shared::Db,
    module_id: scrap_shared::id::ModuleId<'db>,
    module_kind: ModuleKind<'db>,
) -> scrap_ast::module::Module<'db> {
    scrap_ast::module::Module::new(db, module_id, module_kind)
}

#[salsa::tracked(persist)]
fn create_can<'db>(
    db: &'db dyn scrap_shared::Db,
    id: scrap_shared::NodeId,
    name: ModuleId<'db>,
    items: thin_vec::ThinVec<Box<scrap_ast::item::Item<'db>>>,
) -> scrap_ast::Can<'db> {
    scrap_ast::Can::new(db, id, name, items)
}

/// Module map keyed by raw `salsa::Id` instead of a `'db`-bound `ModuleId`.
///
/// Lifetime-free and `Send + Sync`, so worker threads can share `&ModulesById`
/// freely. Tracked structs are reconstructed per worker via `FromId`.
type ModulesById = std::collections::HashMap<salsa::Id, salsa::Id>;

fn modules_by_id<'db>(modules: &Modules<'db>) -> ModulesById {
    use salsa::plumbing::AsId;
    modules
        .iter()
        .map(|(k, v)| (k.as_id(), v.as_id()))
        .collect()
}

pub fn resolve_modules<'db>(
    db: &'db dyn scrap_shared::Db,
    modules: &Modules<'db>,
    entry_file: scrap_parser::ParsedFile<'db>,
) -> scrap_ast::Can<'db> {
    use salsa::plumbing::{AsId, FromId};

    let can = entry_file.ast(db).unwrap_can();
    let mut items = can.items(db).clone();
    let id_modules = modules_by_id(modules);

    // Each worker is seeded with its own `ScrapDb` clone (shared salsa
    // storage). Tracked structs are bridged across the worker/outer lifetime
    // boundary by raw `salsa::Id` — see `ModulesById`.
    items
        .par_iter_mut()
        .for_each_with(db.fork(), |db_local, item| {
            let db_local: &dyn scrap_shared::Db = db_local;
            if let Item {
                kind: ItemKind::Module(module),
                ..
            } = item.as_mut()
            {
                let module_local =
                    <scrap_ast::module::Module<'_> as FromId>::from_id(module.as_id());
                let resolved_local = match module_local.kind(db_local) {
                    ModuleKind::Unloaded => {
                        match resolve_module_by_id(db_local, &id_modules, module_local.id(db_local))
                        {
                            Some(m) => resolve_module_recursive(db_local, &id_modules, m),
                            None => return,
                        }
                    }
                    ModuleKind::Loaded(..) => {
                        resolve_module_recursive(db_local, &id_modules, module_local)
                    }
                };
                let resolved =
                    <scrap_ast::module::Module<'db> as FromId>::from_id(resolved_local.as_id());
                let _ = std::mem::replace(module, resolved);
            }
        });

    // Return the resolved AST
    create_can(db, can.id(db), *can.name(db), items)
}

fn resolve_module_recursive<'db>(
    db: &'db dyn scrap_shared::Db,
    modules: &ModulesById,
    module: scrap_ast::module::Module<'db>,
) -> scrap_ast::module::Module<'db> {
    // Match on the module kind to get items
    match module.kind(db) {
        ModuleKind::Loaded(items, inline, span) => {
            let mut new_items = items.clone();

            // Nested modules are resolved sequentially: trees are typically
            // shallow, so rayon work-stealing overhead would dominate.
            new_items.iter_mut().for_each(|item| {
                if let Item {
                    kind: ItemKind::Module(nested_module),
                    ..
                } = item.as_mut()
                {
                    match nested_module.kind(db) {
                        ModuleKind::Unloaded => {
                            if let Some(resolved_nested) =
                                resolve_module_by_id(db, modules, nested_module.id(db))
                            {
                                let recursively_resolved =
                                    resolve_module_recursive(db, modules, resolved_nested);
                                let _ = std::mem::replace(nested_module, recursively_resolved);
                            }
                        }
                        ModuleKind::Loaded(..) => {
                            let recursively_resolved =
                                resolve_module_recursive(db, modules, *nested_module);
                            let _ = std::mem::replace(nested_module, recursively_resolved);
                        }
                    }
                }
            });

            // Create a new module with resolved items
            create_module(
                db,
                module.id(db),
                ModuleKind::Loaded(new_items, *inline, *span),
            )
        }
        ModuleKind::Unloaded => {
            // If unloaded, try to resolve it from the modules hashmap
            if let Some(resolved) = resolve_module_by_id(db, modules, module.id(db)) {
                resolve_module_recursive(db, modules, resolved)
            } else {
                module
            }
        }
    }
}

fn resolve_module_by_id<'db>(
    db: &'db dyn scrap_shared::Db,
    modules: &ModulesById,
    module_id: scrap_shared::id::ModuleId<'db>,
) -> Option<scrap_ast::module::Module<'db>> {
    use salsa::plumbing::{AsId, FromId};

    if let Some(id) = modules.get(&module_id.as_id()) {
        Some(<scrap_ast::module::Module<'_> as FromId>::from_id(*id))
    } else {
        db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!("Unresolved module: {}", module_id.path_str(db)))
                .element(Level::HELP.message(
                    "Ensure that all modules are included in the source files.".to_string(),
                )),
        );
        None
    }
}
