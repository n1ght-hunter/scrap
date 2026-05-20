// Sample crate the spike driver analyzes. Mix of layouts the interop design must
// handle: repr(Rust) and repr(C) structs, an enum (discriminant + niche), and
// functions with scalar, aggregate-by-value, and reference args/returns.

pub struct Point {
    pub x: i32,
    pub y: i64,
}

#[repr(C)]
pub struct CPoint {
    pub x: i32,
    pub y: i64,
}

pub enum Shape {
    Unit,
    Circle(f64),
    Rect { w: f64, h: f64 },
}

pub fn scalar_add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn make_point(x: i32, y: i64) -> Point {
    Point { x, y }
}

pub fn point_sum(p: Point) -> i64 {
    p.x as i64 + p.y
}

pub fn point_ref(p: &Point) -> i64 {
    p.x as i64 + p.y
}

// Generic fn: the driver instantiates it at concrete args and queries the
// resulting monomorphized symbol / fn_abi.
pub fn identity<T>(x: T) -> T {
    x
}

// Forces Vec<i32> codegen so its monomorphized methods exist in the archive.
pub fn make_vec() -> Vec<i32> {
    vec![1, 2, 3]
}
