extern "C" {
    fn __scrap_print(msg: String, len: usize);
}

fn main() {
    __scrap_print("Hello, World!\n", 14);
}
