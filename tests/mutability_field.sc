//@ run
//@ exit: 42

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

struct Point {
    x: usize,
    y: usize,
}

fn main() {
    let mut p = Point { x: 10, y: 20 };
    p.x = 42;
    __scrap_exit(p.x);
}
