//@ compile-fail
//@ error: cannot assign to data behind a `&` reference

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let mut x: usize = 5;
    let r: &usize = &x;
    *r = 10;
    __scrap_exit(*r);
}
