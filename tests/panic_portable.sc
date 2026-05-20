fn add(a: usize, b: usize) -> usize {
    a + b
}

fn main() {
    let max: usize = 18446744073709551615;
    let result = add(max, 1);
}
