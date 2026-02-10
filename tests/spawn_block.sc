extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    spawn {
        ExitProcess(42);
    };
}
