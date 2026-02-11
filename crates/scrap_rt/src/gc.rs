//! Stop-the-world mark-and-sweep garbage collector with stack-map-based root scanning.
//!
//! Design:
//! - GC roots are discovered by walking the stack via frame pointers (RBP chain)
//!   and consulting a compile-time stack map table emitted by Cranelift.
//! - Cooperative stop-the-world: all threads pause at `__scrap_yield` safepoints
//!   before the GC scans stacks.
//! - No shadow stack, no write barrier — roots are found precisely at safepoints.

// When gc-debug is off, counters used only in gc_log! appear unused.
#![cfg_attr(not(feature = "gc-debug"), allow(unused_variables, unused_assignments))]

use std::cell::Cell;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};

use crate::sync::{Condvar, Mutex};

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
// Stack map table — extern symbols emitted by codegen into the COFF object.
// ---------------------------------------------------------------------------

unsafe extern "C" {
    static __scrap_stackmap_count: u64;
    static __scrap_stackmap_index: u8; // start of IndexEntry array
    static __scrap_stackmap_roots: u8; // start of packed u32 array
}

/// Matches the binary layout emitted by `emit_stack_map_table` in codegen.
#[repr(C)]
#[derive(Clone, Copy)]
struct IndexEntry {
    return_addr: u64,
    roots_start: u32,
    roots_count: u32,
}

/// Sorted (by return_addr) copy of the stack map index, built at init time.
struct StackMapState {
    entries: Vec<IndexEntry>,
    roots: *const u32,
}

unsafe impl Send for StackMapState {}
unsafe impl Sync for StackMapState {}

static STACK_MAPS: std::sync::OnceLock<StackMapState> = std::sync::OnceLock::new();

fn init_stack_maps() {
    STACK_MAPS.get_or_init(|| unsafe {
        let count = __scrap_stackmap_count as usize;
        let index_ptr = &__scrap_stackmap_index as *const u8 as *const IndexEntry;
        let roots_ptr = &__scrap_stackmap_roots as *const u8 as *const u32;

        let mut entries: Vec<IndexEntry> = Vec::with_capacity(count);
        for i in 0..count {
            entries.push(index_ptr.add(i).read());
        }
        entries.sort_by_key(|e| e.return_addr);

        gc_log!("stack maps: {} entries loaded and sorted", count);

        StackMapState {
            entries,
            roots: roots_ptr,
        }
    });
}

/// Binary search for a stack map entry matching `return_addr`.
/// Returns (roots_start_index, roots_count) if found.
fn find_stack_map(return_addr: u64) -> Option<(usize, usize)> {
    let state = STACK_MAPS.get()?;
    let idx = state
        .entries
        .binary_search_by_key(&return_addr, |e| e.return_addr)
        .ok()?;
    let entry = &state.entries[idx];
    Some((entry.roots_start as usize, entry.roots_count as usize))
}

// ---------------------------------------------------------------------------
// Frame-pointer stack walking
// ---------------------------------------------------------------------------

/// Walk the RBP chain starting from `initial_rbp`, discovering GC roots
/// via the stack map table. Each discovered non-null root pointer is passed
/// to `mark_root`.
unsafe fn walk_stack_roots(initial_rbp: u64, mark_root: &mut impl FnMut(*mut u8)) {
    let state = match STACK_MAPS.get() {
        Some(s) => s,
        None => return,
    };
    if state.entries.is_empty() {
        return;
    }

    let mut rbp = initial_rbp;
    #[cfg(feature = "gc-debug")]
    let mut frames_walked: usize = 0;
    #[cfg(feature = "gc-debug")]
    let mut roots_found: usize = 0;

    unsafe {
        loop {
            if rbp == 0 {
                break;
            }
            // Validate pointer alignment and basic readability
            if rbp % 8 != 0 {
                break;
            }

            let return_addr = *((rbp + 8) as *const u64);
            if return_addr == 0 {
                break;
            }

            #[cfg(feature = "gc-debug")]
            {
                frames_walked += 1;
            }

            if let Some((roots_start, roots_count)) = find_stack_map(return_addr) {
                // caller_sp = RBP_callee + 16 (pushed return addr + pushed RBP)
                let caller_sp = rbp + 16;
                for i in 0..roots_count {
                    let sp_offset = *state.roots.add(roots_start + i);
                    let root_addr = (caller_sp + sp_offset as u64) as *const u64;
                    let root_val = *root_addr;
                    if root_val != 0 {
                        #[cfg(feature = "gc-debug")]
                        {
                            roots_found += 1;
                        }
                        mark_root(root_val as *mut u8);
                    }
                }
            }

            // Follow frame chain
            rbp = *(rbp as *const u64);
        }
    }

    gc_log!(
        "walk_stack: {} frames walked, {} roots found",
        frames_walked,
        roots_found
    );
}

