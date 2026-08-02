//@ run
//@ exit: 42

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn first<A, B>(a: A, b: B) -> A {
    a
}

fn main() {
    let x: usize = first(42, true);
    __scrap_exit(x);
}
