//! Walk the requested dependency crates' public API from the anchor's `TyCtxt`
//! and lower each item into the `scrap_rmeta` schema.

use std::collections::HashSet;

use rustc_abi::{BackendRepr, Float, Integer, Primitive};
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, CRATE_DEF_INDEX};
use rustc_middle::ty::{self, Instance, Ty, TyCtxt, TypingEnv};
use rustc_target::callconv::{ArgAbi, FnAbi, PassMode as RPassMode};

use scrap_rmeta::{
    AdtKind, ArgAbi as SArgAbi, FnAbiInfo, LayoutInfo, MonoFn, PassMode, RustCrate, RustField,
    RustFn, RustMetadata, RustTyRef, RustType, RustVariant, SCHEMA_VERSION, Scalar as SScalar,
};

/// Build the full metadata dump for the `want_crates` set.
pub fn extract(tcx: TyCtxt<'_>, want_crates: &[String], target: String) -> RustMetadata {
    let mut crates = Vec::new();
    for &cnum in tcx.crates(()) {
        let name = tcx.crate_name(cnum).to_string();
        if !want_crates.iter().any(|w| *w == name) {
            continue;
        }
        let root = DefId {
            krate: cnum,
            index: CRATE_DEF_INDEX,
        };
        let mut fns = Vec::new();
        let mut types = Vec::new();
        let mut visited = HashSet::new();
        walk_module(tcx, root, cnum, &mut visited, &mut fns, &mut types);
        crates.push(RustCrate { name, fns, types });
    }
    RustMetadata {
        schema_version: SCHEMA_VERSION,
        target,
        crates,
    }
}

fn walk_module(
    tcx: TyCtxt<'_>,
    module: DefId,
    home: rustc_hir::def_id::CrateNum,
    visited: &mut HashSet<DefId>,
    fns: &mut Vec<RustFn>,
    types: &mut Vec<RustType>,
) {
    for child in tcx.module_children(module) {
        if !child.vis.is_public() {
            continue;
        }
        let Res::Def(kind, def_id) = child.res else {
            continue;
        };
        // Stay within the target crate (re-exports may point elsewhere).
        if def_id.krate != home || !visited.insert(def_id) {
            continue;
        }
        match kind {
            DefKind::Mod => walk_module(tcx, def_id, home, visited, fns, types),
            DefKind::Fn => fns.push(extract_fn(tcx, def_id)),
            DefKind::Struct | DefKind::Enum | DefKind::Union => {
                types.push(extract_adt(tcx, def_id))
            }
            _ => {}
        }
    }
}

fn type_param_names(tcx: TyCtxt<'_>, def_id: DefId) -> Vec<String> {
    tcx.generics_of(def_id)
        .own_params
        .iter()
        .filter(|p| matches!(p.kind, ty::GenericParamDefKind::Type { .. }))
        .map(|p| p.name.to_string())
        .collect()
}

/// Whether `def_id` has type or const generic params (lifetimes don't block
/// monomorphization).
fn is_generic(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    tcx.generics_of(def_id).own_params.iter().any(|p| {
        matches!(
            p.kind,
            ty::GenericParamDefKind::Type { .. } | ty::GenericParamDefKind::Const { .. }
        )
    })
}

fn extract_fn(tcx: TyCtxt<'_>, def_id: DefId) -> RustFn {
    let path = tcx.def_path_str(def_id);
    let generic_params = type_param_names(tcx, def_id);

    let sig = tcx.fn_sig(def_id).instantiate_identity().skip_binder();
    let params = sig.inputs().iter().map(|t| ty_ref(tcx, *t)).collect();
    let ret = ty_ref(tcx, sig.output());

    let mono = if is_generic(tcx, def_id) {
        None
    } else {
        let instance = Instance::mono(tcx, def_id);
        let symbol = tcx.symbol_name(instance).to_string();
        let typing_env = TypingEnv::fully_monomorphized();
        tcx.fn_abi_of_instance(typing_env.as_query_input((instance, ty::List::empty())))
            .ok()
            .map(|abi| MonoFn {
                symbol,
                abi: fn_abi_info(abi),
            })
    };

    RustFn {
        path,
        generic_params,
        params,
        ret,
        mono,
    }
}

/// Build a type reference, using the canonical `def_path_str` for ADTs so it
/// matches the type-catalog key scrapc resolves `use rust::…::Type;` against.
fn ty_ref(tcx: TyCtxt<'_>, t: Ty<'_>) -> RustTyRef {
    let display = match t.kind() {
        ty::TyKind::Adt(adt, _) => tcx.def_path_str(adt.did()),
        _ => t.to_string(),
    };
    RustTyRef { display }
}

