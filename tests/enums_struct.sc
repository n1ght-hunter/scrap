extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

enum Message {
    Quit,
    Move { x: usize, y: usize },
}

fn main() {
    let msg = Message::Move { x: 42, y: 10 };
    let result = match msg {
        Message::Move { x, y } => x,
        Message::Quit => 0,
    };
    ExitProcess(result);
}
