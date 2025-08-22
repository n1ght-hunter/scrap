fn test_return() -> i32 {
    return 42;
}

fn test_early_return(x: i32) -> i32 {
    if x > 10 {
        return x * 2;
    }
    return x + 1;
}

fn test_no_return_type() {
    return;
}

fn test_optional_return() {
    let x = 5;
}
