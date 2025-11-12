use std::path::PathBuf;

#[salsa::db]
#[derive(Clone, Default)]
pub struct ScrapDb {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for ScrapDb {}

#[salsa::input(debug)]
pub struct InputFile {
    #[returns(ref)]
    pub path: PathBuf,
    #[returns(ref)]
    pub content: String,
}

#[salsa::input(debug)]
pub struct InputPath {
    #[returns(ref)]
    pub path: PathBuf,
}

#[salsa::tracked]
pub fn load_file<'db>(db: &'db dyn salsa::Database, path: InputPath) -> InputFile {
    let path = path.path(db);
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("Failed to read file {}: {}", path.display(), e)
    });
    InputFile::new(db, path.clone(), content)
}
