//! M:N coroutine scheduler for `spawn` support.
//!
//! Uses scrap_coroutine for context switching and a pool of OS worker threads.
//! Coroutines can migrate between workers (work-stealing) since
//! `CoroutineStack` is `Send`.

use scrap_coroutine::{CoroutineStack, CoroutineStatus};
use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use crate::sync::{Condvar, Mutex};

/// Type-safe wrapper around a compiler-generated spawn trampoline.
///
/// The codegen emits one trampoline per spawn-site with the C signature
/// `extern "C" fn(args_ptr: *const u8)`.  At runtime we receive the raw
/// function address as a `usize` (passed through Cranelift) and recover
/// the typed pointer here.
#[derive(Clone, Copy)]
#[repr(transparent)]
struct Trampoline(extern "C" fn(*const u8));

impl Trampoline {
    /// Recover a typed trampoline from a raw code address.
    ///
    /// # Safety
    /// `addr` must be a valid pointer to a function with the signature
    /// `extern "C" fn(*const u8)`.
    unsafe fn from_raw(addr: usize) -> Self {
        Self(unsafe { std::mem::transmute(addr) })
    }

    fn call(self, args: *const u8) {
        (self.0)(args);
    }
}

// Stack limit for the currently running coroutine on this worker thread.
// Set to the stack's committed_bottom when a coroutine is resumed.
// Cleared to 0 when the coroutine yields or completes.
// 0 means "no limit" (main thread or no coroutine running).
thread_local! {
    static STACK_LIMIT: Cell<u64> = const { Cell::new(0) };
    static NEEDS_GROWTH: Cell<bool> = const { Cell::new(false) };
}

/// Headroom below the stack limit before we trigger growth / abort.
const RED_ZONE: u64 = 2048;

/// Abort with a stack overflow message. Does not return.
fn stack_overflow_abort() -> ! {
    let msg = b"stack overflow in coroutine (max stack size reached)\n";

    use std::io::Write;
    let _ = std::io::stderr().write_all(msg);
    std::process::exit(101);
}

/// A live coroutine.
/// `Send` because `CoroutineStack` is `Send`.
struct Task {
    coro: CoroutineStack,
    /// Stack limit address for overflow checks (committed_bottom of the stack).
    stack_limit: u64,
}

struct SchedulerState {
    queue: Mutex<VecDeque<Task>>,
    condvar: Condvar,
    shutdown: AtomicBool,
    workers: Mutex<Vec<JoinHandle<()>>>,
}

static STATE: OnceLock<SchedulerState> = OnceLock::new();

fn worker_loop() {
    let state = STATE.get().expect("scheduler not initialized");

    loop {
        // Block on global queue for the next task.
        let task = {
            let mut queue = mutex_lock!(state.queue);
            loop {
                // Don't pick up new tasks while GC is scanning
                if crate::gc::GC_SCAN_REQUESTED.load(Ordering::Acquire) {
                    if state.shutdown.load(Ordering::Acquire) {
                        return;
                    }
                    condvar_wait!(state.condvar, queue);
                    continue;
                }
                if let Some(task) = queue.pop_front() {
                    break task;
                }
                if state.shutdown.load(Ordering::Acquire) {
                    return;
                }
                condvar_wait!(state.condvar, queue);
            }
        };

        resume_task(task, state);
    }
}

/// Resume a single task once. If it yields, push it back to the global queue
/// so any worker can pick it up. If it completes, return its stack to the pool.
fn resume_task(mut task: Task, state: &SchedulerState) {
    // Track active coroutines for GC coordination.
    crate::gc::ACTIVE_COROS.fetch_add(1, Ordering::Release);

    // Set the stack limit so __scrap_yield can detect overflow.
    STACK_LIMIT.with(|c| c.set(task.stack_limit));

    match task.coro.resume() {
        CoroutineStatus::Yielded => {
            STACK_LIMIT.with(|c| c.set(0));

            // Check if the coroutine requested a stack growth.
            let needs_growth = NEEDS_GROWTH.with(|c| {
                let v = c.get();
                c.set(false);
                v
            });
            if needs_growth {
                let old_size = task.coro.stack_size();
                let new_size = old_size * 2;
                if new_size > scrap_coroutine::MAX_STACK_SIZE {
                    stack_overflow_abort();
                }
                let new_stack = scrap_coroutine::acquire_stack(new_size);
                let old_stack = task.coro.grow_stack(new_stack);
                task.stack_limit = task.coro.stack_limit();
                scrap_coroutine::release_stack(old_stack);
            }

            // Check if GC wants us to park this coroutine
            if crate::gc::GC_SCAN_REQUESTED.load(Ordering::Acquire) {
                let rbp = task.coro.saved_rbp();
                // Register coroutine RBP and decrement active count under the lock
                // to avoid race with GC thread checking ACTIVE_COROS.
                {
                    let mut ps = mutex_lock!(crate::gc::GC_PAUSE);
                    ps.parked_rbps.push(rbp);
                    crate::gc::ACTIVE_COROS.fetch_sub(1, Ordering::Release);
                    crate::gc::GC_PAUSE_CONDVAR.notify_all();

                    // Wait for GC to finish
                    while !ps.gc_done {
                        condvar_wait!(crate::gc::GC_PAUSE_CONDVAR, ps);
                    }
                }
                // GC is done, push task back to queue
                mutex_lock!(state.queue).push_back(task);
                state.condvar.notify_one();
            } else {
                crate::gc::ACTIVE_COROS.fetch_sub(1, Ordering::Release);
                // Push back to global queue — any worker can resume this coroutine.
                mutex_lock!(state.queue).push_back(task);
                state.condvar.notify_one();
            }
        }
        CoroutineStatus::Completed => {
            STACK_LIMIT.with(|c| c.set(0));
            crate::gc::ACTIVE_COROS.fetch_sub(1, Ordering::Release);
            if let Some(stack) = task.coro.take_stack() {
                scrap_coroutine::release_stack(stack);
            }
        }
    }
}

