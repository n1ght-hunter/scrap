use crate::path::{Path, PathKey};

/// Module identifier, interned by path (equality ignores spans/NodeIds)
#[salsa::interned(debug, persist)]
pub struct ModuleId<'db> {
    #[returns(ref)]
    pub path: PathKey<'db>,
}

impl<'db> ModuleId<'db> {
    /// Create a ModuleId from a Path
    pub fn from_path(db: &'db dyn crate::Db, path: &Path<'db>) -> Self {
        ModuleId::new(db, PathKey::new(path.clone()))
    }

    /// Get the path text as a string
    pub fn path_str(&self, db: &'db dyn crate::Db) -> String {
        self.path(db).path.to_string_db(db)
    }

    /// Compare with another ModuleId including span information
    pub fn eq_with_span(&self, other: &Self, db: &'db dyn crate::Db) -> bool {
        self.path(db).eq_with_span(other.path(db))
    }
}

#[salsa::interned(debug, persist)]
pub struct TypeId<'db> {
    #[returns(ref)]
    pub path: PathKey<'db>,
}

impl<'db> TypeId<'db> {
    /// Create a TypeId from a Path
    pub fn from_path(db: &'db dyn crate::Db, path: &Path<'db>) -> Self {
        TypeId::new(db, PathKey::new(path.clone()))
    }

    /// Get the path text as a string
    pub fn path_str(&self, db: &'db dyn crate::Db) -> String {
        self.path(db).path.to_string_db(db)
    }

    /// Compare with another TypeId including span information
    pub fn eq_with_span(&self, other: &Self, db: &'db dyn crate::Db) -> bool {
        self.path(db).eq_with_span(other.path(db))
    }
}
