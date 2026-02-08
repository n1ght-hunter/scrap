//! Scrap GC Runtime
//!
//! A mark-and-sweep garbage collector for the Scrap programming language.
//! Compiled as a staticlib (.lib) and linked into Scrap executables.

#![allow(unsafe_code)]

use std::ptr::null_mut;

// ---------------------------------------------------------------------------
// GcShape — type descriptor emitted by the compiler as a data section.
// Tells the GC the layout of an object (size, alignment, pointer field offsets).
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
    mark: u8,
    _pad: [u8; 7],
    size: u64,
    shape: *const GcShape,
    next: *mut ObjHeader,
}

impl ObjHeader {
    /// Get a pointer to the user data immediately after the header.
    fn data_ptr(&mut self) -> *mut u8 {
        unsafe { (self as *mut ObjHeader).add(1) as *mut u8 }
    }
}

// ---------------------------------------------------------------------------
// Shadow stack — precise root tracking.
// ---------------------------------------------------------------------------

#[repr(C)]
struct ShadowFrame {
    prev: *mut ShadowFrame,
    slots: *mut *mut u8, // pointer to array of GC root slots (on the function's stack)
    count: u64,
}

// ---------------------------------------------------------------------------
// Global GC state (single-threaded).
// ---------------------------------------------------------------------------

static mut GC_STATE: GcState = GcState {
    all_objects: null_mut(),
    shadow_stack_top: null_mut(),
    bytes_allocated: 0,
    threshold: 1024 * 1024, // 1 MB initial threshold
};

struct GcState {
    all_objects: *mut ObjHeader,
    shadow_stack_top: *mut ShadowFrame,
    bytes_allocated: usize,
    threshold: usize,
}

// ---------------------------------------------------------------------------
// Exported C ABI functions
// ---------------------------------------------------------------------------

/// Initialize the GC. Called from `_start` before `main`.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_init() {
    unsafe {
        GC_STATE.all_objects = null_mut();
        GC_STATE.shadow_stack_top = null_mut();
        GC_STATE.bytes_allocated = 0;
        GC_STATE.threshold = 1024 * 1024;
    }
}

/// Allocate a GC-managed object. Returns pointer to user data.
///
/// `shape` is a compiler-emitted type descriptor telling the GC the
/// object's size, alignment, and which fields contain GC pointers.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_alloc(shape: *const GcShape) -> *mut u8 {
    unsafe {
        let size = (*shape).size as usize;
        let total = std::mem::size_of::<ObjHeader>() + size;

        // Check if we should collect
        if GC_STATE.bytes_allocated + total > GC_STATE.threshold {
            __scrap_gc_collect();
            // If still over threshold after collection, grow it
            if GC_STATE.bytes_allocated + total > GC_STATE.threshold {
                GC_STATE.threshold = (GC_STATE.bytes_allocated + total) * 2;
            }
        }

        // Allocate header + user data
        let layout = std::alloc::Layout::from_size_align(total, 8).unwrap();
        let ptr = std::alloc::alloc_zeroed(layout) as *mut ObjHeader;
        if ptr.is_null() {
            // OOM — try collecting and retry once
            __scrap_gc_collect();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut ObjHeader;
            if ptr.is_null() {
                std::process::exit(101);
            }
        }

        // Initialize header
        (*ptr).mark = MARK_WHITE;
        (*ptr).size = size as u64;
        (*ptr).shape = shape;
        (*ptr).next = GC_STATE.all_objects;
        GC_STATE.all_objects = ptr;
        GC_STATE.bytes_allocated += total;

        // Return pointer to user data (after header)
        (*ptr).data_ptr()
    }
}

/// Force a garbage collection cycle.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_collect() {
    unsafe {
        mark_phase();
        sweep_phase();
    }
}

/// Push a shadow stack frame. Called at function entry for functions with GC roots.
///
/// `slots` points to a stack-allocated array of `count` pointer-sized slots.
/// Each slot holds a GC pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_push_frame(slots: *mut *mut u8, count: u64) {
    unsafe {
        // Allocate frame on the system heap (small, fixed size)
        let frame = Box::into_raw(Box::new(ShadowFrame {
            prev: GC_STATE.shadow_stack_top,
            slots,
            count,
        }));
        GC_STATE.shadow_stack_top = frame;
    }
}

/// Pop the top shadow stack frame. Called before function return.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_gc_pop_frame() {
    unsafe {
        let frame = GC_STATE.shadow_stack_top;
        if !frame.is_null() {
            GC_STATE.shadow_stack_top = (*frame).prev;
            drop(Box::from_raw(frame));
        }
    }
}

// ---------------------------------------------------------------------------
// Mark-and-sweep implementation
// ---------------------------------------------------------------------------

unsafe fn mark_phase() {
    unsafe {
        let mut worklist: Vec<*mut ObjHeader> = Vec::new();

        // Walk shadow stack to find roots
        let mut frame = GC_STATE.shadow_stack_top;
        while !frame.is_null() {
            let slots = (*frame).slots;
            let count = (*frame).count as usize;
            for i in 0..count {
                let slot_val = *slots.add(i);
                if !slot_val.is_null() {
                    if let Some(header) = data_ptr_to_header(slot_val) {
                        if (*header).mark == MARK_WHITE {
                            (*header).mark = MARK_GRAY;
                            worklist.push(header);
                        }
                    }
                }
            }
            frame = (*frame).prev;
        }

        // Process gray objects
        while let Some(obj) = worklist.pop() {
            (*obj).mark = MARK_BLACK;
            let shape = (*obj).shape;
            if shape.is_null() || (*shape).num_pointers == 0 {
                continue;
            }
            // Scan pointer fields in this object
            let data = (*obj).data_ptr();
            for &offset in (*shape).pointer_offsets() {
                let field_ptr = *(data.add(offset as usize) as *const *mut u8);
                if !field_ptr.is_null() {
                    if let Some(child_header) = data_ptr_to_header(field_ptr) {
                        if (*child_header).mark == MARK_WHITE {
                            (*child_header).mark = MARK_GRAY;
                            worklist.push(child_header);
                        }
                    }
                }
            }
        }
    }
}

unsafe fn sweep_phase() {
    unsafe {
        let mut prev: *mut *mut ObjHeader = &raw mut GC_STATE.all_objects;
        let mut current = GC_STATE.all_objects;

        while !current.is_null() {
            let next = (*current).next;

            if (*current).mark == MARK_WHITE {
                // Unreachable — free it
                *prev = next;
                let total = std::mem::size_of::<ObjHeader>() + (*current).size as usize;
                GC_STATE.bytes_allocated -= total;
                let layout = std::alloc::Layout::from_size_align(total, 8).unwrap();
                std::alloc::dealloc(current as *mut u8, layout);
            } else {
                // Reachable — reset for next cycle
                (*current).mark = MARK_WHITE;
                prev = &raw mut (*current).next;
            }

            current = next;
        }
    }
}

/// Convert a user data pointer back to its ObjHeader.
/// The header is immediately before the data pointer.
unsafe fn data_ptr_to_header(data: *mut u8) -> Option<*mut ObjHeader> {
    unsafe {
        if data.is_null() {
            return None;
        }
        let header = (data as *mut ObjHeader).sub(1);
        Some(header)
    }
}
