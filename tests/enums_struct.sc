//@ run
//@ exit: 42

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
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
    __scrap_exit(result);
}