// ---------------------------------------------------------------------------
// STW coordination
// ---------------------------------------------------------------------------

/// Set by the GC thread to request all other threads to pause at safepoints.
pub(crate) static GC_SCAN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Number of coroutines currently being executed by worker threads.
/// Incremented before resume, decremented after yield/complete.
pub(crate) static ACTIVE_COROS: AtomicU32 = AtomicU32::new(0);

pub(crate) struct GcPauseState {
    /// RBPs of threads/coroutines that have reached safepoints.
    pub(crate) parked_rbps: Vec<u64>,
    /// True when GC scanning + sweep is done, parked threads can resume.
    pub(crate) gc_done: bool,
}

pub(crate) static GC_PAUSE: Mutex<GcPauseState> = Mutex::new(GcPauseState {
    parked_rbps: Vec::new(),
    gc_done: false,
});

pub(crate) static GC_PAUSE_CONDVAR: Condvar = Condvar::new();

thread_local! {
    /// True on the thread currently running GC (should not park itself).
    static IS_GC_THREAD: Cell<bool> = const { Cell::new(false) };
    /// True on the main thread.
    static IS_MAIN_THREAD: Cell<bool> = const { Cell::new(false) };
}

/// Check if the current thread is the GC thread (called from coroutine.rs).
pub(crate) fn is_gc_thread() -> bool {
    IS_GC_THREAD.with(|c| c.get())
}

/// Called from `__scrap_yield` when `GC_SCAN_REQUESTED` is true.
/// The calling thread registers its current RBP and blocks until GC is done.
/// Must NOT be called on the GC thread itself.
pub(crate) fn gc_safepoint() {
    if IS_GC_THREAD.with(|c| c.get()) {
        return;
    }

    // Read current RBP
    let rbp: u64;
    unsafe { std::arch::asm!("mov {}, rbp", out(reg) rbp) };

    let mut state = mutex_lock!(GC_PAUSE);
    state.parked_rbps.push(rbp);
    GC_PAUSE_CONDVAR.notify_all(); // wake GC thread if waiting

    // Wait for GC to finish
    while !state.gc_done {
        condvar_wait!(GC_PAUSE_CONDVAR, state);
    }
}

// ---------------------------------------------------------------------------
// Global heap state (protected by mutex).
// ---------------------------------------------------------------------------

struct HeapState {
    all_objects: *mut ObjHeader,
    bytes_allocated: usize,
    threshold: usize,
}

// SAFETY: ObjHeader pointers are only accessed under the HEAP mutex or during
// STW (all other threads paused).
unsafe impl Send for HeapState {}

const INITIAL_HEAP_THRESHOLD: usize = 1024 * 1024; // 1MB

static HEAP: Mutex<HeapState> = Mutex::new(HeapState {
    all_objects: null_mut(),
    bytes_allocated: 0,
    threshold: INITIAL_HEAP_THRESHOLD,
});

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

    // Initialize stack map table (sort by return_addr for binary search)
    init_stack_maps();

    // Mark this as the main thread
    IS_MAIN_THREAD.with(|c| c.set(true));

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

// ---------------------------------------------------------------------------
// Collection: STW pause + mark + sweep
// ---------------------------------------------------------------------------

