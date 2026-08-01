//@ run
//@ exit: 42

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

struct Counter {
    value: usize,
}

impl Counter {
    fn add(self, n: usize) -> usize {
        self.value + n
    }
}

fn main() {
    let c = Counter { value: 32 };
    __scrap_exit(c.add(10));
}
