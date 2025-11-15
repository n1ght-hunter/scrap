use std::path::PathBuf;

#[salsa::db]
#[derive(Clone, Default)]
pub struct ScrapDb {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for ScrapDb {}

#[salsa::input(debug, persist)]
pub struct InputFile {
    #[returns(ref)]
    pub path: PathBuf,
    #[returns(ref)]
    pub content: String,
}

#[salsa::tracked(persist)]
pub struct InputPath<'db> {
    #[returns(ref)]
    pub path: PathBuf,
    #[returns(ref)]
    pub last_modified: std::time::SystemTime,
}

#[salsa::tracked(persist)]
pub fn get_input_path(
    db: &dyn salsa::Database,
    path: PathBuf,
    last_modified: std::time::SystemTime,
) -> InputPath<'_> {
    InputPath::new(db, path, last_modified)
}

#[salsa::tracked(persist)]
pub fn load_file<'db>(db: &'db dyn salsa::Database, input_path: InputPath<'db>) -> InputFile {
    let path = input_path.path(db);
    println!("Loading file: {}", path.display());
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", path.display(), e));
    InputFile::new(db, path.clone(), content)
}
