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

use scrap_ast::enumdef::VariantData;
use scrap_ast::field::FieldDef;
use scrap_ast::fndef::Param;
use scrap_ast::foreign::{ForeignItem, ForeignMod};
use scrap_ast::generics::Generics;
use scrap_ast::item::{Item, ItemKind, UseTreeKind};
use scrap_ast::pat::{Pat, PatKind};
use scrap_ast::structdef::StructDef;
use scrap_ast::typedef::{Ty as AstTy, TyKind};
use scrap_ast::{Visibility, VisibilityKind};
use scrap_codegen::codegen::context::RustLayout;
use scrap_diagnostics::Level;
use scrap_ir as ir;
use scrap_rmeta::{RustMetadata, RustType};
use scrap_shared::NodeId;
use scrap_shared::ident::{Ident, Symbol};
use scrap_shared::path::Path;
use scrap_shared::types::{FloatTy, IntTy, UintTy};
use scrap_span::Span;
use scrap_tycheck::{RustFieldVis, RustTypeVis};
use thin_vec::ThinVec;

/// A non-generic Rust function from the catalog, by fully-qualified path.
pub struct CatalogFn {
    pub symbol: String,
    pub params: Vec<String>,
    pub ret: String,
    /// The exact per-arg/return ABI, used to lower the call (Phase 5).
    pub abi: scrap_rmeta::FnAbiInfo,
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
                        abi: mono.abi.clone(),
                    },
                );
            }
        }
    }
    map
}

