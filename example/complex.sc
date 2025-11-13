mod extenal_module;
use inline_module::greet;
fn main() {
    extenal_module::greet();
    use inline_module::greet;
    greet();
    print("Hello, world!");
}

mod inline_module {
    pub fn greet() {
        print("Greetings from inline module!");
    }
}