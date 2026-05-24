//! Affine use-after-move check for droppable native-Rust values.
//!
//! An owned `ir::Ty::Rust` value (one whose type has a drop wrapper) is moved
//! when passed by value to a Rust function, returned, or stored into another
//! value. Using it again after that would let two owners each run its
//! destructor — a double free. This forward dataflow rejects such programs (a
//! compile error, exactly as Rust does), which is what makes RAII drop sound.

use std::collections::{HashMap, HashSet};

use scrap_ir as ir;

use super::emit_codegen_err;

/// Whether an ABI type display is a reference (`&T`/`&mut T`) — a borrow, not a move.
fn is_reference(display: &str) -> bool {
    display.starts_with('&')
}

/// Emit a "use of moved value" diagnostic for every use of a droppable Rust
/// local that may already have been moved on some path to that use.
pub(crate) fn check_use_after_move<'db>(
    db: &'db dyn scrap_shared::Db,
    body: ir::Body<'db>,
    droppable: &HashSet<usize>,
    rust_fn_abis: &HashMap<String, scrap_rmeta::FnAbiInfo>,
) {
    if droppable.is_empty() {
        return;
    }
    let blocks = body.blocks(db);
    let n = blocks.len();

    // Predecessors, derived from each block's successors.
    let mut preds: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (b, blk) in blocks.iter().enumerate() {
        for s in successors(&blk.terminator(db)) {
            if s < n {
                preds[s].push(b);
            }
        }
    }

    // Forward dataflow: `moved_in[b]` = locals possibly-moved on entry to `b`.
    let mut moved_in: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    let mut changed = true;
    while changed {
        changed = false;
        for b in 0..n {
            let mut entry = HashSet::new();
            for &p in &preds[b] {
                let out = transfer(db, blocks[p], &moved_in[p], droppable, rust_fn_abis, None);
                entry.extend(out);
            }
            if entry != moved_in[b] {
                moved_in[b] = entry;
                changed = true;
            }
        }
    }

    // Final pass: re-run the transfer with diagnostics enabled.
    for b in 0..n {
        transfer(
            db,
            blocks[b],
            &moved_in[b],
            droppable,
            rust_fn_abis,
            Some((db, body)),
        );
    }
}

/// Successor block indices of a terminator (including cleanup edges).
fn successors(term: &ir::Terminator) -> Vec<usize> {
    match term {
        ir::Terminator::Goto { target } => vec![target.0],
        ir::Terminator::SwitchInt { targets, .. } => {
            let mut v: Vec<usize> = targets.values.iter().map(|(_, t)| t.0).collect();
            v.push(targets.otherwise.0);
            v
        }
        ir::Terminator::Call { target, unwind, .. } => {
            let mut v = Vec::new();
            if let Some(t) = target {
                v.push(t.0);
            }
            if let ir::UnwindAction::Cleanup(bb) = unwind {
                v.push(bb.0);
            }
            v
        }
        ir::Terminator::Assert { target, unwind, .. } => {
            let mut v = vec![target.0];
            if let ir::UnwindAction::Cleanup(bb) = unwind {
                v.push(bb.0);
            }
            v
        }
        ir::Terminator::Return | ir::Terminator::Unreachable => Vec::new(),
    }
}

/// Run one block's effect on the moved-set. When `diag` is `Some`, emit a
/// diagnostic for any use of an already-moved droppable local.
fn transfer<'db>(
    db: &'db dyn scrap_shared::Db,
    block: ir::BasicBlock<'db>,
    moved_in: &HashSet<usize>,
    droppable: &HashSet<usize>,
    rust_fn_abis: &HashMap<String, scrap_rmeta::FnAbiInfo>,
    diag: Option<(&'db dyn scrap_shared::Db, ir::Body<'db>)>,
) -> HashSet<usize> {
    let mut moved = moved_in.clone();

    let step =
        |uses: Vec<usize>, consumes: &[usize], def: Option<usize>, moved: &mut HashSet<usize>| {
            for u in uses {
                if droppable.contains(&u)
                    && moved.contains(&u)
                    && let Some((db, body)) = diag
                {
                    report(db, body, u);
                }
            }
            for &c in consumes {
                if droppable.contains(&c) {
                    moved.insert(c);
                }
            }
            if let Some(d) = def
                && droppable.contains(&d)
            {
                moved.remove(&d); // re-initialized → owned again
            }
        };

    for stmt in block.statements(db) {
        let ir::StatementKind::Assign(place, rvalue) = stmt.kind(db);
        let uses = rvalue_uses(&rvalue)
            .into_iter()
            .chain(place_proj_uses(&place))
            .collect();
        let consumes = rvalue_consumes(&rvalue);
        let def = bare_local(&place);
        step(uses, &consumes, def, &mut moved);
    }

    match block.terminator(db) {
        ir::Terminator::Call {
            func,
            args,
            destination,
            ..
        } => {
            // Only a native-Rust call moves Rust values; and only its
            // by-value (non-reference) args are consumes.
            let abi = match &func {
                ir::Operand::FunctionRef(fid) => rust_fn_abis.get(fid.text(db)),
                _ => None,
            };
            let mut uses = Vec::new();
            let mut consumes = Vec::new();
            for (i, a) in args.iter().enumerate() {
                if let ir::Operand::Place(p) = a {
                    if let Some(l) = bare_local(p) {
                        uses.push(l);
                        let by_ref = abi
                            .and_then(|abi| abi.args.get(i))
                            .is_some_and(|arg| is_reference(&arg.ty.display));
                        if abi.is_some() && !by_ref {
                            consumes.push(l);
                        }
                    } else {
                        uses.extend(place_proj_uses(p));
                    }
                }
            }
            let def = bare_local(&destination);
            step(uses, &consumes, def, &mut moved);
        }
        ir::Terminator::SwitchInt { discr, .. } => {
            if let ir::Operand::Place(p) = discr
                && let Some(l) = bare_local(&p)
            {
                step(vec![l], &[], None, &mut moved);
            }
        }
        ir::Terminator::Assert { cond, .. } => {
            if let ir::Operand::Place(p) = cond
                && let Some(l) = bare_local(&p)
            {
                step(vec![l], &[], None, &mut moved);
            }
        }
        _ => {}
    }

    moved
}

