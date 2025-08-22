// Test file for return functionality
// This file tests various return statement scenarios

// Function with explicit return type and return statement
fn explicit_return() -> i32 {
    return 42;
}

// Function with explicit return type but no return statement (implicit return)
fn implicit_return() -> i32 {
    42
}

// Function without return type (void function)
fn no_return_type() {
    let x = 5;
    let y = 10;
}

// Function without return type but with return statement
fn void_with_return() {
    let x = 5;
    return;
}

// Function with early return based on condition
fn early_return(x: i32) -> i32 {
    if x > 10 {
        return x * 2;
    }
    return x + 1;
}

// Function with return in nested block
fn nested_return(x: i32) -> i32 {
    if x > 0 {
        if x > 100 {
            return 999;
        }
        return x * 10;
    }
    return 0;
}
