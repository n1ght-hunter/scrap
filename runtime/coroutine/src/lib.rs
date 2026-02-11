#![allow(unsafe_code)]
//! Stackful coroutine library for the Scrap runtime.
//!
//! Provides `CoroutineStack` — a `Send` coroutine that can migrate between
//! OS threads. Uses custom asm context switching with `extern "sysv64"` to
//! avoid saving XMM registers in the switch itself.
//!
//! Target: **x86_64** (Windows, Linux, macOS).

mod context;
pub mod pool;
mod stack;

pub use context::yield_current;
pub use pool::{acquire_stack, release_stack};
pub use stack::Stack;

use context::{swap_registers, SavedContext};
use std::cell::Cell;
use std::ptr::null_mut;

/// Default initial stack size for coroutines (8 KiB).
pub const DEFAULT_STACK_SIZE: usize = Stack::DEFAULT_SIZE;

/// Maximum stack size after growth (1 MiB).
pub const MAX_STACK_SIZE: usize = Stack::MAX_SIZE;

/// Result of resuming a coroutine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoroutineStatus {
    /// The coroutine yielded and can be resumed again.
    Yielded,
    /// The coroutine's closure has returned.
    Completed,
}

/// A stackful, `Send` coroutine.
///
/// The coroutine runs on its own allocated stack (VirtualAlloc on Windows,
/// mmap on Unix) and can be migrated between OS threads between resume/yield
/// cycles.
pub struct CoroutineStack {
    stack: Stack,
    caller_ctx: SavedContext,
    coro_ctx: SavedContext,
    /// Heap-allocated so its address is stable across moves of CoroutineStack.
    /// The coroutine_wrapper writes `Completed` here via a raw pointer (R13)
    /// that was set up at creation time.
    status: Box<CoroutineStatus>,
    /// Raw pointer to `Box<Box<dyn FnOnce()>>`. Null after first resume.
    closure: *mut u8,
}

// SAFETY: CoroutineStack is Send because:
// - All saved register state is plain integers
// - TEB fields describe the coroutine's own synthetic stack, not any thread
// - The closure is Send + 'static
// - On resume, the current thread's TEB is saved to caller_ctx and the
//   coroutine's TEB is loaded. On yield, reversed. Clean swap every time.
unsafe impl Send for CoroutineStack {}

thread_local! {
    static CALLER_CTX: Cell<*mut SavedContext> = const { Cell::new(null_mut()) };
    static CORO_CTX: Cell<*mut SavedContext> = const { Cell::new(null_mut()) };
}

/// Trampoline called on the coroutine's stack by `bootstrap_green_task`.
///
/// R12 = closure_ptr, R13 = status_ptr, R14 = this function's address.
/// bootstrap_green_task passes R12 in RDI and R13 in RSI (sysv64 ABI).
extern "sysv64" fn coroutine_wrapper(closure_ptr: usize, status_ptr: usize) {
    // Reconstruct the boxed closure and call it.
    let closure: Box<Box<dyn FnOnce()>> = unsafe { Box::from_raw(closure_ptr as *mut _) };
    (*closure)();

    // Mark completed.
    let status = status_ptr as *mut CoroutineStatus;
    unsafe { status.write(CoroutineStatus::Completed) };

    // Final yield back to the caller.
    yield_current();

    // Should never get here — resuming a completed coroutine is a bug.
    unreachable!("resumed a completed coroutine");
}

impl CoroutineStack {
    /// Create a new coroutine with a fresh stack (for tests / convenience).
    pub fn new(f: impl FnOnce() + Send + 'static) -> Self {
        Self::with_stack(Stack::new(Stack::DEFAULT_SIZE), f)
    }

