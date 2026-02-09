extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
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
    ExitProcess(p.get_x());
}
