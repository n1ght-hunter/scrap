//@ run
//@ exit: 9

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
    fn __scrap_gc_collect();
}

fn main() {
    let mut arr: *usize = alloc_array(4);
    arr[0] = 1;
    arr[1] = 3;
    arr[3] = 5;
    __scrap_gc_collect();
    __scrap_exit(arr[0] + arr[1] + arr[3]);
}
