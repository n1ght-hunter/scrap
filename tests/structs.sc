extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

struct Point {
    x: usize,
    y: usize,
}

fn main() {
    let p = Point { x: 42, y: 10 };
    __scrap_exit(p.x);
}
