fn foo(a: f64, b: f64) -> f64 {
    let test = asdfasd;
    let c: f64 = if a > 1.0 {
        a + b
    } else {
        50.0
    };
    c + 2.0
}

fn baz() -> i32 {
    42
}

fn qux() -> bool {
    true
}

fn foo_no_rt(a: f64, b: f64) {
    let _c = if a > 1.0 {
        a + b
    } else {
        50.0
    };
}

fn bar() -> String {
    "Hello, \\world!"
}

enum MyEnum {
    Variant1,
    Variant2(MyStruct),
}

struct MyStruct {
    field1: i32,
    field2: String,
}