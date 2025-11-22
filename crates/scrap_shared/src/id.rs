use crate::path::Path;



#[salsa::interned(debug, persist)]
pub struct ModuleId {
    #[returns(ref)]
    pub path: Path<'db>,
}

#[salsa::interned(debug, persist)]
pub struct TypeId {
    #[returns(ref)]
    pub path: Path<'db>,
}