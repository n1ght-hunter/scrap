use std::path::PathBuf;

use scrap_diagnostics::Level;

#[salsa::db]
#[derive(Clone, Default)]
pub struct ScrapDb {
    storage: salsa::Storage<Self>,
    emitter: scrap_diagnostics::DiagnosticEmitter,
}

#[salsa::db]
impl salsa::Database for ScrapDb {}

#[salsa::db]
pub trait Db: salsa::Database {
    /// get diagnostic handler
    fn dcx(&self) -> &scrap_diagnostics::DiagnosticEmitter;

    /// Fork this handle into an owned, cheaply-cloneable concrete database.
    ///
    /// Salsa storage is refcounted, so forks share state. Used to seed rayon
    /// workers without needing `Db: Sync` — each worker holds its own
    /// `ScrapDb` instead of sharing a `&dyn Db`.
    fn fork(&self) -> ScrapDb;
}

#[salsa::db]
impl Db for ScrapDb {
    fn dcx(&self) -> &scrap_diagnostics::DiagnosticEmitter {
        &self.emitter
    }

    fn fork(&self) -> ScrapDb {
        self.clone()
    }
}

#[salsa::tracked(debug, persist)]
pub struct InputFile<'db> {
    #[returns(ref)]
    pub path: PathBuf,
    #[returns(ref)]
    pub content: String,
}

#[salsa::tracked(debug, persist)]
pub struct InputPath<'db> {
    #[returns(ref)]
    pub path: PathBuf,
    #[returns(ref)]
    pub last_modified: std::time::SystemTime,
}

#[salsa::tracked(persist, returns(clone))]
pub fn get_input_path(
    db: &dyn Db,
    path: PathBuf,
    last_modified: std::time::SystemTime,
) -> InputPath<'_> {
    InputPath::new(db, path, last_modified)
}

#[salsa::tracked(persist, returns(clone))]
pub fn load_file<'db>(db: &'db dyn Db, input_path: InputPath<'db>) -> Option<InputFile<'db>> {
    tracing::debug!("Loading file: {}", input_path.path(db).display());
    let path = input_path.path(db);
    match std::fs::read_to_string(path) {
        Ok(content) => Some(InputFile::new(db, path.clone(), content)),
        Err(e) => {
            db.dcx()
                .emit_err(Level::ERROR.primary_title("Failed to read file").element(
                    Level::HELP.message(format!("Could not read file '{}': {}", path.display(), e)),
                ));
            None
        }
    }
}
