use std::path::PathBuf;

use scrap_diagnostics::Level;

#[salsa::db]
#[derive(Clone, Default)]
pub struct ScrapDb {
    storage: salsa::Storage<Self>,
    emitter: scrap_diagnostics::DiagnosticEmitter<'static>,
}

#[salsa::db]
impl salsa::Database for ScrapDb {}

#[salsa::db]
pub trait Db: salsa::Database + Sync {
    /// get diagnostic handler
    fn dcx<'a>(&'a self) -> &'a scrap_diagnostics::DiagnosticEmitter<'a>;
}

#[salsa::db]
impl Db for ScrapDb {
    fn dcx<'a>(&'a self) -> &'a scrap_diagnostics::DiagnosticEmitter<'a> {
        // SAFETY: 'a is tied to self, so this is safe
        #[allow(unsafe_code)]
        unsafe {
            std::mem::transmute(&self.emitter)
        }
    }
}

impl Drop for ScrapDb {
    fn drop(&mut self) {
        // clear to make sure all inner references are dropped first
        // may or maynot be necessary because of the transmute above
        self.emitter.clear();
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

#[salsa::tracked(persist)]
pub fn get_input_path(
    db: &dyn Db,
    path: PathBuf,
    last_modified: std::time::SystemTime,
) -> InputPath<'_> {
    InputPath::new(db, path, last_modified)
}

#[salsa::tracked(persist)]
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
