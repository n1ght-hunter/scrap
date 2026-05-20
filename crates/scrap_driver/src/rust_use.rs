//! `use rust::<crate>::<item>;` → synthesized `extern "Rust"` bindings.
//!
//! Resolution is **import-scoped**, mirroring Rust: each `use rust::…` binds a
//! local name to one fully-qualified catalog path (unique → one mangled symbol).
//! Only imported functions enter the symbol map, keyed by their local name, so
//! two catalog functions sharing a final segment (`new`, `add`, …) never
//! collide unless the *same local name* is imported twice — an ambiguity the
//! user resolves with `as`, exactly as in Rust.
//!
//! For type checking we synthesize an `extern "Rust"` AST `ForeignMod` (so the
//! existing foreign-fn path registers the signature); for codegen we synthesize
//! a parallel IR module of `ExternFn`s (lowering reads each file's own AST, not
//! the resolved `Can`, so the AST externs don't reach it). Both are built from
//! the same Phase 2 metadata.

use std::collections::HashMap;

use scrap_ast::fndef::Param;
use scrap_ast::foreign::{ForeignItem, ForeignMod};
use scrap_ast::item::{Item, ItemKind, UseTreeKind};
use scrap_ast::pat::{Pat, PatKind};
use scrap_ast::typedef::{Ty as AstTy, TyKind};
use scrap_ast::{Visibility, VisibilityKind};
use scrap_diagnostics::Level;
use scrap_ir as ir;
use scrap_rmeta::RustMetadata;
use scrap_shared::NodeId;
use scrap_shared::ident::{Ident, Symbol};
use scrap_shared::path::Path;
use scrap_shared::types::{FloatTy, IntTy, UintTy};
use scrap_span::Span;
use thin_vec::ThinVec;

/// A non-generic Rust function from the catalog, by fully-qualified path.
pub struct CatalogFn {
    pub symbol: String,
    pub params: Vec<String>,
    pub ret: String,
}

/// Build the catalog (full path → fn) from interop metadata. Only monomorphic
/// functions (those with a concrete symbol) are included.
pub fn build_catalog(meta: &RustMetadata) -> HashMap<String, CatalogFn> {
    let mut map = HashMap::new();
    for krate in &meta.crates {
        for f in &krate.fns {
            if let Some(mono) = &f.mono {
                map.insert(
                    f.path.clone(),
                    CatalogFn {
                        symbol: mono.symbol.clone(),
                        params: f.params.iter().map(|p| p.display.clone()).collect(),
                        ret: f.ret.display.clone(),
                    },
                );
            }
        }
    }
    map
}

/// One `use rust::…` import: the local name it binds and the catalog path it
/// refers to (segments after the leading `rust`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub struct RustUseRef {
    pub local: String,
    pub path: String,
}

/// Scan a resolved `Can` for `use rust::<crate>::…;` items. Reads tracked
/// fields, so it must run as a tracked query.
#[salsa::tracked]
pub fn scan_rust_uses<'db>(
    db: &'db dyn scrap_shared::Db,
    can: scrap_ast::Can<'db>,
) -> Vec<RustUseRef> {
    let mut uses = Vec::new();
    for item in can.items(db) {
        let ItemKind::Use(tree) = &item.kind else {
            continue;
        };
        // Only simple `use a::b::c;` (optionally `as alias`) for now.
        let UseTreeKind::Simple(alias) = &tree.kind else {
            continue;
        };
        let segments = &tree.prefix.segments;
        if segments.len() < 3 || segments[0].ident.name.text() != "rust" {
            continue;
        }
        let path = segments[1..]
            .iter()
            .map(|s| s.ident.name.text().to_string())
            .collect::<Vec<_>>()
            .join("::");
        let local = match alias {
            Some(id) => id.name.text().to_string(),
            None => segments.last().unwrap().ident.name.text().to_string(),
        };
        uses.push(RustUseRef { local, path });
    }
    uses
}

/// A resolved import ready to synthesize into an IR `ExternFn`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub struct ResolvedImport<'db> {
    pub local: String,
    pub params: Vec<ir::Ty<'db>>,
    pub ret: ir::Ty<'db>,
}

/// The product of resolving `use rust::…` imports against the catalog.
pub struct ResolveOutcome<'db> {
    /// `extern "Rust"` AST items to inject into the `Can` for type checking.
    pub ast_items: ThinVec<Box<Item<'db>>>,
    /// Imports to synthesize as IR `ExternFn`s for codegen.
    pub imports: Vec<ResolvedImport<'db>>,
    /// Local name → mangled symbol, for codegen import linking.
    pub fn_symbols: HashMap<String, String>,
}

/// Map a Rust primitive type display name to an `ir::Ty`. Primitives carry no
/// `'db` data, so this is a plain (non-tracked) construction.
fn prim_ir<'db>(display: &str) -> Option<ir::Ty<'db>> {
    Some(match display {
        "i8" => ir::Ty::Int(IntTy::I8),
        "i16" => ir::Ty::Int(IntTy::I16),
        "i32" => ir::Ty::Int(IntTy::I32),
        "i64" => ir::Ty::Int(IntTy::I64),
        "isize" => ir::Ty::Int(IntTy::Isize),
        "u8" => ir::Ty::Uint(UintTy::U8),
        "u16" => ir::Ty::Uint(UintTy::U16),
        "u32" => ir::Ty::Uint(UintTy::U32),
        "u64" => ir::Ty::Uint(UintTy::U64),
        "usize" => ir::Ty::Uint(UintTy::Usize),
        "f32" => ir::Ty::Float(FloatTy::F32),
        "f64" => ir::Ty::Float(FloatTy::F64),
        "bool" => ir::Ty::Bool,
        _ => return None,
    })
}

