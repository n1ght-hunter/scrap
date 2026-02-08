//@ run
//@ exit: 0

// Regression: `element_count == 0` used to be overloaded as "not an array",
// so a zero-length array of a pointer-bearing element type made the tracer
// walk one phantom element past the (empty) allocation and treat whatever
// bytes it found there as a pointer to trace.

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
    fn __scrap_gc_collect();
}

fn main() {
    let arr: **usize = alloc_array(0);
    __scrap_gc_collect();
    __scrap_gc_collect();
    __scrap_exit(0);
}
