//! Thread-safe pool of reusable coroutine stacks.
//!
//! Avoids per-spawn syscalls (VirtualAlloc/mmap) by caching freed stacks
//! and handing them out to new coroutines.

use crate::stack::Stack;
use std::{
    cell::{LazyCell, RefCell},
    sync::{Mutex, OnceLock},
};

/// A thread-safe pool of reusable coroutine stacks.
pub struct StackPool {
    free_list: Mutex<Vec<Stack>>,
    max_cached: usize,
}

impl StackPool {
    const fn new(max_cached: usize) -> Self {
        StackPool {
            free_list: Mutex::new(Vec::new()),
            max_cached,
        }
    }

    /// Get a stack from the pool, or allocate a new one.
    pub fn acquire(&self, size: usize) -> Stack {
        let mut free = self.free_list.lock().unwrap();
        let mut best_bigger: Option<(usize, usize)> = None;
        for (i, s) in free.iter().enumerate() {
            let s_size = s.size();
            if s_size == size {
                return free.swap_remove(i);
            }
            if s_size > size && best_bigger.map_or(true, |(_, best)| s_size < best) {
                best_bigger = Some((i, s_size));
            }
        }
        if let Some((i, _)) = best_bigger {
            return free.swap_remove(i);
        }
        drop(free);
        Stack::new(size)
    }

    /// Return a stack to the pool for reuse.
    /// If the pool is full, the stack is dropped (deallocated).
    pub fn release(&self, stack: Stack) {
        let mut free = self.free_list.lock().unwrap();
        if free.len() < self.max_cached {
            free.push(stack);
        }
        // else: `stack` drops here → VirtualFree / munmap
    }
}

struct LocalCache {
    stack: [Option<Stack>; 8],
    allocated: usize,
}

impl LocalCache {
    fn new() -> Self {
        LocalCache {
            stack: [None, None, None, None, None, None, None, None],
            allocated: 0,
        }
    }

    fn acquire(&mut self, size: usize) -> Option<Stack> {
        let mut best_bigger: Option<(usize, usize)> = None; // (index, size)
        for (i, slot) in self.stack.iter().enumerate() {
            if let Some(s) = slot {
                let s_size = s.size();
                if s_size == size {
                    // Exact match — take it immediately.
                    let stack = self.stack[i].take().unwrap();
                    self.allocated += 1;
                    return Some(stack);
                }
                if s_size > size {
                    if best_bigger.map_or(true, |(_, best)| s_size < best) {
                        best_bigger = Some((i, s_size));
                    }
                }
            }
        }
        // No exact match — use the smallest bigger stack if available.
        if let Some((i, _)) = best_bigger {
            let stack = self.stack[i].take().unwrap();
            self.allocated += 1;
            return Some(stack);
        }
        None
    }

    /// Try to cache locally. Returns `None` if cached, `Some(stack)` if full.
    fn release(&mut self, stack: Stack) -> Option<Stack> {
        for slot in self.stack.iter_mut() {
            if slot.is_none() {
                *slot = Some(stack);
                self.allocated -= 1;
                return None;
            }
        }
        Some(stack)
    }
}

static POOL: OnceLock<StackPool> = OnceLock::new();

thread_local! {
    /// Per-thread stack cache. Avoids locking for the common case of reusing
    /// the same stack across multiple coroutines on the same thread.
    static THREAD_CACHE: LazyCell<RefCell<LocalCache>> = LazyCell::new(|| RefCell::new(LocalCache::new()));
}

pub fn acquire_stack(size: usize) -> Stack {
    let res = THREAD_CACHE.with(|cache| cache.borrow_mut().acquire(size));
    if let Some(stack) = res {
        return stack;
    }
    POOL.get_or_init(|| StackPool::new(64)).acquire(size)
}


pub fn release_stack(stack: Stack) {
    let released = THREAD_CACHE.with(|cache| cache.borrow_mut().release(stack));
    if let Some(stack) = released {
        POOL.get_or_init(|| StackPool::new(64)).release(stack);
    }
}