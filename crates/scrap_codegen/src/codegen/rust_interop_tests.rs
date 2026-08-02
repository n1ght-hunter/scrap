//! Codegen tests for memory-backed native Rust interop values (`ir::Ty::Rust`).
//!
//! There is no front-end producing `Ty::Rust` yet (that is Phase 4), so these
//! build the IR by hand and drive codegen with a mirrored layout — exercising
//! the stack-slot allocation and field store/load at metadata offsets that
//! Phases 4/5 will rely on. Tracked-struct construction must happen inside a
//! `#[salsa::tracked]` function, hence the `build_and_compile` indirection.

use std::collections::HashMap;

use cranelift::prelude::types;
use scrap_ir as ir;
use scrap_shared::id::ModuleId;
use scrap_shared::ident::Symbol;
use scrap_shared::path::Path;
use scrap_shared::types::{IntTy, IntVal};
use target_lexicon::Triple;

use super::CodegenContext;
use super::context::{RustFieldLayout, RustLayout};

/// Trivial salsa input so `build_and_compile` is a well-formed tracked query.
#[salsa::input]
struct Seed {
    #[returns(clone)]
    val: u32,
}

/// Build, by hand, a function that constructs a Rust value field-by-field, reads
/// a field back at its mirrored (reordered) offset, and returns it — then drive
/// codegen with a layout mirroring repr(Rust) `Point { x: i32, y: i64 }` (`y` at
/// offset 0, `x` at offset 8). Returns whether codegen succeeded.
#[salsa::tracked(returns(clone))]
fn build_and_compile(db: &dyn scrap_shared::Db, _seed: Seed) -> bool {
    let point = ir::TypeId::new(db, "rmeta_fixture::Point".to_string());

    // _0 = i32 return place, _1 = the Rust value.
    let l0 = ir::LocalDecl::new(db, None, ir::Ty::Int(IntTy::I32));
    let l1 = ir::LocalDecl::new(db, None, ir::Ty::Rust(point));

    // _1 = Point { x: 7i32, y: 9i64 }  (operands in declaration order)
    let construct = ir::Statement::new(
        db,
        ir::StatementKind::Assign(
            ir::Place::Local(ir::LocalId(1)),
            ir::Rvalue::Aggregate(
                ir::AggregateKind::Struct(point, vec![Symbol::new("x"), Symbol::new("y")]),
                vec![
                    ir::Operand::Constant(ir::Constant::Int(IntVal::I32(7))),
                    ir::Operand::Constant(ir::Constant::Int(IntVal::I64(9))),
                ],
            ),
        ),
    );
    // _0 = _1.x  (field 0, read at the mirrored offset)
    let read = ir::Statement::new(
        db,
        ir::StatementKind::Assign(
            ir::Place::Local(ir::LocalId(0)),
            ir::Rvalue::Use(ir::Operand::Place(ir::Place::Field(
                Box::new(ir::Place::Local(ir::LocalId(1))),
                0,
                None,
            ))),
        ),
    );
    let bb0 = ir::BasicBlock::new(db, vec![construct, read], ir::Terminator::Return);

    let body = ir::Body::new(db, vec![bb0], vec![l0, l1], 0);
    let sig = ir::Signature::new(
        db,
        Symbol::new("use_point"),
        vec![],
        ir::Ty::Int(IntTy::I32),
    );
    let func = ir::Function::new(db, sig, body);

    let module_id = ModuleId::from_path(db, &Path::from_segment("test"));
    let module = ir::Module::new(db, module_id, vec![ir::Items::Function(func)]);

    let mut ctx = match CodegenContext::new(db, &Triple::host()) {
        Some(c) => c,
        None => return false,
    };
    let mut layouts = HashMap::new();
    layouts.insert(
        "rmeta_fixture::Point".to_string(),
        RustLayout {
            size: 16,
            align: 8,
            fields: vec![
                RustFieldLayout {
                    offset: 8,
                    cl_ty: Some(types::I32),
                },
                RustFieldLayout {
                    offset: 0,
                    cl_ty: Some(types::I64),
                },
            ],
        },
    );
    ctx.set_rust_layouts(layouts);

    ctx.compile_module(module).is_some()
}

#[test]
fn rust_value_construct_and_field_read_compiles() {
    let db = scrap_shared::salsa::ScrapDb::default();
    let seed = Seed::new(&db, 0);
    assert!(
        build_and_compile(&db, seed),
        "compiling a memory-backed Ty::Rust value should succeed"
    );
}

#[test]
fn rust_layout_align_shift() {
    let layout = RustLayout {
        size: 16,
        align: 8,
        fields: vec![],
    };
    assert_eq!(layout.align_shift(), 3); // log2(8)
    assert_eq!(
        RustLayout {
            size: 1,
            align: 1,
            fields: vec![]
        }
        .align_shift(),
        0
    );
}
