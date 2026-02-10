//! Concurrent mark-and-sweep garbage collector with per-thread shadow stacks.
//!
//! Design:
//! - Each thread has its own `ThreadState` with a shadow stack (lock-free push/pop)
//! - Global heap state protected by a `Mutex`
//! - Tri-color marking with concurrent write barrier (Dijkstra insertion barrier)
//! - GC phase flag: Idle → Mark → Sweep → Idle
//! - Write barrier fast path: one atomic load + one branch (near-zero cost when idle)

// When gc-debug is off, counters used only in gc_log! appear unused.
#![cfg_attr(not(feature = "gc-debug"), allow(unused_variables, unused_assignments))]

use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicU8, Ordering};

#[cfg(not(feature = "parking-lot"))]
use std::sync::Mutex;
#[cfg(feature = "parking-lot")]
use parking_lot::Mutex;

/// Lock a mutex, abstracting over std (`lock().unwrap()`) vs parking_lot (`lock()`).
macro_rules! mutex_lock {
    ($mutex:expr) => {{
        #[cfg(not(feature = "parking-lot"))]
        { $mutex.lock().unwrap() }
        #[cfg(feature = "parking-lot")]
        { $mutex.lock() }
    }};
}

/// Try to lock a mutex, abstracting over std vs parking_lot.
macro_rules! mutex_try_lock {
    ($mutex:expr) => {{
        #[cfg(not(feature = "parking-lot"))]
        { $mutex.try_lock().ok() }
        #[cfg(feature = "parking-lot")]
        { $mutex.try_lock() }
    }};
}

// ---------------------------------------------------------------------------
// Debug logging — only compiled when the `gc-debug` feature is enabled.
// ---------------------------------------------------------------------------

macro_rules! gc_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "gc-debug")]
        eprintln!("[GC] {}", format_args!($($arg)*));
    };
}

// ---------------------------------------------------------------------------
// GcShape — type descriptor emitted by the compiler as a data section.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct GcShape {
    pub size: u64,
    pub align: u64,
    pub num_pointers: u64,
    // Flexible array of pointer offsets follows in memory.
    // pointer_offsets: [u64; num_pointers]
}

impl GcShape {
    unsafe fn pointer_offsets(&self) -> &[u64] {
        unsafe {
            let base = (self as *const GcShape).add(1) as *const u64;
            std::slice::from_raw_parts(base, self.num_pointers as usize)
        }
    }
}

// ---------------------------------------------------------------------------
// ObjHeader — prepended to every GC-allocated object.
// mark field is AtomicU8 for concurrent access during marking.
// ---------------------------------------------------------------------------

const MARK_WHITE: u8 = 0;
const MARK_GRAY: u8 = 1;
const MARK_BLACK: u8 = 2;

#[repr(C)]
struct ObjHeader {
    mark: AtomicU8,
    _pad: [u8; 7],
    size: u64,
    shape: *const GcShape,
    next: *mut ObjHeader,
}

impl ObjHeader {
    fn data_ptr(&mut self) -> *mut u8 {
        unsafe { (self as *mut ObjHeader).add(1) as *mut u8 }
    }
}

// ---------------------------------------------------------------------------
// Shadow stack frame — same structure, per-thread linked list.
// ---------------------------------------------------------------------------

#[repr(C)]
struct ShadowFrame {
    prev: *mut ShadowFrame,
    slots: *mut *mut u8,
    count: u64,
}

// ---------------------------------------------------------------------------
// Per-thread state
// ---------------------------------------------------------------------------

struct ThreadState {
    shadow_stack_top: AtomicPtr<ShadowFrame>,
}

// SAFETY: ThreadState is heap-allocated per thread, shadow_stack_top is atomic.
unsafe impl Send for ThreadState {}
unsafe impl Sync for ThreadState {}

thread_local! {
    static THREAD_STATE: std::cell::Cell<*mut ThreadState> = const { std::cell::Cell::new(null_mut()) };
}

fn ensure_thread_registered() -> *mut ThreadState {
    THREAD_STATE.with(|cell| {
        let ptr = cell.get();
        if !ptr.is_null() {
            return ptr;
        }
        let state = Box::into_raw(Box::new(ThreadState {
            shadow_stack_top: AtomicPtr::new(null_mut()),
        }));
        cell.set(state);
        let mut registry = mutex_lock!(THREAD_REGISTRY);
        registry.push(SendPtr(state));
        gc_log!(
            "thread registered: state={:?}, total_threads={}",
            state,
            registry.len()
        );
        state
    })
}

