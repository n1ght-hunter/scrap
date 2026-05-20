//! Cross-platform panic and process-exit entry points.
//!
//! Codegen emits calls to these instead of raw OS APIs (`WriteFile`/`ExitProcess`
//! on Windows), so the same compiled output works on every Tier 1 platform.

use std::io::Write;

/// Print a panic message to stderr and terminate the process with code 101
/// (matching Rust's panic exit code). Diverges.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_panic(msg_ptr: *const u8, msg_len: usize) -> ! {
    if !msg_ptr.is_null() && msg_len > 0 {
        let msg = unsafe { std::slice::from_raw_parts(msg_ptr, msg_len) };
        let mut stderr = std::io::stderr();
        let _ = stderr.write_all(msg);
        let _ = stderr.flush();
    }
    std::process::exit(101);
}

/// Terminate the process with the given exit code. Diverges.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_exit(code: i64) -> ! {
    std::process::exit(code as i32);
}
