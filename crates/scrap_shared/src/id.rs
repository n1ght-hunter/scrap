use crate::path::Path;



#[salsa::interned]
pub struct ModuleId {
    #[returns(ref)]
    pub path: Path<'db>,
}

#[salsa::interned]
pub struct TypeId {
    #[returns(ref)]
    pub path: Path<'db>,
}