// ---------------------------------------------------------------------------
// GC phase flag — checked by write barrier fast path.
// ---------------------------------------------------------------------------

const PHASE_IDLE: u8 = 0;
const PHASE_MARK: u8 = 1;
const PHASE_SWEEP: u8 = 2;

static GC_PHASE: AtomicU8 = AtomicU8::new(PHASE_IDLE);

// ---------------------------------------------------------------------------
// Global heap state (protected by mutex).
// ---------------------------------------------------------------------------

struct HeapState {
    all_objects: *mut ObjHeader,
    bytes_allocated: usize,
    threshold: usize,
}

// SAFETY: ObjHeader pointers are only accessed under the HEAP mutex or during
// single-threaded mark/sweep with proper atomic coordination.
unsafe impl Send for HeapState {}

const INITIAL_HEAP_THRESHOLD: usize = 1024 * 1024; // 1MB

static HEAP: Mutex<HeapState> = Mutex::new(HeapState {
    all_objects: null_mut(),
    bytes_allocated: 0,
    threshold: INITIAL_HEAP_THRESHOLD,
});

// ---------------------------------------------------------------------------
// Thread registry — tracks all thread states for root scanning.
// ---------------------------------------------------------------------------

static THREAD_REGISTRY: Mutex<Vec<SendPtr<ThreadState>>> = Mutex::new(Vec::new());

// SAFETY: ShadowFrame pointers are only accessed by their owning thread or during GC.
unsafe impl Send for ShadowFrame {}

// Wrapper types to make raw pointers Send for use in Mutex<Vec<...>>.
struct SendPtr<T>(*mut T);
impl<T> Clone for SendPtr<T> {
    fn clone(&self) -> Self {
        SendPtr(self.0)
    }
}
impl<T> Copy for SendPtr<T> {}
unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

// ---------------------------------------------------------------------------
// Barrier worklist — objects shaded gray by write barriers during marking.
// ---------------------------------------------------------------------------

static BARRIER_WORKLIST: Mutex<Vec<SendPtr<ObjHeader>>> = Mutex::new(Vec::new());

// ---------------------------------------------------------------------------
// Exported C ABI functions
// ---------------------------------------------------------------------------

/// Initialize the GC. Called from `_start` before `main`.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_init() {
    gc_log!("init: resetting global state");
    // Reset global state
    {
        let mut heap = mutex_lock!(HEAP);
        heap.all_objects = null_mut();
        heap.bytes_allocated = 0;
        heap.threshold = 1024 * 1024;
    }
    GC_PHASE.store(PHASE_IDLE, Ordering::Relaxed);

    // Register the main thread
    ensure_thread_registered();
    gc_log!("init: complete, threshold={} bytes", INITIAL_HEAP_THRESHOLD);
}

/// Allocate a GC-managed object. Returns pointer to user data.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_alloc(shape: *const GcShape) -> *mut u8 {
    unsafe {
        let size = (*shape).size as usize;
        let total = std::mem::size_of::<ObjHeader>() + size;

        gc_log!("alloc: size={}, total={} (header+data)", size, total);

        let mut heap = mutex_lock!(HEAP);

        // Check if we should collect
        if heap.bytes_allocated + total > heap.threshold {
            gc_log!(
                "alloc: threshold exceeded ({} + {} > {}), triggering collection",
                heap.bytes_allocated,
                total,
                heap.threshold
            );
            collect(&mut heap);
            if heap.bytes_allocated + total > heap.threshold {
                #[cfg(feature = "gc-debug")]
                let old = heap.threshold;
                heap.threshold = (heap.bytes_allocated + total) * 2;
                gc_log!("alloc: grew threshold {} -> {}", old, heap.threshold);
            }
        }

        // Allocate header + user data
        let layout = std::alloc::Layout::from_size_align(total, 8).unwrap();
        let ptr = std::alloc::alloc_zeroed(layout) as *mut ObjHeader;
        if ptr.is_null() {
            gc_log!("alloc: first alloc failed, retrying after collection");
            collect(&mut heap);
            let ptr = std::alloc::alloc_zeroed(layout) as *mut ObjHeader;
            if ptr.is_null() {
                gc_log!("alloc: OOM after collection, exiting");
                std::process::exit(101);
            }
            init_obj_header(ptr, size, shape, &mut heap);
            let data = (*ptr).data_ptr();
            gc_log!(
                "alloc: ok (retry) header={:?}, data={:?}, heap_bytes={}",
                ptr,
                data,
                heap.bytes_allocated
            );
            return data;
        }

        init_obj_header(ptr, size, shape, &mut heap);
        let data = (*ptr).data_ptr();
        gc_log!(
            "alloc: ok header={:?}, data={:?}, heap_bytes={}",
            ptr,
            data,
            heap.bytes_allocated
        );
        data
    }
}

