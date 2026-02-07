use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use scrap_ast::{
    item::{Item, ItemKind},
    module::ModuleKind,
};
use scrap_diagnostics::Level;
use scrap_shared::id::ModuleId;

/// Parse all input files in parallel
pub fn parse_input_files<'db>(
    args: &crate::args::Args,
    db: &'db dyn scrap_shared::Db,
) -> Vec<scrap_parser::ParsedFile<'db>> {
    let root_path = &args.entry_source_file;

    let modules = args
        .source_files
        .par_iter()
        .chain(rayon::iter::once(&args.entry_source_file))
        .filter_map(|file_path| {
            let (is_root, root_path_segments) =
                crate::utils::compute_relative_path_segments(args, db, root_path, file_path)?;

            let modifed = match std::fs::metadata(file_path).and_then(|m| m.modified()) {
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
                scrap_shared::salsa::get_input_path(db, file_path.to_path_buf(), modifed);
            let input_file = scrap_shared::salsa::load_file(db, input_path)?;
            let lexed_tokens = scrap_lexer::lex_file(db, input_file);
            let parsed_file = scrap_parser::parse_tokens(
                db,
                input_file,
                lexed_tokens,
                is_root,
                root_path_segments,
            )?;
            Some(parsed_file)
        })
        .collect::<Vec<_>>();

    modules
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

pub fn resolve_modules<'db>(
    db: &'db dyn scrap_shared::Db,
    modules: &Modules<'db>,
    entry_file: scrap_parser::ParsedFile<'db>,
) -> scrap_ast::Can<'db> {
    let can = entry_file.ast(db).unwrap_can();
    let mut items = can.items(db).clone();
    items.par_iter_mut().for_each(|item| {
        if let Item {
            kind: ItemKind::Module(module),
            ..
        } = item.as_mut()
        {
            match module.kind(db) {
                ModuleKind::Unloaded => {
                    if let Some(resolved_module) = resolve_module_by_id(db, modules, &module.id(db))
                    {
                        // Recursively resolve the module
                        let recursively_resolved =
                            resolve_module_recursive(db, modules, resolved_module);
                        let _ = std::mem::replace(module, recursively_resolved);
                    }
                }
                ModuleKind::Loaded(..) => {
                    // Also resolve already loaded modules recursively
                    let recursively_resolved = resolve_module_recursive(db, modules, module);
                    let _ = std::mem::replace(module, recursively_resolved);
                }
            }
        }
    });

    // Return the resolved AST
    create_can(db, can.id(db), *can.name(db), items)
}

fn resolve_module_recursive<'db>(
    db: &'db dyn scrap_shared::Db,
    modules: &Modules<'db>,
    module: &scrap_ast::module::Module<'db>,
) -> scrap_ast::module::Module<'db> {
    // Match on the module kind to get items
    match module.kind(db) {
        ModuleKind::Loaded(items, inline, span) => {
            let mut new_items = items.clone();

            // Recursively resolve all nested modules
            new_items.par_iter_mut().for_each(|item| {
                if let Item {
                    kind: ItemKind::Module(nested_module),
                    ..
                } = item.as_mut()
                {
                    match nested_module.kind(db) {
                        ModuleKind::Unloaded => {
                            // Resolve unloaded modules from the modules hashmap
                            if let Some(resolved_nested) =
                                resolve_module_by_id(db, modules, &nested_module.id(db))
                            {
                                // Recursive call to resolve nested modules
                                let recursively_resolved =
                                    resolve_module_recursive(db, modules, resolved_nested);
                                let _ = std::mem::replace(nested_module, recursively_resolved);
                            }
                        }
                        ModuleKind::Loaded(..) => {
                            // Also recursively resolve already loaded modules
                            let recursively_resolved =
                                resolve_module_recursive(db, modules, nested_module);
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
            if let Some(resolved) = resolve_module_by_id(db, modules, &module.id(db)) {
                // Recursively resolve the newly loaded module
                resolve_module_recursive(db, modules, resolved)
            } else {
                // If still unresolved, return as is
                module.clone()
            }
        }
    }
}

fn resolve_module_by_id<'a, 'db>(
    db: &'db dyn scrap_shared::Db,
    modules: &'a Modules<'db>,
    module_id: &scrap_shared::id::ModuleId<'db>,
) -> Option<&'a scrap_ast::module::Module<'db>> {
    // ModuleIds are now interned by path string, so direct lookup works
    if let Some(mo) = modules.get(module_id) {
        Some(mo)
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
