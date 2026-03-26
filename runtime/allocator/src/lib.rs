cfg_if::cfg_if! {
    if #[cfg(feature = "rtmalloc")] {
        #[global_allocator]
        static GLOBAL: rtmalloc::RtMalloc = rtmalloc::RtMalloc;
    } else if #[cfg(feature = "mimalloc")] {
        #[global_allocator]
        static GLOBAL: mimalloc_crate::MiMalloc = mimalloc_crate::MiMalloc;
    } else if #[cfg(feature = "tcmalloc")] {
        #[global_allocator]
        static GLOBAL: tcmalloc_crate::TCMalloc = tcmalloc_crate::TCMalloc;
    } else if #[cfg(feature = "jemalloc")] {
        #[global_allocator]
        static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
    } else if #[cfg(feature = "rpmalloc")] {
        #[global_allocator]
        static GLOBAL: rpmalloc_crate::RpMalloc = rpmalloc_crate::RpMalloc;
    }
}