unsafe fn init_obj_header(
    ptr: *mut ObjHeader,
    size: usize,
    shape: *const GcShape,
    heap: &mut HeapState,
) {
    unsafe {
        // Initialize mark as white. We can't use simple assignment for AtomicU8 in
        // a zeroed allocation, so use store.
        (*ptr).mark.store(MARK_WHITE, Ordering::Relaxed);
        (*ptr).size = size as u64;
        (*ptr).shape = shape;
        (*ptr).next = heap.all_objects;
        heap.all_objects = ptr;
        heap.bytes_allocated += std::mem::size_of::<ObjHeader>() + size;
    }
}

/// Force a garbage collection cycle.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_collect() {
    gc_log!("collect: forced collection requested");
    let mut heap = mutex_lock!(HEAP);
    collect(&mut heap);
}

/// Push a shadow stack frame. Lock-free — writes only to this thread's state.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_push_frame(slots: *mut *mut u8, count: u64) {
    let state = ensure_thread_registered();
    unsafe {
        let prev = (*state).shadow_stack_top.load(Ordering::Relaxed);
        let frame = Box::into_raw(Box::new(ShadowFrame { prev, slots, count }));
        (*state).shadow_stack_top.store(frame, Ordering::Release);
        gc_log!(
            "push_frame: slots={:?}, count={}, frame={:?}",
            slots,
            count,
            frame
        );
    }
}

/// Pop the top shadow stack frame. Lock-free.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_pop_frame() {
    THREAD_STATE.with(|cell| {
        let state = cell.get();
        if !state.is_null() {
            unsafe {
                let frame = (*state).shadow_stack_top.load(Ordering::Relaxed);
                if !frame.is_null() {
                    gc_log!("pop_frame: frame={:?}, count={}", frame, (*frame).count);
                    let prev = (*frame).prev;
                    (*state).shadow_stack_top.store(prev, Ordering::Release);
                    drop(Box::from_raw(frame));
                }
            }
        }
    });
}

