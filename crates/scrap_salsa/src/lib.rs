#[salsa::db]
#[derive(Clone, Default)]
pub struct ScrapDb {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for ScrapDb {}