/// Build the type catalog (full path → type) from interop metadata. Only
/// non-generic types (those with a concrete layout) are included.
pub fn build_type_catalog(meta: &RustMetadata) -> HashMap<String, &RustType> {
    let mut map = HashMap::new();
    for krate in &meta.crates {
        for t in &krate.types {
            if t.layout.is_some() {
                map.insert(t.path.clone(), t);
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
    /// `extern "Rust"` foreign-fn and synthesized `struct` AST items to inject
    /// into the `Can` for type checking.
    pub ast_items: ThinVec<Box<Item<'db>>>,
    /// Imports to synthesize as IR `ExternFn`s for codegen.
    pub imports: Vec<ResolvedImport<'db>>,
    /// Local name → mangled symbol, for codegen import linking.
    pub fn_symbols: HashMap<String, String>,
    /// Visibility facts for imported Rust types, for tycheck construction gating.
    pub rust_vis: Vec<RustTypeVis>,
    /// Mirrored layouts of imported Rust types (by local name), for codegen.
    pub rust_layouts: HashMap<String, RustLayout>,
    /// Local fn name → ABI, for codegen call marshalling (Phase 5).
    pub rust_fn_abis: HashMap<String, scrap_rmeta::FnAbiInfo>,
    /// Imported types that need dropping: `(Scrap local name, full Rust path)`.
    /// Drives drop-wrapper generation (anchor) + RAII drop (codegen).
    pub droppable: Vec<(String, String)>,
}

/// Mutable accumulators shared across import resolution. Bundled so the fn/type
/// import helpers don't need a dozen `&mut` parameters each.
#[derive(Default)]
struct Acc<'db> {
    foreign_items: ThinVec<ForeignItem>,
    struct_items: ThinVec<Box<Item<'db>>>,
    imports: Vec<ResolvedImport<'db>>,
    fn_symbols: HashMap<String, String>,
    rust_vis: Vec<RustTypeVis>,
    rust_layouts: HashMap<String, RustLayout>,
    rust_fn_abis: HashMap<String, scrap_rmeta::FnAbiInfo>,
    /// Full path → Scrap-visible local name, for idempotent type imports.
    imported_types: HashMap<String, String>,
    /// Droppable imported types: `(local name, full path)`.
    droppable: Vec<(String, String)>,
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

/// Whether a Rust type display name is a scalar primitive we can mirror as a
/// loadable field today (non-scalar fields are out of scope for this slice).
fn is_scalar(display: &str) -> bool {
    matches!(
        display,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "usize"
            | "f32"
            | "f64"
            | "bool"
    )
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
    type_catalog: &HashMap<String, &RustType>,
) -> ResolveOutcome<'db> {
    let mut acc = Acc::default();

    for u in use_refs {
        if let Some(cat) = catalog.get(&u.path) {
            if acc.fn_symbols.contains_key(&u.local) {
                emit_dup(db, &u.local);
                continue;
            }
            resolve_fn_import(db, u, cat, type_catalog, &mut acc);
        } else if let Some(ty) = type_catalog.get(&u.path) {
            // Explicit `use rust::…::Type [as alias];` — alias is the local name.
            ensure_type_imported(db, &u.path, &u.local, ty, &mut acc);
        } else {
            emit_err(
                db,
                format!("no Rust item `{}`", u.path),
                "not found in the interop catalog (or it is generic — not yet supported)",
            );
        }
    }

    // Synthesized struct types must precede the `extern "Rust"` block: tycheck's
    // first pass collects signatures in order, so a foreign fn that references a
    // Rust struct needs that struct already registered.
    let mut ast_items = ThinVec::new();
    ast_items.extend(std::mem::take(&mut acc.struct_items));
    if !acc.foreign_items.is_empty() {
        let foreign_mod = ForeignMod {
            abi: Symbol::new("Rust"),
            items: std::mem::take(&mut acc.foreign_items),
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
        imports: acc.imports,
        fn_symbols: acc.fn_symbols,
        rust_vis: acc.rust_vis,
        rust_layouts: acc.rust_layouts,
        rust_fn_abis: acc.rust_fn_abis,
        droppable: acc.droppable,
    }
}

/// Sanitize a full Rust type path into a unique identifier suffix for the
/// generated drop-wrapper fn name (e.g. `a::B<c>` → `a__B_c_`).
pub fn sanitize_path(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

fn emit_dup(db: &dyn scrap_shared::Db, local: &str) {
    emit_err(
        db,
        format!("`{}` is imported more than once", local),
        "import one with `use rust::… as other_name;`",
    );
}

/// Map one fn param/return Rust type display to an `(ir::Ty, AST type)` pair,
/// auto-importing referenced struct types as needed. Returns `None` (with a
/// diagnostic) for an unsupported type.
fn resolve_sig_ty<'db>(
    db: &'db dyn scrap_shared::Db,
    display: &str,
    type_catalog: &HashMap<String, &RustType>,
    acc: &mut Acc<'db>,
) -> Option<(ir::Ty<'db>, AstTy)> {
    if let Some(ty) = prim_ir(display) {
        return Some((ty, prim_ast_ty(display)));
    }
    // A non-scalar type: it must be a struct in the catalog (auto-import it).
    let Some(rust_ty) = type_catalog.get(display) else {
        emit_err(
            db,
            format!("`{display}` is not a supported interop type"),
            "expected a scalar primitive or a Rust struct in the interop catalog",
        );
        return None;
    };
    let local = ensure_type_imported(db, display, last_segment(display), rust_ty, acc)?;
    let type_id = ir::TypeId::new(db, local.clone());
    Some((ir::Ty::Rust(type_id), prim_ast_ty(&local)))
}

/// Resolve a `use rust::…::fn;` import: map its signature (auto-importing any
/// referenced struct types) and record the foreign fn, IR import, mangled
/// symbol, and ABI.
fn resolve_fn_import<'db>(
    db: &'db dyn scrap_shared::Db,
    u: &RustUseRef,
    cat: &CatalogFn,
    type_catalog: &HashMap<String, &RustType>,
    acc: &mut Acc<'db>,
) {
    let mut param_ir = Vec::with_capacity(cat.params.len());
    let mut param_ast = ThinVec::new();
    for (i, p) in cat.params.iter().enumerate() {
        let Some((ty, ast_ty)) = resolve_sig_ty(db, p, type_catalog, acc) else {
            return;
        };
        param_ir.push(ty);
        param_ast.push(Param {
            id: NodeId::dummy(),
            ident: Ident::dummy_with_name(&format!("arg{i}")),
            ty: Box::new(ast_ty),
            pat: dummy_pat(),
            span: Span::default(),
        });
    }
    let (ret_ir, ret_ast) = if cat.ret == "()" {
        (ir::Ty::Void, None)
    } else {
        let Some((ty, ast_ty)) = resolve_sig_ty(db, &cat.ret, type_catalog, acc) else {
            return;
        };
        (ty, Some(ast_ty))
    };

    acc.foreign_items.push(ForeignItem {
        id: NodeId::dummy(),
        ident: Ident::dummy_with_name(&u.local),
        args: param_ast,
        ret_type: ret_ast,
        span: Span::default(),
    });
    acc.imports.push(ResolvedImport {
        local: u.local.clone(),
        params: param_ir,
        ret: ret_ir,
    });
    acc.fn_symbols.insert(u.local.clone(), cat.symbol.clone());
    acc.rust_fn_abis.insert(u.local.clone(), cat.abi.clone());
}

/// The last `::`-separated segment of a path (`a::b::C` → `C`).
fn last_segment(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path)
}

/// Ensure a Rust struct type is imported under `local`: synthesize its Scrap
/// `struct` (for tycheck), record its mirrored layout (codegen) and visibility
/// facts (construction gating). Idempotent per full `path`. Returns the
/// Scrap-visible local name, or `None` (with a diagnostic) for an unsupported
/// shape (non-struct, non-scalar fields) or a name conflict.
fn ensure_type_imported<'db>(
    db: &'db dyn scrap_shared::Db,
    path: &str,
    local: &str,
    ty: &RustType,
    acc: &mut Acc<'db>,
) -> Option<String> {
    use scrap_rmeta::AdtKind;

    if let Some(existing) = acc.imported_types.get(path) {
        return Some(existing.clone());
    }
    // The desired local name must not already bind a different type or a fn.
    if acc.imported_types.values().any(|l| l == local) || acc.fn_symbols.contains_key(local) {
        emit_err(
            db,
            format!("`{local}` is imported more than once"),
            "import one with `use rust::… as other_name;`",
        );
        return None;
    }

    if ty.kind != AdtKind::Struct {
        emit_err(
            db,
            format!("`{path}` is not a struct"),
            "only struct types are supported so far (enums/unions are not yet)",
        );
        return None;
    }
    let layout = ty
        .layout
        .as_ref()
        .expect("type catalog only holds types with a layout");

    // Non-scalar fields are imported as opaque memory (offset only): the type can
    // be passed/returned by value, but such fields can't be read or constructed
    // from Scrap (gated in tycheck via the `scalar` flag below).
    let mut fields_ast = ThinVec::new();
    let mut vis_fields = Vec::with_capacity(ty.fields.len());
    let mut offsets_displays: Vec<(u64, String)> = Vec::with_capacity(ty.fields.len());
    for (i, f) in ty.fields.iter().enumerate() {
        let offset = layout.field_offsets.get(i).copied().unwrap_or(0);
        let scalar = is_scalar(&f.ty.display);
        offsets_displays.push((offset, f.ty.display.clone()));
        vis_fields.push(RustFieldVis {
            name: f.name.clone(),
            public: f.public,
            scalar,
        });
        // Scalar fields keep their real type; opaque fields get a pointer-width
        // placeholder (never read/constructed, only present so indices line up).
        let field_ast_ty = if scalar {
            prim_ast_ty(&f.ty.display)
        } else {
            prim_ast_ty("usize")
        };
        fields_ast.push(FieldDef {
            id: NodeId::dummy(),
            span: Span::default(),
            vis: Visibility {
                kind: VisibilityKind::Public,
                span: Span::default(),
            },
            ident: Some(Ident::dummy_with_name(&f.name)),
            ty: Box::new(field_ast_ty),
        });
    }

    let struct_def = StructDef {
        id: NodeId::dummy(),
        ident: Ident::dummy_with_name(local),
        generics: Generics::default(),
        data: VariantData::Struct { fields: fields_ast },
    };
    acc.struct_items.push(Box::new(Item {
        kind: ItemKind::Struct(struct_def),
        span: Span::default(),
        id: NodeId::dummy(),
        vis: Visibility {
            kind: VisibilityKind::Inherited,
            span: Span::default(),
        },
    }));

    let offset_refs: Vec<(u64, &str)> = offsets_displays
        .iter()
        .map(|(o, d)| (*o, d.as_str()))
        .collect();
    acc.rust_layouts.insert(
        local.to_string(),
        scrap_codegen::rust_layout_from_metadata(layout.size, layout.align, &offset_refs),
    );
    acc.rust_vis.push(RustTypeVis {
        name: local.to_string(),
        fields: vis_fields,
        non_exhaustive: ty.non_exhaustive,
    });
    if layout.needs_drop {
        acc.droppable.push((local.to_string(), path.to_string()));
    }
    acc.imported_types
        .insert(path.to_string(), local.to_string());

    // Auto-synthesize a callable extern per supported inherent method, named
    // `L::method` so the existing method-call path (`recv.method(args)`) resolves
    // it. The receiver is `params[0]` (its `&self`/`&mut self`/`self` ABI is
    // applied at codegen). Methods with non-scalar/non-imported signatures are
    // skipped (calling one is then a normal "undefined function" error).
    synth_methods(db, local, ty, acc);

    Some(local.to_string())
}