fn report<'db>(db: &'db dyn scrap_shared::Db, body: ir::Body<'db>, local: usize) {
    let name = body
        .local_decls(db)
        .get(local)
        .and_then(|d| d.name(db))
        .map(|s| s.text().to_string())
        .unwrap_or_else(|| format!("_{local}"));
    emit_codegen_err(
        db,
        format!(
            "use of moved Rust value `{name}`: it was already moved out (e.g. passed by value to a Rust function)"
        ),
    );
}

/// The bare local id if `place` is `Local(l)` (not a projection).
fn bare_local(place: &ir::Place) -> Option<usize> {
    match place {
        ir::Place::Local(l) => Some(l.0),
        _ => None,
    }
}

/// Local read through a place projection (`l.f`, `*l`, …) — a use of `l`.
fn place_proj_uses(place: &ir::Place) -> Vec<usize> {
    match place {
        ir::Place::Local(_) => Vec::new(),
        ir::Place::Field(b, _, _) | ir::Place::Deref(b) | ir::Place::Downcast(b, _, _) => {
            base_local(b).into_iter().collect()
        }
        ir::Place::__Phantom(_) => Vec::new(),
    }
}

/// The innermost local of a place projection.
fn base_local(place: &ir::Place) -> Option<usize> {
    match place {
        ir::Place::Local(l) => Some(l.0),
        ir::Place::Field(b, _, _) | ir::Place::Deref(b) | ir::Place::Downcast(b, _, _) => {
            base_local(b)
        }
        ir::Place::__Phantom(_) => None,
    }
}

fn operand_use(op: &ir::Operand) -> Option<usize> {
    match op {
        ir::Operand::Place(p) => base_local(p),
        _ => None,
    }
}

/// All locals read by an rvalue.
fn rvalue_uses(rv: &ir::Rvalue) -> Vec<usize> {
    let mut v = Vec::new();
    let mut add = |o: &ir::Operand| {
        if let Some(l) = operand_use(o) {
            v.push(l);
        }
    };
    match rv {
        ir::Rvalue::Use(o) | ir::Rvalue::Box(_, o) => add(o),
        ir::Rvalue::Intrinsic(_, ops) | ir::Rvalue::Array(ops) | ir::Rvalue::Aggregate(_, ops) => {
            ops.iter().for_each(&mut add)
        }
        ir::Rvalue::Spawn(o, ops) => {
            add(o);
            ops.iter().for_each(&mut add);
        }
        ir::Rvalue::Discriminant(p) | ir::Rvalue::Ref(_, p) => v.extend(base_local(p)),
        ir::Rvalue::Constant(_) => {}
    }
    v
}

/// Droppable locals consumed (moved by value) by an rvalue. A `Ref` borrow is
/// deliberately not a consume.
fn rvalue_consumes(rv: &ir::Rvalue) -> Vec<usize> {
    let mut v = Vec::new();
    let mut add = |o: &ir::Operand| {
        if let ir::Operand::Place(p) = o
            && let Some(l) = bare_local(p)
        {
            v.push(l);
        }
    };
    match rv {
        // `box(value)` moves the value into the GC heap (consumes it); the GC
        // finalizer now owns the drop.
        ir::Rvalue::Use(o) | ir::Rvalue::Box(_, o) => add(o),
        ir::Rvalue::Aggregate(_, ops) => ops.iter().for_each(&mut add),
        _ => {}
    }
    v
}
