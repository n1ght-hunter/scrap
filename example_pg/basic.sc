fn foo(a: f64, b: f64) -> f64 {
    let c = if a > 1.0 {
        a + b
    } else {
        50.0
    };
    c + 2.0
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