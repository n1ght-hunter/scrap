extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

struct Wrapper<T> {
    value: T,
}

fn main() {
    let w = Wrapper { value: 42 };
    ExitProcess(w.value);
}
