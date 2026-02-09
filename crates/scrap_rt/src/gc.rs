//! Concurrent mark-and-sweep garbage collector with per-thread shadow stacks.
//!
//! Design:
//! - Each thread has its own `ThreadState` with a shadow stack (lock-free push/pop)
//! - Global heap state protected by a `Mutex`
//! - Tri-color marking with concurrent write barrier (Dijkstra insertion barrier)
//! - GC phase flag: Idle → Mark → Sweep → Idle
//! - Write barrier fast path: one atomic load + one branch (near-zero cost when idle)

use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicU8, Ordering};
use std::sync::Mutex;

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
        THREAD_REGISTRY.lock().unwrap().push(SendPtr(state));
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

static HEAP: Mutex<HeapState> = Mutex::new(HeapState {
    all_objects: null_mut(),
    bytes_allocated: 0,
    threshold: 1024 * 1024,
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
    // Reset global state
    {
        let mut heap = HEAP.lock().unwrap();
        heap.all_objects = null_mut();
        heap.bytes_allocated = 0;
        heap.threshold = 1024 * 1024;
    }
    GC_PHASE.store(PHASE_IDLE, Ordering::Relaxed);

    // Register the main thread
    ensure_thread_registered();
}

/// Allocate a GC-managed object. Returns pointer to user data.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_alloc(shape: *const GcShape) -> *mut u8 {
    unsafe {
        let size = (*shape).size as usize;
        let total = std::mem::size_of::<ObjHeader>() + size;

        let mut heap = HEAP.lock().unwrap();

        // Check if we should collect
        if heap.bytes_allocated + total > heap.threshold {
            collect(&mut heap);
            if heap.bytes_allocated + total > heap.threshold {
                heap.threshold = (heap.bytes_allocated + total) * 2;
            }
        }

        // Allocate header + user data
        let layout = std::alloc::Layout::from_size_align(total, 8).unwrap();
        let ptr = std::alloc::alloc_zeroed(layout) as *mut ObjHeader;
        if ptr.is_null() {
            collect(&mut heap);
            let ptr = std::alloc::alloc_zeroed(layout) as *mut ObjHeader;
            if ptr.is_null() {
                std::process::exit(101);
            }
            init_obj_header(ptr, size, shape, &mut heap);
            return (*ptr).data_ptr();
        }

        init_obj_header(ptr, size, shape, &mut heap);
        (*ptr).data_ptr()
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
    let mut heap = HEAP.lock().unwrap();
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
                BARRIER_WORKLIST.lock().unwrap().push(SendPtr(header));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Collection: mark + sweep
// ---------------------------------------------------------------------------

/// Run a full GC cycle. Caller must hold the HEAP mutex.
fn collect(heap: &mut HeapState) {
    GC_PHASE.store(PHASE_MARK, Ordering::Release);

    unsafe {
        mark_phase();
    }

    GC_PHASE.store(PHASE_SWEEP, Ordering::Release);

    unsafe {
        sweep_phase(heap);
    }

    GC_PHASE.store(PHASE_IDLE, Ordering::Release);
}

// ---------------------------------------------------------------------------
// Mark phase — scans all thread shadow stacks, traces heap.
// ---------------------------------------------------------------------------

unsafe fn mark_phase() {
    unsafe {
        let mut worklist: Vec<*mut ObjHeader> = Vec::new();

        // Snapshot thread registry (brief lock)
        let threads: Vec<SendPtr<ThreadState>> = THREAD_REGISTRY.lock().unwrap().clone();

        // Scan all thread shadow stacks for roots
        for &SendPtr(thread_state) in &threads {
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
                            }
                        }
                    }
                }
                frame = (*frame).prev;
            }
        }

        // Process gray objects (trace from roots)
        while let Some(obj) = worklist.pop() {
            (*obj).mark.store(MARK_BLACK, Ordering::Relaxed);

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
            if let Ok(mut barrier) = BARRIER_WORKLIST.try_lock() {
                for SendPtr(obj) in barrier.drain(..) {
                    worklist.push(obj);
                }
            }
        }

        // Final drain — ensure no barrier entries were missed
        let mut barrier = BARRIER_WORKLIST.lock().unwrap();
        let mut remaining: Vec<*mut ObjHeader> = barrier.drain(..).map(|SendPtr(p)| p).collect();
        drop(barrier);

        // Process any final barrier entries (and their transitive children)
        while let Some(obj) = remaining.pop() {
            if (*obj).mark.load(Ordering::Relaxed) != MARK_BLACK {
                (*obj).mark.store(MARK_BLACK, Ordering::Relaxed);

                let shape = (*obj).shape;
                if !shape.is_null() && (*shape).num_pointers > 0 {
                    let data = (*obj).data_ptr();
                    for &offset in (*shape).pointer_offsets() {
                        let field_ptr = *(data.add(offset as usize) as *const *mut u8);
                        if !field_ptr.is_null() {
                            if let Some(child_header) = data_ptr_to_header(field_ptr) {
                                if (*child_header).mark.load(Ordering::Relaxed) == MARK_WHITE {
                                    (*child_header)
                                        .mark
                                        .store(MARK_GRAY, Ordering::Relaxed);
                                    remaining.push(child_header);
                                }
                            }
                        }
                    }
                }
            }
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

        while !current.is_null() {
            let next = (*current).next;

            if (*current).mark.load(Ordering::Relaxed) == MARK_WHITE {
                // Unreachable — free it
                *prev = next;
                let total = std::mem::size_of::<ObjHeader>() + (*current).size as usize;
                heap.bytes_allocated -= total;
                let layout = std::alloc::Layout::from_size_align(total, 8).unwrap();
                std::alloc::dealloc(current as *mut u8, layout);
            } else {
                // Reachable — reset mark for next cycle
                (*current).mark.store(MARK_WHITE, Ordering::Relaxed);
                prev = &raw mut (*current).next;
            }

            current = next;
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

unsafe fn data_ptr_to_header(data: *mut u8) -> Option<*mut ObjHeader> {
    unsafe {
        if data.is_null() {
            return None;
        }
        let header = (data as *mut ObjHeader).sub(1);
        Some(header)
    }
}