/// Write barrier — called from generated code when storing a `*T` to a heap location.
///
/// Fast path (GC not marking): one relaxed atomic load + one branch.
/// Slow path: shade the target gray via CAS, add to barrier worklist.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_write_barrier(new_val: *mut u8) {
    // Fast path — Acquire pairs with the Release store in collect() that sets PHASE_MARK,
    // ensuring the mutator sees the phase transition and doesn't skip the barrier.
    if GC_PHASE.load(Ordering::Acquire) != PHASE_MARK {
        return;
    }

    // Slow path
    if new_val.is_null() {
        gc_log!("write_barrier: slow path, new_val is null — skip");
        return;
    }
    unsafe {
        if let Some(header) = data_ptr_to_header(new_val) {
            // CAS white → gray (idempotent if already gray or black)
            if (*header)
                .mark
                .compare_exchange(MARK_WHITE, MARK_GRAY, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                gc_log!("write_barrier: shaded {:?} white->gray", header);
                mutex_lock!(BARRIER_WORKLIST).push(SendPtr(header));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Collection: mark + sweep
// ---------------------------------------------------------------------------

/// Run a full GC cycle. Caller must hold the HEAP mutex.
fn collect(heap: &mut HeapState) {
    gc_log!(
        "=== collection start: bytes_allocated={}, threshold={} ===",
        heap.bytes_allocated,
        heap.threshold
    );

    GC_PHASE.store(PHASE_MARK, Ordering::Release);
    gc_log!("phase -> MARK");

    unsafe {
        mark_phase();
    }

    GC_PHASE.store(PHASE_SWEEP, Ordering::Release);
    gc_log!("phase -> SWEEP");

    let bytes_before = heap.bytes_allocated;
    unsafe {
        sweep_phase(heap);
    }

    GC_PHASE.store(PHASE_IDLE, Ordering::Release);
    gc_log!(
        "=== collection end: reclaimed {} bytes, {} bytes remain ===",
        bytes_before.saturating_sub(heap.bytes_allocated),
        heap.bytes_allocated
    );
}

// ---------------------------------------------------------------------------
// Mark phase — scans all thread shadow stacks, traces heap.
// ---------------------------------------------------------------------------

unsafe fn mark_phase() {
    unsafe {
        let mut worklist: Vec<*mut ObjHeader> = Vec::new();
        #[cfg(feature = "gc-debug")]
        let mut roots_found: usize = 0;
        #[cfg(feature = "gc-debug")]
        let mut ti: usize = 0;

        // Snapshot thread registry (brief lock)
        let threads: Vec<SendPtr<ThreadState>> = mutex_lock!(THREAD_REGISTRY).clone();
        gc_log!("mark: scanning {} threads for roots", threads.len());

        // Scan all thread shadow stacks for roots
        for &SendPtr(thread_state) in &threads {
            #[cfg(feature = "gc-debug")]
            let mut frame_count: usize = 0;
            let mut frame = (*thread_state).shadow_stack_top.load(Ordering::Acquire);
            while !frame.is_null() {
                let slots = (*frame).slots;
                let count = (*frame).count as usize;
                for i in 0..count {
                    let slot_val = *slots.add(i);
                    if !slot_val.is_null() {
                        if let Some(header) = data_ptr_to_header(slot_val) {
                            if (*header).mark.load(Ordering::Relaxed) == MARK_WHITE {
                                (*header).mark.store(MARK_GRAY, Ordering::Relaxed);
                                worklist.push(header);
                                #[cfg(feature = "gc-debug")]
                                {
                                    roots_found += 1;
                                }
                            }
                        }
                    }
                }
                #[cfg(feature = "gc-debug")]
                {
                    frame_count += 1;
                }
                frame = (*frame).prev;
            }
            gc_log!("mark: thread[{}] scanned {} frames", ti, frame_count);
            #[cfg(feature = "gc-debug")]
            {
                ti += 1;
            }
        }
        gc_log!("mark: found {} roots", roots_found);

        // Process gray objects (trace from roots)
        #[cfg(feature = "gc-debug")]
        let mut traced: usize = 0;
        #[cfg(feature = "gc-debug")]
        let mut barrier_drained: usize = 0;
        while let Some(obj) = worklist.pop() {
            (*obj).mark.store(MARK_BLACK, Ordering::Relaxed);
            #[cfg(feature = "gc-debug")]
            {
                traced += 1;
            }

            let shape = (*obj).shape;
            if !shape.is_null() && (*shape).num_pointers > 0 {
                let data = (*obj).data_ptr();
                for &offset in (*shape).pointer_offsets() {
                    let field_ptr = *(data.add(offset as usize) as *const *mut u8);
                    if !field_ptr.is_null() {
                        if let Some(child_header) = data_ptr_to_header(field_ptr) {
                            if (*child_header).mark.load(Ordering::Relaxed) == MARK_WHITE {
                                (*child_header).mark.store(MARK_GRAY, Ordering::Relaxed);
                                worklist.push(child_header);
                            }
                        }
                    }
                }
            }

            // Drain barrier worklist (objects shaded gray by concurrent mutators)
            if let Some(mut barrier) = mutex_try_lock!(BARRIER_WORKLIST) {
                #[cfg(feature = "gc-debug")]
                {
                    let n = barrier.len();
                    if n > 0 {
                        gc_log!("mark: draining {} barrier entries", n);
                        barrier_drained += n;
                    }
                }
                for SendPtr(obj) in barrier.drain(..) {
                    worklist.push(obj);
                }
            }
        }
        gc_log!(
            "mark: traced {} objects, {} barrier entries drained",
            traced,
            barrier_drained
        );

        // Final drain — ensure no barrier entries were missed
        let mut barrier = mutex_lock!(BARRIER_WORKLIST);
        let mut remaining: Vec<*mut ObjHeader> = barrier.drain(..).map(|SendPtr(p)| p).collect();
        drop(barrier);

        #[cfg(feature = "gc-debug")]
        if !remaining.is_empty() {
            gc_log!("mark: final drain has {} entries", remaining.len());
        }

        // Process any final barrier entries (and their transitive children)
        #[cfg(feature = "gc-debug")]
        let mut final_traced: usize = 0;
        while let Some(obj) = remaining.pop() {
            if (*obj).mark.load(Ordering::Relaxed) != MARK_BLACK {
                (*obj).mark.store(MARK_BLACK, Ordering::Relaxed);
                #[cfg(feature = "gc-debug")]
                {
                    final_traced += 1;
                }

                let shape = (*obj).shape;
                if !shape.is_null() && (*shape).num_pointers > 0 {
                    let data = (*obj).data_ptr();
                    for &offset in (*shape).pointer_offsets() {
                        let field_ptr = *(data.add(offset as usize) as *const *mut u8);
                        if !field_ptr.is_null() {
                            if let Some(child_header) = data_ptr_to_header(field_ptr) {
                                if (*child_header).mark.load(Ordering::Relaxed) == MARK_WHITE {
                                    (*child_header).mark.store(MARK_GRAY, Ordering::Relaxed);
                                    remaining.push(child_header);
                                }
                            }
                        }
                    }
                }
            }
        }
        #[cfg(feature = "gc-debug")]
        if final_traced > 0 {
            gc_log!(
                "mark: final drain traced {} additional objects",
                final_traced
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Sweep phase — free white objects, reset marks on survivors.
// ---------------------------------------------------------------------------

unsafe fn sweep_phase(heap: &mut HeapState) {
    unsafe {
        let mut prev: *mut *mut ObjHeader = &raw mut heap.all_objects;
        let mut current = heap.all_objects;
        #[cfg(feature = "gc-debug")]
        let mut freed: usize = 0;
        #[cfg(feature = "gc-debug")]
        let mut freed_bytes: usize = 0;
        #[cfg(feature = "gc-debug")]
        let mut survived: usize = 0;

        while !current.is_null() {
            let next = (*current).next;

            if (*current).mark.load(Ordering::Relaxed) == MARK_WHITE {
                // Unreachable — free it
                *prev = next;
                let total = std::mem::size_of::<ObjHeader>() + (*current).size as usize;
                heap.bytes_allocated -= total;
                #[cfg(feature = "gc-debug")]
                {
                    freed += 1;
                    freed_bytes += total;
                }
                let layout = std::alloc::Layout::from_size_align(total, 8).unwrap();
                std::alloc::dealloc(current as *mut u8, layout);
            } else {
                // Reachable — reset mark for next cycle
                (*current).mark.store(MARK_WHITE, Ordering::Relaxed);
                #[cfg(feature = "gc-debug")]
                {
                    survived += 1;
                }
                prev = &raw mut (*current).next;
            }

            current = next;
        }

        gc_log!(
            "sweep: freed {} objects ({} bytes), {} survived",
            freed,
            freed_bytes,
            survived
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Shadow stack save/restore — used by coroutine scheduler for context switch.
// ---------------------------------------------------------------------------

/// Save the current thread's shadow stack top pointer.
pub(crate) fn save_shadow_stack_top() -> crate::coroutine::ShadowStackTop {
    use crate::coroutine::ShadowStackTop;
    THREAD_STATE.with(|cell| {
        let state = cell.get();
        if state.is_null() {
            return ShadowStackTop::NULL;
        }
        ShadowStackTop(unsafe { (*state).shadow_stack_top.load(Ordering::Relaxed) as *mut u8 })
    })
}

/// Restore the current thread's shadow stack top pointer.
pub(crate) fn restore_shadow_stack_top(top: crate::coroutine::ShadowStackTop) {
    THREAD_STATE.with(|cell| {
        let state = cell.get();
        if !state.is_null() {
            unsafe {
                (*state)
                    .shadow_stack_top
                    .store(top.0 as *mut ShadowFrame, Ordering::Release);
            }
        }
    })
}

unsafe fn data_ptr_to_header(data: *mut u8) -> Option<*mut ObjHeader> {
    unsafe {
        if data.is_null() {
            return None;
        }
        let header = (data as *mut ObjHeader).sub(1);
        Some(header)
    }
}
