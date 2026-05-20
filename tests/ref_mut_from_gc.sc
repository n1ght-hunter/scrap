extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let mut x: *usize = box(10);
    let r: &mut usize = &mut x;
    *r = 42;
    __scrap_exit(*x);
}
