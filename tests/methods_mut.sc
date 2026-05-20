extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

struct Pair {
    a: usize,
    b: usize,
}

impl Pair {
    fn sum(&mut self) -> usize {
        self.a + self.b
    }
}

fn main() {
    let p = Pair { a: 32, b: 10 };
    __scrap_exit(p.sum());
}