fn fn_abi_info(fn_abi: &FnAbi<'_, Ty<'_>>) -> FnAbiInfo {
    FnAbiInfo {
        conv: format!("{:?}", fn_abi.conv),
        args: fn_abi.args.iter().map(arg_abi).collect(),
        ret: arg_abi(&fn_abi.ret),
    }
}

fn arg_abi(a: &ArgAbi<'_, Ty<'_>>) -> SArgAbi {
    SArgAbi {
        ty: RustTyRef {
            display: a.layout.ty.to_string(),
        },
        mode: pass_mode(a),
    }
}

/// Map a rustc `Scalar` to a Cranelift-mappable scalar kind. Unsupported widths
/// (i128/f16/f128) fall back to the nearest 64-bit kind.
fn map_scalar(s: rustc_abi::Scalar) -> SScalar {
    match s.primitive() {
        Primitive::Int(i, _) => match i {
            Integer::I8 => SScalar::I8,
            Integer::I16 => SScalar::I16,
            Integer::I32 => SScalar::I32,
            Integer::I64 | Integer::I128 => SScalar::I64,
        },
        Primitive::Float(f) => match f {
            Float::F16 | Float::F32 => SScalar::F32,
            Float::F64 | Float::F128 => SScalar::F64,
        },
        Primitive::Pointer(_) => SScalar::Ptr,
    }
}

fn pass_mode(a: &ArgAbi<'_, Ty<'_>>) -> PassMode {
    match &a.mode {
        RPassMode::Ignore => PassMode::Ignore,
        RPassMode::Direct(_) => match a.layout.backend_repr {
            BackendRepr::Scalar(s) => PassMode::Direct(map_scalar(s)),
            _ => PassMode::Direct(SScalar::Ptr),
        },
        RPassMode::Pair(_, _) => match a.layout.backend_repr {
            BackendRepr::ScalarPair(s0, s1) => PassMode::Pair(map_scalar(s0), map_scalar(s1)),
            _ => PassMode::Pair(SScalar::Ptr, SScalar::Ptr),
        },
        RPassMode::Cast { .. } => PassMode::Cast,
        RPassMode::Indirect { on_stack, .. } => PassMode::Indirect {
            on_stack: *on_stack,
        },
    }
}

fn extract_adt(tcx: TyCtxt<'_>, def_id: DefId) -> RustType {
    let path = tcx.def_path_str(def_id);
    let adt = tcx.adt_def(def_id);

    let kind = if adt.is_enum() {
        AdtKind::Enum
    } else if adt.is_union() {
        AdtKind::Union
    } else {
        AdtKind::Struct
    };

    let repr = repr_string(adt.repr());
    let generic_params = type_param_names(tcx, def_id);

    let mut fields = Vec::new();
    let mut variants = Vec::new();
    if adt.is_enum() {
        for v in adt.variants() {
            variants.push(RustVariant {
                name: v.name.to_string(),
                fields: v.fields.iter().map(|f| field(tcx, f)).collect(),
            });
        }
    } else {
        fields = adt
            .non_enum_variant()
            .fields
            .iter()
            .map(|f| field(tcx, f))
            .collect();
    }

    let layout = if is_generic(tcx, def_id) {
        None
    } else {
        adt_layout(tcx, def_id)
    };

    let non_exhaustive = if adt.is_enum() {
        adt.is_variant_list_non_exhaustive()
    } else {
        adt.non_enum_variant().is_field_list_non_exhaustive()
    };

    RustType {
        path,
        kind,
        repr,
        generic_params,
        fields,
        variants,
        non_exhaustive,
        layout,
    }
}

fn field(tcx: TyCtxt<'_>, f: &ty::FieldDef) -> RustField {
    RustField {
        name: f.name.to_string(),
        public: f.vis.is_public(),
        ty: RustTyRef {
            display: tcx.type_of(f.did).instantiate_identity().to_string(),
        },
    }
}

fn repr_string(r: rustc_abi::ReprOptions) -> String {
    if r.c() {
        "C".to_string()
    } else if r.transparent() {
        "transparent".to_string()
    } else {
        "Rust".to_string()
    }
}

fn adt_layout(tcx: TyCtxt<'_>, def_id: DefId) -> Option<LayoutInfo> {
    let ty = tcx.type_of(def_id).instantiate_identity();
    let typing_env = TypingEnv::fully_monomorphized();
    let layout = tcx.layout_of(typing_env.as_query_input(ty)).ok()?;

    let field_offsets = match &layout.fields {
        rustc_abi::FieldsShape::Arbitrary { offsets, .. } => {
            offsets.iter().map(|o| o.bytes()).collect()
        }
        _ => Vec::new(),
    };

    Some(LayoutInfo {
        size: layout.size.bytes(),
        align: layout.align.abi.bytes(),
        field_offsets,
        is_copy: tcx.type_is_copy_modulo_regions(typing_env, ty),
    })
}
