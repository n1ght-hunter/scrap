//@ run
//@ exit: 42

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn identity<T>(x: T) -> T {
    x
}

fn main() {
    let a: usize = identity(42);
    __scrap_exit(a);
}
