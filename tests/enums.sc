extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

enum Option {
    None,
    Some(usize),
}

fn main() {
    let x = Option::Some(42);
    let result = match x {
        Option::Some(val) => val,
        Option::None => 0,
    };
    ExitProcess(result);
}
