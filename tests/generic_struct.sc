//@ run
//@ exit: 42

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

struct Wrapper<T> {
    value: T,
}

fn main() {
    let w = Wrapper { value: 42 };
    __scrap_exit(w.value);
}
