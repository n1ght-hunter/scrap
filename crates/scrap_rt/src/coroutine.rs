//! M:N coroutine scheduler for `spawn` support.
//!
//! Uses corosensei for context switching and a pool of OS worker threads.
//! Spawned coroutines run concurrently with `main` on real OS threads.

use corosensei::{Coroutine, CoroutineResult, Yielder};
use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};

use crate::sync::{Condvar, Mutex};

// ---------------------------------------------------------------------------
// Typed wrappers for raw pointers
// ---------------------------------------------------------------------------

/// Opaque handle to a `corosensei::Yielder<(), ()>`.
///
/// Stored in thread-local storage so `__scrap_yield` can suspend the current
/// coroutine without holding a borrow on the `Yielder`.
#[derive(Clone, Copy)]
#[repr(transparent)]
struct YielderPtr(*const Yielder<(), ()>);

impl YielderPtr {
    const NULL: Self = Self(std::ptr::null());

    fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Suspend the coroutine through the stored yielder.
    ///
    /// # Safety
    /// The pointer must be valid and point to a live `Yielder` whose
    /// coroutine is currently executing.
    unsafe fn suspend(self) {
        unsafe { (*self.0).suspend(()) }
    }
}

/// Opaque handle to the GC shadow-stack top pointer.
///
/// Saved/restored on every coroutine context switch so the collector can
/// walk the shadow stacks of parked coroutines.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct ShadowStackTop(pub(crate) *mut u8);

impl ShadowStackTop {
    pub(crate) const NULL: Self = Self(std::ptr::null_mut());
}

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

// ---------------------------------------------------------------------------
// Scheduler internals
// ---------------------------------------------------------------------------

/// What gets sent through the global queue: just the spawn arguments.
/// The `Coroutine` is created on the worker thread that picks this up,
/// so `Coroutine` (which is `!Send`) never crosses a thread boundary.
struct SpawnRequest {
    tramp: Trampoline,
    args: Vec<u8>,
}

/// A live coroutine, only ever lives on one worker's local queue.
/// Never sent between threads — no `Send` impl needed.
struct Task {
    coro: Coroutine<(), (), ()>,
    shadow_top: ShadowStackTop,
}

struct SchedulerState {
    queue: Mutex<VecDeque<SpawnRequest>>,
    condvar: Condvar,
    shutdown: AtomicBool,
    workers: Mutex<Vec<JoinHandle<()>>>,
}

static STATE: OnceLock<SchedulerState> = OnceLock::new();

thread_local! {
    /// Pointer to the current coroutine's Yielder (null when on main/idle stack).
    static CURRENT_YIELDER: Cell<YielderPtr> = const { Cell::new(YielderPtr::NULL) };
}

// ---------------------------------------------------------------------------
// Worker thread logic
// ---------------------------------------------------------------------------

fn worker_loop() {
    let state = STATE.get().expect("scheduler not initialized");
    // Per-worker local run queue. Coroutines that yield stay pinned to this
    // thread (no cross-thread migration — corosensei TEB issue on Windows).
    // Yield round-robins among all coroutines assigned to this worker.
    let mut local: VecDeque<Task> = VecDeque::new();

    loop {
        // Pick next task: prefer local queue, then create from global spawn request.
        let task = if let Some(task) = local.pop_front() {
            task
        } else {
            // Block on global queue for a new SpawnRequest.
            let req = {
                let mut queue = mutex_lock!(state.queue);
                loop {
                    if let Some(req) = queue.pop_front() {
                        break req;
                    }
                    if state.shutdown.load(Ordering::Acquire) {
                        return;
                    }
                    condvar_wait!(state.condvar, queue);
                }
            };
            // Create the Coroutine here on the worker thread — never crosses threads.
            make_task(req)
        };

        resume_task(task, &mut local);
    }
}

/// Turn a `SpawnRequest` into a live `Task` (coroutine).
/// Must be called on the worker thread that will own this coroutine.
fn make_task(req: SpawnRequest) -> Task {
    let coro = Coroutine::new(move |yielder, ()| {
        CURRENT_YIELDER.with(|c| c.set(YielderPtr(yielder as *const Yielder<(), ()>)));
        req.tramp.call(req.args.as_ptr());
    });
    Task {
        coro,
        shadow_top: ShadowStackTop::NULL,
    }
}

/// Resume a single task once. If it yields, push it to the back of `local`.
/// If it returns, drop it.
fn resume_task(mut task: Task, local: &mut VecDeque<Task>) {
    crate::gc::restore_shadow_stack_top(task.shadow_top);

    match task.coro.resume(()) {
        CoroutineResult::Yield(()) => {
            task.shadow_top = crate::gc::save_shadow_stack_top();
            local.push_back(task);
        }
        CoroutineResult::Return(()) => {
            crate::gc::restore_shadow_stack_top(ShadowStackTop::NULL);
            CURRENT_YIELDER.with(|c| c.set(YielderPtr::NULL));
        }
    }
}

// ---------------------------------------------------------------------------
// Public ABI (called from compiler-generated code)
// ---------------------------------------------------------------------------

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

    let state = STATE.get().expect("scheduler not initialized");
    mutex_lock!(state.queue).push_back(SpawnRequest { tramp, args: args_copy });
    state.condvar.notify_one();
}

/// Yield the current coroutine. No-op if called from the main stack.
///
/// After suspend, the coroutine may resume on a different OS thread.
/// We must not hold a TLS borrow across the suspend point.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_yield() {
    // Copy the yielder pointer out of TLS (drops the TLS borrow).
    let ptr = CURRENT_YIELDER.with(|cell| cell.get());
    if ptr.is_null() {
        return; // Not in a coroutine — no-op.
    }

    // Suspend. When this returns, we may be on a different OS thread.
    unsafe { ptr.suspend() }

    // Re-set CURRENT_YIELDER on whatever thread we resumed on.
    CURRENT_YIELDER.with(|cell| cell.set(ptr));
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
