extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
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
    ExitProcess(c.add(10));
}