/// Run a full GC cycle. Caller must hold the HEAP mutex.
fn collect(heap: &mut HeapState) {
    gc_log!(
        "=== collection start: bytes_allocated={}, threshold={} ===",
        heap.bytes_allocated,
        heap.threshold
    );

    IS_GC_THREAD.with(|c| c.set(true));

    // Get our own RBP for stack scanning
    let gc_rbp: u64;
    unsafe { std::arch::asm!("mov {}, rbp", out(reg) gc_rbp) };

    // Request all threads to pause at safepoints
    {
        let mut ps = mutex_lock!(GC_PAUSE);
        ps.gc_done = false;
        ps.parked_rbps.clear();
    }
    GC_SCAN_REQUESTED.store(true, Ordering::Release);

    // Wait for all active coroutines to yield and park.
    // Workers decrement ACTIVE_COROS after yield, then check GC_SCAN_REQUESTED
    // and register their coroutine's RBP in GC_PAUSE.
    {
        let mut ps = mutex_lock!(GC_PAUSE);
        loop {
            let active = ACTIVE_COROS.load(Ordering::Acquire);
            if active == 0 {
                break;
            }
            gc_log!("collect: waiting for {} active coroutines to yield", active);
            condvar_wait!(GC_PAUSE_CONDVAR, ps);
        }
    }

    gc_log!("collect: all threads paused, scanning stacks");

    // Collect parked RBPs (from workers and possibly main thread safepoints)
    let parked_rbps: Vec<u64> = {
        let ps = mutex_lock!(GC_PAUSE);
        ps.parked_rbps.clone()
    };

    // Also get RBPs from all coroutines sitting in the scheduler queue
    let queued_rbps = crate::coroutine::get_queued_coro_rbps();

    // Mark phase: walk all stacks to discover roots
    unsafe {
        let mut worklist: Vec<*mut ObjHeader> = Vec::new();

        // 1. Walk the GC thread's own stack
        gc_log!("collect: walking GC thread stack (rbp={:#x})", gc_rbp);
        walk_stack_roots(gc_rbp, &mut |root| {
            if let Some(header) = data_ptr_to_header(root) {
                if (*header).mark.load(Ordering::Relaxed) == MARK_WHITE {
                    (*header).mark.store(MARK_GRAY, Ordering::Relaxed);
                    worklist.push(header);
                }
            }
        });

        // 2. Walk parked coroutine stacks (workers that yielded for GC)
        for &rbp in &parked_rbps {
            gc_log!("collect: walking parked coroutine stack (rbp={:#x})", rbp);
            walk_stack_roots(rbp, &mut |root| {
                if let Some(header) = data_ptr_to_header(root) {
                    if (*header).mark.load(Ordering::Relaxed) == MARK_WHITE {
                        (*header).mark.store(MARK_GRAY, Ordering::Relaxed);
                        worklist.push(header);
                    }
                }
            });
        }

        // 3. Walk queued coroutine stacks (already yielded, sitting in queue)
        for rbp in &queued_rbps {
            gc_log!("collect: walking queued coroutine stack (rbp={:#x})", rbp);
            walk_stack_roots(*rbp, &mut |root| {
                if let Some(header) = data_ptr_to_header(root) {
                    if (*header).mark.load(Ordering::Relaxed) == MARK_WHITE {
                        (*header).mark.store(MARK_GRAY, Ordering::Relaxed);
                        worklist.push(header);
                    }
                }
            });
        }

        gc_log!("collect: found {} root objects", worklist.len());

        // Trace from roots (transitive marking)
        #[cfg(feature = "gc-debug")]
        let mut traced: usize = 0;
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
        }
        gc_log!("collect: traced {} objects", traced);
    }

    // Sweep phase
    let bytes_before = heap.bytes_allocated;
    unsafe {
        sweep_phase(heap);
    }

    // Resume: signal all parked threads
    GC_SCAN_REQUESTED.store(false, Ordering::Release);
    {
        let mut ps = mutex_lock!(GC_PAUSE);
        ps.gc_done = true;
        ps.parked_rbps.clear();
    }
    GC_PAUSE_CONDVAR.notify_all();

    IS_GC_THREAD.with(|c| c.set(false));

    gc_log!(
        "=== collection end: reclaimed {} bytes, {} bytes remain ===",
        bytes_before.saturating_sub(heap.bytes_allocated),
        heap.bytes_allocated
    );
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

unsafe fn data_ptr_to_header(data: *mut u8) -> Option<*mut ObjHeader> {
    unsafe {
        if data.is_null() {
            return None;
        }
        let header = (data as *mut ObjHeader).sub(1);
        Some(header)
    }
}
