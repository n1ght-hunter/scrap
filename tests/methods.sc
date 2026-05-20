extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn get_x(&self) -> usize {
        self.x
    }
}

fn main() {
    let p = Point { x: 42, y: 10 };
    __scrap_exit(p.get_x());
}