    /// Create a new coroutine that will execute `f` on the provided stack.
    pub fn with_stack(stack: Stack, f: impl FnOnce() + Send + 'static) -> Self {
        // Double-box: outer Box<Box<dyn FnOnce()>> so we have a thin pointer.
        let closure: Box<Box<dyn FnOnce()>> = Box::new(Box::new(f));
        let closure_ptr = Box::into_raw(closure) as *mut u8;

        let mut coro = CoroutineStack {
            stack,
            caller_ctx: SavedContext::zeroed(),
            coro_ctx: SavedContext::zeroed(),
            status: Box::new(CoroutineStatus::Yielded),
            closure: closure_ptr,
        };

        // Set up the initial coroutine context so that the first
        // swap_registers lands in bootstrap_green_task.
        let stack_top = coro.stack.top() as u64;
        // Align to 16 bytes and leave space for the return address slot.
        // After `ret` in swap_registers pops the bootstrap address, RSP will
        // be 16-byte aligned (required by both Windows and sysv64 ABIs).
        let initial_rsp = (stack_top - 8) & !15;

        // Write bootstrap_green_task address as the "return address" at [RSP].
        unsafe {
            let rsp_ptr = initial_rsp as *mut u64;
            rsp_ptr.write(context::bootstrap_green_task as *const () as u64);
        }

        coro.coro_ctx.rsp = initial_rsp;
        // R12 = closure_ptr, R13 = &mut status, R14 = coroutine_wrapper fn
        coro.coro_ctx.r12 = closure_ptr as u64;
        coro.coro_ctx.r13 = &mut *coro.status as *mut CoroutineStatus as u64;
        coro.coro_ctx.r14 = coroutine_wrapper as *const () as u64;

        // TEB fields for the coroutine's stack (Windows only).
        #[cfg(windows)]
        {
            coro.coro_ctx.teb_stack_base = coro.stack.top() as u64;
            coro.coro_ctx.teb_stack_limit = coro.stack.committed_bottom() as u64;
            coro.coro_ctx.teb_dealloc = coro.stack.base() as u64;
        }

        coro
    }

    /// Current stack allocation size in bytes.
    pub fn stack_size(&self) -> usize {
        self.stack.size()
    }

    /// The lowest usable stack address (stack overflow limit).
    pub fn stack_limit(&self) -> u64 {
        self.stack.committed_bottom() as u64
    }

    /// Copy the coroutine's stack to `new_stack` (which must be larger),
    /// conservatively relocate pointers, and return the old stack.
    ///
    /// Called by the scheduler when a coroutine needs more stack space.
    /// The coroutine must be yielded (not running) when this is called.
    pub fn grow_stack(&mut self, new_stack: Stack) -> Stack {
        let old_rsp = self.coro_ctx.rsp;
        let old_top = self.stack.top() as u64;
        let new_top = new_stack.top() as u64;
        let used = (old_top - old_rsp) as usize;
        let new_rsp = new_top - used as u64;
        let delta = new_top as i64 - old_top as i64;

        // Copy used portion from old stack to new stack.
        unsafe {
            std::ptr::copy_nonoverlapping(
                old_rsp as *const u8,
                new_rsp as *mut u8,
                used,
            );
        }

        // Conservative pointer relocation: scan every 8-byte-aligned word
        // on the new stack. If it looks like it pointed into the old stack,
        // adjust it by delta.
        let mut addr = new_rsp;
        while addr + 8 <= new_top {
            let ptr = addr as *mut u64;
            let val = unsafe { ptr.read() };
            if val >= old_rsp && val < old_top {
                unsafe { ptr.write((val as i64 + delta) as u64) };
            }
            addr += 8;
        }

        // Adjust saved registers.
        self.coro_ctx.rsp = (self.coro_ctx.rsp as i64 + delta) as u64;
        // RBP and other callee-saved registers: adjust if they point into
        // the old stack range.
        for reg in [
            &mut self.coro_ctx.rbp,
            &mut self.coro_ctx.rbx,
            &mut self.coro_ctx.r12,
            &mut self.coro_ctx.r13,
            &mut self.coro_ctx.r14,
            &mut self.coro_ctx.r15,
        ] {
            if *reg >= old_rsp && *reg < old_top {
                *reg = (*reg as i64 + delta) as u64;
            }
        }

        // Update TEB fields for the new stack.
        #[cfg(windows)]
        {
            self.coro_ctx.teb_stack_base = new_stack.top() as u64;
            self.coro_ctx.teb_stack_limit = new_stack.committed_bottom() as u64;
            self.coro_ctx.teb_dealloc = new_stack.base() as u64;
        }

        std::mem::replace(&mut self.stack, new_stack)
    }

