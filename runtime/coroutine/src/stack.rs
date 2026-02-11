//! Stack allocation for coroutines.
//!
//! Uses the global allocator (`std::alloc`) instead of direct syscalls,
//! so stacks benefit from whatever allocator is configured via `scrap_allocator`.

use std::alloc::Layout;

/// A heap-allocated stack for a coroutine.
///
/// Layout (addresses increase upward):
///
/// ```text
///   base (low addr)  = allocation start
///   +-- usable stack  -- read/write
///   +-- top (high addr) -- stack grows down from here
/// ```
///
/// No guard pages — overflow is detected by software checks in `__scrap_yield`.
pub struct Stack {
    /// Lowest address of the allocated region.
    base: *mut u8,
    /// Allocation layout (size + alignment).
    layout: Layout,
}

// SAFETY: The raw pointer `base` is a unique heap allocation owned by this Stack.
unsafe impl Send for Stack {}

impl Stack {
    /// Default initial stack size: 8 KiB.
    pub const DEFAULT_SIZE: usize = 8 * 1024;

    /// Maximum stack size after growth: 1 MiB.
    pub const MAX_SIZE: usize = 1024 * 1024;

    /// Page-aligned layout for stack allocations.
    const ALIGN: usize = 4096;

    /// A null/empty stack. Drop is a no-op.
    pub fn null() -> Self {
        Stack {
            base: std::ptr::null_mut(),
            layout: Layout::new::<()>(),
        }
    }

    /// Allocate a new stack of the given size.
    pub fn new(size: usize) -> Self {
        let size = (size + Self::ALIGN - 1) & !(Self::ALIGN - 1);
        let layout = Layout::from_size_align(size, Self::ALIGN).expect("invalid stack layout");
        let base = unsafe { std::alloc::alloc(layout) };
        assert!(!base.is_null(), "stack allocation failed");
        Stack { base, layout }
    }

    /// Total size in bytes.
    pub fn size(&self) -> usize {
        self.layout.size()
    }

    /// Top of the stack (highest address). Stack grows downward from here.
    pub fn top(&self) -> *mut u8 {
        unsafe { self.base.add(self.layout.size()) }
    }

    /// Base of the allocation (lowest address).
    pub fn base(&self) -> *mut u8 {
        self.base
    }

    /// Bottom of the usable region. With no guard pages, this equals `base`.
    pub fn committed_bottom(&self) -> *mut u8 {
        self.base
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if !self.base.is_null() && self.layout.size() > 0 {
            unsafe { std::alloc::dealloc(self.base, self.layout) };
        }
    }
}