fn prim_ast_ty(display: &str) -> AstTy {
    AstTy {
        id: NodeId::dummy(),
        kind: TyKind::Path(Path::from_segment(display)),
        span: Span::default(),
    }
}

fn dummy_pat() -> Box<Pat> {
    Box::new(Pat {
        id: NodeId::dummy(),
        kind: PatKind::Missing,
        span: Span::default(),
    })
}

fn emit_err(db: &dyn scrap_shared::Db, title: impl Into<String>, note: impl Into<String>) {
    db.dcx().emit_err(
        Level::ERROR
            .primary_title(title.into())
            .element(Level::NOTE.message(note.into())),
    );
}

/// Resolve scanned `use rust::…` imports against the catalog, emitting
/// diagnostics for unknown items, ambiguous local names, and not-yet-supported
/// (non-primitive / generic) signatures. Plain (no tracked-struct creation).
pub fn resolve_uses<'db>(
    db: &'db dyn scrap_shared::Db,
    use_refs: &[RustUseRef],
    catalog: &HashMap<String, CatalogFn>,
) -> ResolveOutcome<'db> {
    let mut foreign_items = ThinVec::new();
    let mut imports = Vec::new();
    let mut fn_symbols = HashMap::new();

    for u in use_refs {
        let Some(cat) = catalog.get(&u.path) else {
            emit_err(
                db,
                format!("no Rust item `{}`", u.path),
                "not found in the interop catalog (or it is generic — not yet supported)",
            );
            continue;
        };
        if fn_symbols.contains_key(&u.local) {
            emit_err(
                db,
                format!("`{}` is imported more than once", u.local),
                "import one with `use rust::… as other_name;`",
            );
            continue;
        }

        // Map the signature to primitives; bail on anything else for now.
        let mut param_ir = Vec::with_capacity(cat.params.len());
        let mut param_ast = ThinVec::new();
        let mut unsupported = false;
        for (i, p) in cat.params.iter().enumerate() {
            match prim_ir(p) {
                Some(ty) => {
                    param_ir.push(ty);
                    param_ast.push(Param {
                        id: NodeId::dummy(),
                        ident: Ident::dummy_with_name(&format!("arg{i}")),
                        ty: Box::new(prim_ast_ty(p)),
                        pat: dummy_pat(),
                        span: Span::default(),
                    });
                }
                None => unsupported = true,
            }
        }
        let (ret_ir, ret_ast) = if cat.ret == "()" {
            (ir::Ty::Void, None)
        } else {
            match prim_ir(&cat.ret) {
                Some(ty) => (ty, Some(prim_ast_ty(&cat.ret))),
                None => {
                    unsupported = true;
                    (ir::Ty::Void, None)
                }
            }
        };
        if unsupported {
            emit_err(
                db,
                format!("`{}` uses non-primitive Rust types", u.path),
                "only primitive scalar params/returns are supported so far",
            );
            continue;
        }

        foreign_items.push(ForeignItem {
            id: NodeId::dummy(),
            ident: Ident::dummy_with_name(&u.local),
            args: param_ast,
            ret_type: ret_ast,
            span: Span::default(),
        });
        imports.push(ResolvedImport {
            local: u.local.clone(),
            params: param_ir,
            ret: ret_ir,
        });
        fn_symbols.insert(u.local.clone(), cat.symbol.clone());
    }

    let mut ast_items = ThinVec::new();
    if !foreign_items.is_empty() {
        let foreign_mod = ForeignMod {
            abi: Symbol::new("Rust"),
            items: foreign_items,
            span: Span::default(),
        };
        ast_items.push(Box::new(Item {
            kind: ItemKind::ForeignMod(foreign_mod),
            span: Span::default(),
            id: NodeId::dummy(),
            vis: Visibility {
                kind: VisibilityKind::Inherited,
                span: Span::default(),
            },
        }));
    }

    ResolveOutcome {
        ast_items,
        imports,
        fn_symbols,
    }
}

/// Append synthesized `extern "Rust"` items to a `Can` (tracked: it creates a
/// new `Can`). Passing the items directly mirrors `create_can`.
#[salsa::tracked]
pub fn rebuild_can<'db>(
    db: &'db dyn scrap_shared::Db,
    can: scrap_ast::Can<'db>,
    extra: ThinVec<Box<Item<'db>>>,
) -> scrap_ast::Can<'db> {
    let mut items = can.items(db).clone();
    items.extend(extra);
    scrap_ast::Can::new(db, can.id(db), *can.name(db), items)
}

/// Build an IR module of `ExternFn`s for the resolved imports (tracked: it
/// creates IR structs). `_seed` is an unused salsa-struct key (salsa requires at
/// least one salsa-struct argument).
#[salsa::tracked]
pub fn synth_extern_module<'db>(
    db: &'db dyn scrap_shared::Db,
    _seed: scrap_ast::Can<'db>,
    imports: Vec<ResolvedImport<'db>>,
) -> ir::Module<'db> {
    let items = imports
        .iter()
        .map(|imp| {
            let sig = ir::Signature::new(
                db,
                Symbol::new(&imp.local),
                imp.params.clone(),
                imp.ret.clone(),
            );
            ir::Items::ExternFunction(ir::ExternFn::new(db, Symbol::new("Rust"), sig))
        })
        .collect();
    let module_id = scrap_shared::id::ModuleId::from_path(db, &Path::from_segment("__rust_interop"));
    ir::Module::new(db, module_id, items)
}