    /// Take ownership of the underlying stack for pool return.
    /// After this call, Drop will not deallocate the stack.
    pub fn take_stack(&mut self) -> Option<Stack> {
        if self.stack.base().is_null() {
            return None;
        }
        Some(std::mem::replace(&mut self.stack, Stack::null()))
    }

    /// Resume the coroutine. Returns its status after it yields or completes.
    ///
    /// # Panics
    /// Panics if called on a coroutine that has already completed.
    pub fn resume(&mut self) -> CoroutineStatus {
        assert!(
            *self.status != CoroutineStatus::Completed,
            "cannot resume a completed coroutine"
        );

        // Null out closure pointer after first resume (closure is now owned
        // by the coroutine stack via coroutine_wrapper).
        self.closure = null_mut();

        // Default to Yielded — coroutine_wrapper overwrites to Completed
        // if the closure returns.
        *self.status = CoroutineStatus::Yielded;

        // Set thread-locals so yield_current() knows where to switch.
        CALLER_CTX.with(|c| c.set(&mut self.caller_ctx as *mut SavedContext));
        CORO_CTX.with(|c| c.set(&mut self.coro_ctx as *mut SavedContext));

        unsafe { swap_registers(&mut self.caller_ctx, &self.coro_ctx) };

        // Execution returns here after yield or completion.
        CALLER_CTX.with(|c| c.set(null_mut()));
        CORO_CTX.with(|c| c.set(null_mut()));

        *self.status
    }
}

impl Drop for CoroutineStack {
    fn drop(&mut self) {
        // If the closure was never consumed (coroutine never resumed),
        // drop it to avoid a leak.
        if !self.closure.is_null() {
            let _: Box<Box<dyn FnOnce()>> = unsafe { Box::from_raw(self.closure as *mut _) };
            self.closure = null_mut();
        }
        // Stack is freed by Stack::drop.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    #[test]
    fn basic_run_to_completion() {
        let flag = Arc::new(AtomicU64::new(0));
        let f = flag.clone();
        let mut coro = CoroutineStack::new(move || {
            f.store(42, Ordering::SeqCst);
        });
        let status = coro.resume();
        assert_eq!(status, CoroutineStatus::Completed);
        assert_eq!(flag.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn yield_and_resume() {
        let counter = Arc::new(AtomicU64::new(0));
        let c = counter.clone();
        let mut coro = CoroutineStack::new(move || {
            c.store(1, Ordering::SeqCst);
            yield_current();
            c.store(2, Ordering::SeqCst);
            yield_current();
            c.store(3, Ordering::SeqCst);
        });

        assert_eq!(coro.resume(), CoroutineStatus::Yielded);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        assert_eq!(coro.resume(), CoroutineStatus::Yielded);
        assert_eq!(counter.load(Ordering::SeqCst), 2);

        assert_eq!(coro.resume(), CoroutineStatus::Completed);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn cross_thread_migration() {
        let counter = Arc::new(AtomicU64::new(0));
        let c = counter.clone();
        let mut coro = CoroutineStack::new(move || {
            c.fetch_add(1, Ordering::SeqCst);
            yield_current();
            c.fetch_add(1, Ordering::SeqCst);
        });

        // First resume on the main thread.
        assert_eq!(coro.resume(), CoroutineStatus::Yielded);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Send to another thread and resume there.
        let handle = std::thread::spawn(move || {
            assert_eq!(coro.resume(), CoroutineStatus::Completed);
            coro // return so it's dropped on this thread
        });
        handle.join().unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn drop_without_resume() {
        // Should not leak or crash.
        let _coro = CoroutineStack::new(|| {
            panic!("should never run");
        });
    }

    #[test]
    fn many_yields() {
        let counter = Arc::new(AtomicU64::new(0));
        let c = counter.clone();
        let mut coro = CoroutineStack::new(move || {
            for _ in 0..100 {
                c.fetch_add(1, Ordering::SeqCst);
                yield_current();
            }
        });

        for _ in 0..100 {
            assert_eq!(coro.resume(), CoroutineStatus::Yielded);
        }
        assert_eq!(coro.resume(), CoroutineStatus::Completed);
        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }
}
