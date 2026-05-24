extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::id_i128;

// `id_i128` passes/returns an `i128` scalar, which has no Cranelift lowering yet.
// Importing it must produce a clean "unsupported 128-bit scalar" diagnostic
// rather than silently truncating to 64 bits.
fn main() {
    let x = id_i128(5);
    __scrap_exit(0);
}
