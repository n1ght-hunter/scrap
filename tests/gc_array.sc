//@ run
//@ exit: 7

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
    fn __scrap_gc_collect();
}

fn main() {
    let mut arr: *usize = alloc_array(3);
    *arr = 7;
    __scrap_gc_collect();
    __scrap_exit(*arr);
}
