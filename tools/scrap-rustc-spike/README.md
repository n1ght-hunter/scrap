# scrap-rustc-spike

Phase 0 feasibility spike for the [Rust-interop plan](../../.claude/plans/rust-interop.md).
Proves that a `rustc_private` driver, built against the pinned toolchain, can extract the three
things the interop design depends on: **mangled symbol names**, **`FnAbi`** (call ABI), and exact
**type layouts**.

**Status: GO.** All three work against `nightly-2026-02-10`, for non-generic items, **generic
fn instantiations**, and **cross-crate std generics** (`Vec<i32>`, `Option<i32>`).

## Run

```powershell
$sysroot = (rustc --print sysroot).Trim()
$env:PATH = (Join-Path $sysroot "bin") + ";" + $env:PATH   # rustc_driver.dll at runtime
cargo run -- sample.rs --crate-type lib --edition 2024
```

The driver auto-injects `--sysroot` if absent (a rustc-private tool can't locate it relative to its
own exe). It's a detached workspace (own `[workspace]` + `rust-toolchain.toml`) so the heavy
`rustc-dev` component and the rustc-private link step don't burden the main scrap build.

## Findings (validate the design)

- **v0 symbols** out of the box: `_RNvCs..._6sample10scalar_add`.
- **Rust ABI ≠ C ABI**, as expected — must replicate `FnAbi`, not assume C. All three pass modes
  the Phase 5 cg_clif port must handle show up:
  - `make_point(..) -> Point` (16-byte repr(Rust)) → return mode **`Pair`** (two registers), not sret.
  - `point_sum(Point)` → arg mode **`Pair`** (by value in registers), not by pointer.
  - `make_vec() -> Vec<i32>` (24 bytes) → return mode **`Indirect`** (hidden sret pointer).
  - `point_ref(&Point)` → `Direct` `NonNull` pointer.
- **repr(Rust) reorders fields**: `Point { x: i32, y: i64 }` → offsets `[8, 0]` (y first). `CPoint`
  (repr(C)) → `[0, 8]`. Offsets must come from the compiler; never assume declaration order.
- Enum (`Shape`) yields tag (`Int(I64)`, `Direct` encoding) + full per-variant layouts.
- **Generic instantiation works**: `identity::<i32>` → monomorphized v0 symbol + ABI;
  `Vec<i32>`/`Option<i32>` layouts resolve cross-crate.
- **Defaulted generic params must be filled.** `Vec<T, A = Global>` panics if you pass only `[i32]`
  (`type parameter A/#1 out of range`). Use `GenericArgs::for_item` and supply each param's default
  for the ones the user didn't write — see `args_first_type_i32` in `src/main.rs`.

## Key API notes (nightly-2026-02-10, churns across versions)

- `Callbacks::after_analysis(&mut self, &Compiler, tcx: TyCtxt) -> Compilation` (tcx passed directly).
- `rustc_driver::run_compiler(&args, &mut callbacks)` inside `catch_fatal_errors`.
- Free items: `tcx.hir_crate_items(()).free_items()` → `ItemId`; `id.owner_id.to_def_id()`.
- `tcx.symbol_name(Instance::mono(tcx, def_id))`.
- `tcx.layout_of(typing_env.as_query_input(ty))` where `typing_env = TypingEnv::fully_monomorphized()`.
- `tcx.fn_abi_of_instance(typing_env.as_query_input((instance, ty::List::empty())))` — note the
  **`PseudoCanonicalInput` wrapper** (`as_query_input`); a plain tuple does not compile.
