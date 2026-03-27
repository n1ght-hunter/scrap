extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn first<A, B>(a: A, b: B) -> A {
    a
}

fn main() {
    let x: usize = first(42, true);
    ExitProcess(x);
}
