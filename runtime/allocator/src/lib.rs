#[cfg(feature = "tcmalloc")]
#[global_allocator]
static GLOBAL: tcmalloc_crate::TCMalloc = tcmalloc_crate::TCMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc_crate::MiMalloc = mimalloc_crate::MiMalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(feature = "rpmalloc")]
#[global_allocator]
static GLOBAL: rpmalloc_crate::RpMalloc = rpmalloc_crate::RpMalloc;
