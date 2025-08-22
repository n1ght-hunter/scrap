// Simple return functionality test file
// All return statements must end with semicolons

fn simple_return() -> i32 {
    return 42;
}

fn no_return_type() {
    return;
}

fn early_return(x: i32) -> i32 {
    if x > 10 {
        return x * 2;
    }
    return x + 1;
}

fn implicit_return() -> i32 {
    42
}

fn no_return_no_type() {
    let x = 5;
    let y = 10;
}

fn mixed_returns(x: i32) -> i32 {
    if x > 100 {
        return 999;
    }
    if x > 50 {
        return x * 5;
    }
    return x;
}
