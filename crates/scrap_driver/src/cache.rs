use scrap_errors::SimpleError;
use std::path::Path;

/// Load database cache from disk if it exists
pub fn load_cache(db: &mut scrap_shared::salsa::ScrapDb, cache_path: &Path) {
    let db_cache_path = cache_path.with_extension("json");
    if db_cache_path.exists() {
        tracing::info!(
            "Loading database snapshot from '{}'",
            db_cache_path.display()
        );
        let serialized = std::fs::read_to_string(&db_cache_path)
            .sexpect("Failed to read JSON database snapshot for debugging purposes");
        <dyn salsa::Database>::deserialize(
            db,
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

/// Save database cache to disk
pub fn save_cache(db: &mut scrap_shared::salsa::ScrapDb, cache_path: &Path) {
    tracing::info!("Saving database snapshot to '{}'", cache_path.display());
    std::fs::create_dir_all(
        cache_path
            .parent()
            .expect("Failed to get parent directory of database snapshot path"),
    )
    .sexpect("Failed to create parent directory for database snapshot");
    let output = serde_json::to_string(&<dyn salsa::Database>::as_serialize(db)).sunwrap();
    std::fs::write(cache_path.with_extension("json"), output)
        .sexpect("Failed to write JSON database snapshot");
}
