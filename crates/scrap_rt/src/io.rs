//! Minimal cross-platform I/O entry points for compiled Scrap programs.

use std::io::Write;

/// Write `len` bytes from `ptr` to stdout. No-op on a null/empty buffer.
/// Cross-platform replacement for programs that previously called Win32
/// `WriteFile` directly.
#[unsafe(no_mangle)]
pub extern "C" fn __scrap_print(ptr: *const u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let mut out = std::io::stdout();
    let _ = out.write_all(bytes);
    let _ = out.flush();
}
