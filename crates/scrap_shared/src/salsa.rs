use std::path::PathBuf;

#[salsa::db]
#[derive(Clone, Default)]
pub struct ScrapDb {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for ScrapDb {}

#[salsa::db]
pub trait Db: salsa::Database {}

#[salsa::db]
impl Db for ScrapDb {}

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
pub fn load_file<'db>(db: &'db dyn Db, input_path: InputPath<'db>) -> InputFile<'db> {
    tracing::debug!("Loading file: {}", input_path.path(db).display());
    let path = input_path.path(db);
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", path.display(), e));
    InputFile::new(db, path.clone(), content)
}