/// Map a method param/return display to `(ir::Ty, AST type)` using only scalars
/// and already-imported Rust types (no auto-import / no diagnostics — an
/// unsupported method is silently skipped).
fn method_sig_ty<'db>(
    db: &'db dyn scrap_shared::Db,
    display: &str,
    acc: &Acc<'db>,
) -> Option<(ir::Ty<'db>, AstTy)> {
    if let Some(ty) = prim_ir(display) {
        return Some((ty, prim_ast_ty(display)));
    }
    if let Some(l) = acc.imported_types.get(display) {
        return Some((ir::Ty::Rust(ir::TypeId::new(db, l.clone())), prim_ast_ty(l)));
    }
    None
}

fn synth_methods<'db>(
    db: &'db dyn scrap_shared::Db,
    local: &str,
    ty: &RustType,
    acc: &mut Acc<'db>,
) {
    for m in &ty.methods {
        let Some(mono) = &m.mono else { continue }; // generic → no symbol
        let mname = m.path.rsplit("::").next().unwrap_or(m.path.as_str());
        // `L::name`, callable as `recv.name(..)` (method) or `L::name(..)` (assoc fn).
        let fn_local = format!("{local}::{mname}");
        if acc.fn_symbols.contains_key(&fn_local) {
            continue; // already synthesized (idempotent re-import)
        }

        // A `self`-method's `params[0]` is the receiver (mapped to this type; its
        // &/&mut-ness lives in the ABI). An associated fn has no receiver.
        let mut param_ir = Vec::new();
        let mut param_ast = ThinVec::new();
        let rest = if m.has_self {
            if m.params.is_empty() {
                continue;
            }
            param_ir.push(ir::Ty::Rust(ir::TypeId::new(db, local.to_string())));
            param_ast.push(Param {
                id: NodeId::dummy(),
                ident: Ident::dummy_with_name("self"),
                ty: Box::new(prim_ast_ty(local)),
                pat: dummy_pat(),
                span: Span::default(),
            });
            &m.params[1..]
        } else {
            &m.params[..]
        };

        let mut supported = true;
        for (i, p) in rest.iter().enumerate() {
            match method_sig_ty(db, &p.display, acc) {
                Some((t, ast_ty)) => {
                    param_ir.push(t);
                    param_ast.push(Param {
                        id: NodeId::dummy(),
                        ident: Ident::dummy_with_name(&format!("arg{i}")),
                        ty: Box::new(ast_ty),
                        pat: dummy_pat(),
                        span: Span::default(),
                    });
                }
                None => {
                    supported = false;
                    break;
                }
            }
        }
        if !supported {
            continue;
        }

        let (ret_ir, ret_ast) = if m.ret.display == "()" {
            (ir::Ty::Void, None)
        } else {
            match method_sig_ty(db, &m.ret.display, acc) {
                Some((t, a)) => (t, Some(a)),
                None => continue,
            }
        };

        acc.foreign_items.push(ForeignItem {
            id: NodeId::dummy(),
            ident: Ident::dummy_with_name(&fn_local),
            args: param_ast,
            ret_type: ret_ast,
            span: Span::default(),
        });
        acc.imports.push(ResolvedImport {
            local: fn_local.clone(),
            params: param_ir,
            ret: ret_ir,
        });
        acc.fn_symbols.insert(fn_local.clone(), mono.symbol.clone());
        acc.rust_fn_abis.insert(fn_local, mono.abi.clone());
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
    let module_id =
        scrap_shared::id::ModuleId::from_path(db, &Path::from_segment("__rust_interop"));
    ir::Module::new(db, module_id, items)
}
