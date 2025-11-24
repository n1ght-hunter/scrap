use scrap_diagnostics::Level;
use scrap_shared::Db;

use crate::parsing::Modules;

pub fn compute_relative_path_segments<'a, 'db>(
    args: &crate::args::Args,
    db: &'db dyn scrap_shared::Db,
    root_path: &std::path::PathBuf,
    file_path: &std::path::PathBuf,
) -> Option<(bool, Vec<String>)> {
    if root_path == file_path {
        return Some((true, vec![args.crate_name.clone()]));
    }
    if !is_beside_or_below(root_path.as_path(), file_path.as_path()) {
        db.dcx().emit_err(
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
        db.dcx().emit_err(
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
                db.dcx().emit_err(
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

#[salsa::tracked]
pub fn collect_modules<'db>(
    db: &'db dyn scrap_shared::Db,
    entry_file: &'db scrap_parser::ParsedFile<'db>,
    other_files: &'db [scrap_parser::ParsedFile<'db>],
) -> Modules<'db> {
    let mut modules = hashbrown::HashMap::new();
    let entry_module = entry_file.ast(db).to_module(db);
    modules.insert(entry_module.id(db), entry_module);
    entry_file.modules(db).iter().for_each(|module| {
        modules.insert(module.id(db), module.clone());
    });

    other_files.iter().for_each(|file| {
        file.modules(db).iter().for_each(|module| {
            modules.insert(module.id(db), module.clone());
        });
    });

    modules
}
