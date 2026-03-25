//! Low-level context switching: SavedContext + naked function asm routines.
//!
//! Uses `extern "sysv64"` calling convention so that the compiler
//! handles saving/restoring XMM6-15 at the call site, keeping the
//! asm to ~30 instructions.
//!
//! Platform support:
//! - **Windows**: saves/restores callee-saved GPRs + TEB fields (StackBase,
//!   StackLimit, DeallocationStack) via `gs:[0x30]`.
//! - **Unix** (Linux, macOS): saves/restores callee-saved GPRs only.

#[cfg(not(target_arch = "x86_64"))]
compile_error!("scrap_coroutine currently only supports x86_64");

use std::arch::naked_asm;

/// Register state saved across a context switch.
///
/// Layout is chosen so that callee-saved GPRs and TEB fields are at
/// fixed offsets accessible from the asm. The TEB fields are only used
/// on Windows; on Unix they remain zeroed.
#[repr(C)]
pub(crate) struct SavedContext {
    pub(crate) rbx: u64, // offset  0
    pub(crate) rsp: u64, // offset  8
    pub(crate) rbp: u64, // offset 16
    _pad0: u64,          // offset 24 (alignment)
    pub(crate) r12: u64, // offset 32
    pub(crate) r13: u64, // offset 40
    pub(crate) r14: u64, // offset 48
    pub(crate) r15: u64, // offset 56
    #[cfg(windows)]
    _pad1: [u64; 3], // offsets 64, 72, 80 (alignment)
    #[cfg(windows)]
    pub(crate) teb_stack_base: u64, // offset 88
    #[cfg(windows)]
    pub(crate) teb_stack_limit: u64, // offset 96
    #[cfg(windows)]
    pub(crate) teb_dealloc: u64, // offset 104
}

impl SavedContext {
    pub(crate) fn zeroed() -> Self {
        // SAFETY: all-zero is a valid SavedContext (all fields are u64).
        unsafe { std::mem::zeroed() }
    }
}

/// Saves caller state to `out`, loads state from `in_`, continues at the
/// loaded RIP (via `ret`).
///
/// sysv64 ABI:
///   RDI = first arg (out), RSI = second arg (in_)
///   Callee-saved: RBX, RBP, R12-R15
///   Caller-saved: RAX, RCX, RDX, R8-R11, XMM0-15  (XMM handled by compiler)
#[cfg(windows)]
#[unsafe(naked)]
pub(crate) unsafe extern "sysv64" fn swap_registers(
    _out: *mut SavedContext,
    _in: *const SavedContext,
) {
    naked_asm!(
        // ----- Save current state into *RDI (out) -----
        "mov [rdi + 0*8], rbx",
        "mov [rdi + 1*8], rsp",
        "mov [rdi + 2*8], rbp",
        "mov [rdi + 4*8], r12",
        "mov [rdi + 5*8], r13",
        "mov [rdi + 6*8], r14",
        "mov [rdi + 7*8], r15",
        // Save TEB fields: gs:[0x30] → TEB pointer on x86_64 Windows
        "mov r10, gs:[0x30]",
        "mov rax, [r10 + 0x08]",
        "mov [rdi + 11*8], rax", // StackBase
        "mov rax, [r10 + 0x10]",
        "mov [rdi + 12*8], rax", // StackLimit
        "mov rax, [r10 + 0x1478]",
        "mov [rdi + 13*8], rax", // DeallocationStack
        // ----- Load state from *RSI (in_) -----
        "mov rbx, [rsi + 0*8]",
        "mov rsp, [rsi + 1*8]",
        "mov rbp, [rsi + 2*8]",
        "mov r12, [rsi + 4*8]",
        "mov r13, [rsi + 5*8]",
        "mov r14, [rsi + 6*8]",
        "mov r15, [rsi + 7*8]",
        // Restore TEB fields
        "mov rax, [rsi + 13*8]",
        "mov [r10 + 0x1478], rax", // DeallocationStack
        "mov rax, [rsi + 12*8]",
        "mov [r10 + 0x10], rax", // StackLimit
        "mov rax, [rsi + 11*8]",
        "mov [r10 + 0x08], rax", // StackBase
        // Return into the loaded context's return address (now on the new stack).
        "ret",
    );
}

#[cfg(unix)]
#[unsafe(naked)]
pub(crate) unsafe extern "sysv64" fn swap_registers(
    _out: *mut SavedContext,
    _in: *const SavedContext,
) {
    naked_asm!(
        // ----- Save current state into *RDI (out) -----
        "mov [rdi + 0*8], rbx",
        "mov [rdi + 1*8], rsp",
        "mov [rdi + 2*8], rbp",
        "mov [rdi + 4*8], r12",
        "mov [rdi + 5*8], r13",
        "mov [rdi + 6*8], r14",
        "mov [rdi + 7*8], r15",
        // No TEB on Unix.

        // ----- Load state from *RSI (in_) -----
        "mov rbx, [rsi + 0*8]",
        "mov rsp, [rsi + 1*8]",
        "mov rbp, [rsi + 2*8]",
        "mov r12, [rsi + 4*8]",
        "mov r13, [rsi + 5*8]",
        "mov r14, [rsi + 6*8]",
        "mov r15, [rsi + 7*8]",
        // Return into the loaded context's return address (now on the new stack).
        "ret",
    );
}

/// Initial entry point for newly created coroutines.
///
/// Entered via the first swap_registers into a new coroutine.
/// At that point the callee-saved registers hold:
///   R12 = closure_ptr (arg 1)
///   R13 = status_ptr  (arg 2)
///   R14 = coroutine_wrapper fn ptr (the Rust trampoline)
///
/// We pass R12/R13 to coroutine_wrapper via the sysv64 calling convention
/// (RDI / RSI), align the stack, and call into coroutine_wrapper.
///
/// Identical on all platforms (no TEB interaction).
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn bootstrap_green_task() {
    naked_asm!(
        // sysv64: first arg in RDI, second in RSI
        "mov rdi, r12",
        "mov rsi, r13",
        // 16-byte align the stack (required by sysv64 ABI at call site)
        "and rsp, -16",
        // Call the Rust trampoline. If it ever returns (it shouldn't after
        // the final yield_current), we'll hit ud2.
        "call r14",
        "ud2",
    );
}

/// Yield the current coroutine back to its caller.
///
/// No-op if called outside a coroutine (CALLER_CTX / CORO_CTX are null).
pub fn yield_current() {
    let caller = super::CALLER_CTX.with(|c| c.get());
    let coro = super::CORO_CTX.with(|c| c.get());
    if caller.is_null() || coro.is_null() {
        return; // Not inside a coroutine — no-op.
    }
    unsafe { swap_registers(coro, caller) };
}
