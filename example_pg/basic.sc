fn foo(a: f64, b: f64) -> f64 {
    let c = if a {
        if b {
            30.0
        } else {
            40.0
        }
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