/// Return the saved RBPs of all coroutines currently sitting in the
/// scheduler queue. Called by the GC during STW to scan queued coroutine stacks.
pub(crate) fn get_queued_coro_rbps() -> Vec<u64> {
    let Some(state) = STATE.get() else {
        return Vec::new();
    };
    let queue = mutex_lock!(state.queue);
    queue.iter().map(|task| task.coro.saved_rbp()).collect()
}

/// Initialize the coroutine scheduler. Called from `_start` before `main`.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_sched_init() {
    let state = STATE.get_or_init(|| SchedulerState {
        queue: Mutex::new(VecDeque::new()),
        condvar: Condvar::new(),
        shutdown: AtomicBool::new(false),
        workers: Mutex::new(Vec::new()),
    });

    state.shutdown.store(false, Ordering::Release);

    let nworkers = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let mut handles = mutex_lock!(state.workers);
    for _ in 0..nworkers {
        handles.push(thread::spawn(worker_loop));
    }
}

/// Spawn a new coroutine.
///
/// - `trampoline`: function pointer (as integer) to a compiler-generated trampoline
///   with signature `extern "C" fn(*const u8)` that unpacks args and calls the target.
/// - `args_ptr`: pointer to packed arguments on the caller's stack.
/// - `nargs`: number of 8-byte argument slots.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_spawn(trampoline: usize, args_ptr: *const u8, nargs: u64) {
    // Copy args to heap — the caller's stack slot may be reclaimed.
    let args_copy: Vec<u8> = if nargs > 0 && !args_ptr.is_null() {
        let byte_count = (nargs as usize) * 8;
        let mut buf = vec![0u8; byte_count];
        unsafe {
            std::ptr::copy_nonoverlapping(args_ptr, buf.as_mut_ptr(), byte_count);
        }
        buf
    } else {
        Vec::new()
    };

    let tramp = unsafe { Trampoline::from_raw(trampoline) };

    let stack = scrap_coroutine::acquire_stack(scrap_coroutine::DEFAULT_STACK_SIZE);
    let stack_limit = stack.committed_bottom() as u64;

    let coro = CoroutineStack::with_stack(stack, move || {
        tramp.call(args_copy.as_ptr());
    });

    let task = Task { coro, stack_limit };

    let state = STATE.get().expect("scheduler not initialized");
    mutex_lock!(state.queue).push_back(task);
    state.condvar.notify_one();
}

/// Combined yield point + stack overflow check.
///
/// Called at every function prologue. On the main thread (STACK_LIMIT == 0),
/// this is a fast no-op (unless GC is requested). Inside a coroutine, checks
/// the stack and yields.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_yield() {
    // GC safepoint check — all threads (main + coroutine workers).
    // Must come BEFORE the STACK_LIMIT early-return so the main thread
    // also cooperates with STW.
    if crate::gc::GC_SCAN_REQUESTED.load(Ordering::Acquire) {
        crate::gc::gc_safepoint();
    }

    let limit = STACK_LIMIT.with(|c| c.get());
    if limit == 0 {
        return; // Main thread — no coroutine, no check.
    }

    // Stack overflow check: read RSP and compare against the limit.
    let sp: u64;
    unsafe { std::arch::asm!("mov {}, rsp", out(reg) sp) };
    if sp <= limit + RED_ZONE {
        // Signal the scheduler to grow the stack, then yield so it can.
        NEEDS_GROWTH.with(|c| c.set(true));
    }

    // Cooperative yield.
    scrap_coroutine::yield_current();
}

/// Drain remaining coroutines and shut down worker threads.
/// Called from `_start` after `main`.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_sched_shutdown() {
    let Some(state) = STATE.get() else { return };

    // Signal workers to exit once the queue is drained.
    state.shutdown.store(true, Ordering::Release);
    state.condvar.notify_all();

    // Join all worker threads.
    let mut handles = mutex_lock!(state.workers);
    for handle in handles.drain(..) {
        let _ = handle.join();
    }
}
