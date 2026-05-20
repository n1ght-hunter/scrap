extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let x: *usize = box(42);
    let r: &usize = &x;
    __scrap_exit(*r);
}
