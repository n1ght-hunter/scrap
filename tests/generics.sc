extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

// PASS: basic generic identity
fn identity<T>(x: T) -> T {
    x
}

// PASS: two type params
fn second<A, B>(a: A, b: B) -> B {
    b
}

// PASS: generic calling another generic
fn wrap<U>(x: U) -> U {
    identity(x)
}

// ERROR: undefined type P, should suggest T
fn bad_type<T>(x: P) -> T {
    x
}

// ERROR: type mismatch — T resolves to bool, assigned to i64
fn mismatch_user() {
    let a: i64 = identity(true);
}

fn main() {
    let a: usize = identity(42);
    let b: bool = identity(true);
    let c: usize = second(true, 10);
    let d: usize = wrap(42);
    ExitProcess(a);
}
