extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn identity<T>(x: T) -> T {
    x
}

fn main() {
    let a: usize = identity(42);
    ExitProcess(a);
}
