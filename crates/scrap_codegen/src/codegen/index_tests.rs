//! Codegen tests for bounds-checked indexing (`Rvalue::ArrayLen`, `Place::Index`,
//! `AssertMessage::BoundsCheck`).
//!
//! There is no front-end producing these nodes yet at the time this test was written
//! independently of the full pipeline, so the IR is built by hand — exercising the
//! `ArrayLen` load, the `Assert` bounds-check branch, and the indexed load/store address
//! computation through the Cranelift verifier. Tracked-struct construction must happen
//! inside a `#[salsa::tracked]` function, hence the `build_and_compile` indirection.

use scrap_ir as ir;
use scrap_shared::id::ModuleId;
use scrap_shared::ident::Symbol;
use scrap_shared::path::Path;
use scrap_shared::types::{UintTy, UintVal};
use target_lexicon::Triple;

use super::CodegenContext;

/// Trivial salsa input so `build_and_compile` is a well-formed tracked query.
#[salsa::input]
struct Seed {
    #[returns(clone)]
    val: u32,
}

/// Build, by hand, `fn use_index(arr: *usize) -> usize { arr[0] = 42; arr[0] }`:
///
/// ```text
/// bb0:
///   _2 = array_len(_1)
///   _3 = 0usize < _2
///   assert(_3, "index out of bounds") -> bb1
/// bb1:
///   _1[0] = 42usize
///   _0 = _1[0]
///   return
/// ```
///
/// Returns whether codegen (through the Cranelift verifier) succeeded.
#[salsa::tracked(returns(clone))]
fn build_and_compile(db: &dyn scrap_shared::Db, _seed: Seed) -> bool {
    let usize_const = |v: usize| ir::Operand::Constant(ir::Constant::Uint(UintVal::Usize(v)));

    // _0 = usize return place, _1 = *usize param, _2 = usize len temp, _3 = bool ok temp.
    let l0 = ir::LocalDecl::new(db, None, ir::Ty::Uint(UintTy::Usize));
    let l1 = ir::LocalDecl::new(db, None, ir::Ty::Ptr(Box::new(ir::Ty::Uint(UintTy::Usize))));
    let l2 = ir::LocalDecl::new(db, None, ir::Ty::Uint(UintTy::Usize));
    let l3 = ir::LocalDecl::new(db, None, ir::Ty::Bool);

    let array_len = ir::Statement::new(
        db,
        ir::StatementKind::Assign(
            ir::Place::Local(ir::LocalId(2)),
            ir::Rvalue::ArrayLen(ir::Operand::Place(ir::Place::Local(ir::LocalId(1)))),
        ),
    );
    let bounds_ok = ir::Statement::new(
        db,
        ir::StatementKind::Assign(
            ir::Place::Local(ir::LocalId(3)),
            ir::Rvalue::Intrinsic(
                ir::IntrinsicOp::Lt,
                vec![
                    usize_const(0),
                    ir::Operand::Place(ir::Place::Local(ir::LocalId(2))),
                ],
            ),
        ),
    );
    let bb0 = ir::BasicBlock::new(
        db,
        vec![array_len, bounds_ok],
        ir::Terminator::Assert {
            cond: ir::Operand::Place(ir::Place::Local(ir::LocalId(3))),
            expected: true,
            msg: ir::AssertMessage::BoundsCheck,
            target: ir::BasicBlockId(1),
            unwind: ir::UnwindAction::Continue,
        },
    );

    let index_place = ir::Place::Index(
        Box::new(ir::Place::Local(ir::LocalId(1))),
        Box::new(usize_const(0)),
    );
    let write = ir::Statement::new(
        db,
        ir::StatementKind::Assign(index_place.clone(), ir::Rvalue::Use(usize_const(42))),
    );
    let read = ir::Statement::new(
        db,
        ir::StatementKind::Assign(
            ir::Place::Local(ir::LocalId(0)),
            ir::Rvalue::Use(ir::Operand::Place(index_place)),
        ),
    );
    let bb1 = ir::BasicBlock::new(db, vec![write, read], ir::Terminator::Return);

    let body = ir::Body::new(db, vec![bb0, bb1], vec![l0, l1, l2, l3], 1);
    let sig = ir::Signature::new(
        db,
        Symbol::new("use_index"),
        vec![ir::Ty::Ptr(Box::new(ir::Ty::Uint(UintTy::Usize)))],
        ir::Ty::Uint(UintTy::Usize),
    );
    let func = ir::Function::new(db, sig, body);

    let module_id = ModuleId::from_path(db, &Path::from_segment("test"));
    let module = ir::Module::new(db, module_id, vec![ir::Items::Function(func)]);

    let mut ctx = match CodegenContext::new(db, &Triple::host()) {
        Some(c) => c,
        None => return false,
    };

    ctx.compile_module(module).is_some()
}

#[test]
fn indexed_load_and_store_compiles() {
    let db = scrap_shared::salsa::ScrapDb::default();
    let seed = Seed::new(&db, 0);
    assert!(
        build_and_compile(&db, seed),
        "compiling a bounds-checked indexed load/store should succeed"
    );
}
