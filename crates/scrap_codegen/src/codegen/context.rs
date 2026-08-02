//! CodegenContext — holds the Cranelift module and compilation state.

use cranelift::codegen::binemit::CodeOffset;
use cranelift::codegen::isa::unwind::UnwindInfo;
use cranelift::prelude::*;
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use scrap_ir as ir;
use std::collections::HashMap;
use target_lexicon::Triple;

use super::ResultExt;
use super::emit_codegen_err;

/// Per-function unwind metadata collected during compilation.
pub(crate) struct UnwindEntry {
    pub func_id: FuncId,
    pub code_size: u32,
    pub unwind_bytes: Vec<u8>,
}

/// Mirrored in-memory layout of a native Rust interop type, keyed by its
/// fully-qualified path. Sourced from interop metadata: codegen places a value
/// of this type in a stack slot of this size/align and reads/writes its fields
/// at these byte offsets (rather than decomposing it into SSA variables).
#[derive(Debug, Clone)]
pub struct RustLayout {
    pub size: u32,
    pub align: u32,
    pub fields: Vec<RustFieldLayout>,
}

/// One field of a [`RustLayout`].
#[derive(Debug, Clone)]
pub struct RustFieldLayout {
    /// Byte offset of the field within the value.
    pub offset: u32,
    /// Cranelift type of the field when it is a scalar, or `None` when the field
    /// is itself an aggregate (addressed by `base + offset`).
    pub cl_ty: Option<types::Type>,
}

impl RustLayout {
    /// Stack-slot alignment shift (log2 of the byte alignment) Cranelift's
    /// `StackSlotData` expects.
    pub fn align_shift(&self) -> u8 {
        self.align.max(1).trailing_zeros() as u8
    }
}

/// The main code generation context.
pub struct CodegenContext<'db> {
    pub(crate) db: &'db dyn scrap_shared::Db,
    pub(crate) module: ObjectModule,
    pub(crate) ctx: codegen::Context,
    pub(crate) func_ctx: FunctionBuilderContext,
    /// Maps function name → Cranelift FuncId.
    pub(crate) functions: HashMap<String, FuncId>,
    /// Collected unwind info for each compiled function.
    pub(crate) unwind_entries: Vec<UnwindEntry>,
    /// GcShape data sections: type descriptor key → DataId.
    pub(crate) gc_shapes: HashMap<String, DataId>,
    /// Struct layout: struct name → list of field IR types.
    pub(crate) struct_layouts: HashMap<String, Vec<ir::Ty<'db>>>,
    /// Enum layout: enum name → per-variant field types (Vec of variants, each a Vec of field types).
    pub(crate) enum_layouts: HashMap<String, Vec<Vec<ir::Ty<'db>>>>,
    /// Mirrored layouts of native Rust interop types, by fully-qualified path.
    /// Populated from interop metadata before codegen (Phase 4); drives the
    /// memory-backed handling of `ir::Ty::Rust` locals.
    pub(crate) rust_layouts: HashMap<String, RustLayout>,
    /// Native Rust interop functions: declared `extern "Rust"` name → the real
    /// v0-mangled symbol (from interop metadata). An `extern` import whose name
    /// is in this map is linked against the mangled symbol instead of its name.
    pub(crate) rust_fn_symbols: HashMap<String, String>,
    /// Native Rust interop functions: declared `extern "Rust"` name → its
    /// per-arg/return ABI (from interop metadata). When present, the Cranelift
    /// call signature + arg/return marshalling are built from this `FnAbiInfo`
    /// rather than the IR types (Phase 5).
    pub(crate) rust_fn_abis: HashMap<String, scrap_rmeta::FnAbiInfo>,
    /// Native Rust interop types that need dropping: Scrap type name → the
    /// mangled symbol of the anchor's drop wrapper (`drop_in_place::<T>` glue).
    /// A `Ty::Rust` local of such a type gets RAII drop at scope exit.
    pub(crate) rust_drop_syms: HashMap<String, String>,
    /// Monotonically increasing counter for data section names (persists across functions).
    pub(crate) data_id_counter: usize,
    /// Collected stack map entries across all compiled functions.
    /// Each entry: (func_id, code_offset, vec of SP-relative offsets for GC roots).
    pub(crate) stack_map_entries: Vec<(FuncId, CodeOffset, Vec<u32>)>,
}

impl<'db> CodegenContext<'db> {
    /// Create a new code generation context for the given target triple.
    ///
    /// When `target` is the host triple, the host ISA builder is used so that
    /// native CPU features are enabled; otherwise a baseline ISA is looked up
    /// for the requested target (cross-compilation). The object format
    /// (COFF/ELF/Mach-O) is derived from the triple automatically.
    pub fn new(db: &'db dyn scrap_shared::Db, target: &Triple) -> Option<Self> {
        let mut shared_builder = settings::builder();
        shared_builder
            .set("preserve_frame_pointers", "true")
            .unwrap();
        let shared_flags = settings::Flags::new(shared_builder);

        let isa_builder = if *target == Triple::host() {
            cranelift_native::builder()
                .map_err(|e| format!("host ISA builder failed: {e}"))
                .or_emit(db)?
        } else {
            cranelift::codegen::isa::lookup(target.clone())
                .map_err(|e| format!("ISA lookup failed: {e}"))
                .or_emit(db)?
        };
        let isa = isa_builder
            .finish(shared_flags)
            .map_err(|e| format!("ISA finish failed: {e}"))
            .or_emit(db)?;

        let object_builder = ObjectBuilder::new(
            isa,
            "scrap_program",
            cranelift_module::default_libcall_names(),
        )
        .map_err(|e| format!("ObjectBuilder failed: {e}"))
        .or_emit(db)?;

        let module = ObjectModule::new(object_builder);

        Some(Self {
            db,
            module,
            ctx: codegen::Context::new(),
            func_ctx: FunctionBuilderContext::new(),
            functions: HashMap::new(),
            unwind_entries: Vec::new(),
            gc_shapes: HashMap::new(),
            struct_layouts: HashMap::new(),
            enum_layouts: HashMap::new(),
            rust_layouts: HashMap::new(),
            rust_fn_symbols: HashMap::new(),
            rust_fn_abis: HashMap::new(),
            rust_drop_syms: HashMap::new(),
            data_id_counter: 0,
            stack_map_entries: Vec::new(),
        })
    }

    /// Install the mirrored Rust interop layouts (from interop metadata) used to
    /// codegen `ir::Ty::Rust` locals. Must be called before `compile_module`.
    pub fn set_rust_layouts(&mut self, layouts: HashMap<String, RustLayout>) {
        self.rust_layouts = layouts;
    }

    /// Install the `extern "Rust"` name → mangled-symbol map (from interop
    /// metadata). Must be called before `compile_module`.
    pub fn set_rust_fn_symbols(&mut self, symbols: HashMap<String, String>) {
        self.rust_fn_symbols = symbols;
    }

    /// Install the `extern "Rust"` name → `FnAbiInfo` map (from interop
    /// metadata). Must be called before `compile_module`.
    pub fn set_rust_fn_abis(&mut self, abis: HashMap<String, scrap_rmeta::FnAbiInfo>) {
        self.rust_fn_abis = abis;
    }

    /// Install the droppable-type name → drop-wrapper-symbol map (from interop
    /// metadata). Must be called before `compile_module`.
    pub fn set_rust_drop_syms(&mut self, syms: HashMap<String, String>) {
        self.rust_drop_syms = syms;
    }

    /// Compile an entire IR module (declare then define).
    pub fn compile_module(&mut self, module: ir::Module<'db>) -> Option<()> {
        self.declare_items(module)?;
        self.declare_panic_runtime()?;
        self.declare_gc_runtime()?;
        self.declare_spawn_runtime()?;
        self.define_functions(module)?;
        Some(())
    }

    /// Whether the target goes through libc's startup (`crt` → `__libc_start_main`
    /// → `main`). True for ELF/Mach-O; false for COFF/PE, where we emit a custom
    /// `_start` and bypass the CRT. The runtime is Rust std, which relies on the
    /// libc/TLS init that crt startup performs, so on ELF/Mach-O we must not
    /// bypass it.
    pub(crate) fn uses_libc_startup(&self) -> bool {
        !matches!(
            self.module.isa().triple().binary_format,
            target_lexicon::BinaryFormat::Coff
        )
    }

    /// Symbol name the user's `main` is emitted under. On libc-startup targets it
    /// is renamed so our generated entry can claim the `main` symbol that crt
    /// calls; elsewhere it keeps its name.
    pub(crate) fn user_main_symbol(&self) -> &'static str {
        if self.uses_libc_startup() {
            "__scrap_user_main"
        } else {
            "main"
        }
    }

    /// Generate the program entry point that initializes the runtime and calls
    /// the user's `main`. On COFF this is a custom `_start` (CRT bypassed); on
    /// ELF/Mach-O it is `main`, invoked by the platform's crt startup.
    pub fn generate_start(&mut self) -> Option<()> {
        let frontend_config = self.module.target_config();
        let main_func_id = match self.functions.get("main").copied() {
            Some(id) => id,
            None => {
                emit_codegen_err(self.db, "function 'main' not found");
                return None;
            }
        };

        let entry_name = if self.uses_libc_startup() {
            "main"
        } else {
            "_start"
        };

        // Entry: no params, no returns (it diverges via __scrap_exit).
        let mut start_sig = self.module.make_signature();
        start_sig.call_conv = self.module.target_config().default_call_conv;

        let start_func_id = self
            .module
            .declare_function(entry_name, Linkage::Export, &start_sig)
            .or_emit(self.db)?;

        self.ctx.func.signature = start_sig;

        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.func_ctx);
            let entry_block = builder.create_block();
            builder.switch_to_block(entry_block);

            // Call __scrap_gc_init before main
            if let Some(&gc_init_id) = self.functions.get("__scrap_gc_init") {
                let gc_init_ref = self.module.declare_func_in_func(gc_init_id, builder.func);
                builder.ins().call(gc_init_ref, &[]);
            }

            // Call __scrap_sched_init after gc_init
            if let Some(&sched_init_id) = self.functions.get("__scrap_sched_init") {
                let sched_init_ref = self
                    .module
                    .declare_func_in_func(sched_init_id, builder.func);
                builder.ins().call(sched_init_ref, &[]);
            }

            // Call main
            let main_ref = self.module.declare_func_in_func(main_func_id, builder.func);
            builder.ins().call(main_ref, &[]);

            // Call __scrap_sched_shutdown after main (runs remaining coroutines)
            if let Some(&sched_shutdown_id) = self.functions.get("__scrap_sched_shutdown") {
                let sched_shutdown_ref = self
                    .module
                    .declare_func_in_func(sched_shutdown_id, builder.func);
                builder.ins().call(sched_shutdown_ref, &[]);
            }

            // Call __scrap_exit(0) for a clean exit after main + scheduler finish.
            // Programs that need a specific exit code call __scrap_exit explicitly.
            if let Some(&exit_id) = self.functions.get("__scrap_exit") {
                let exit_ref = self.module.declare_func_in_func(exit_id, builder.func);
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().call(exit_ref, &[zero]);
            }

            // Fallback trap (unreachable — __scrap_exit diverges)
            builder.ins().trap(TrapCode::user(1).unwrap());

            builder.seal_all_blocks();
            builder.finalize(frontend_config);
        }

        self.module
            .define_function(start_func_id, &mut self.ctx)
            .or_emit(self.db)?;

        self.collect_unwind_info(start_func_id);
        self.module.clear_context(&mut self.ctx);

        Some(())
    }

    /// Declare the panic/exit runtime imported from `scrap_rt`: `__scrap_panic`
    /// and `__scrap_exit`. Both are platform-agnostic (the runtime uses Rust std),
    /// so codegen emits no OS-specific calls. Must be called before
    /// `define_functions()` so user code can reference `__scrap_panic`.
    pub fn declare_panic_runtime(&mut self) -> Option<()> {
        let ptr_ty = types::I64;
        let call_conv = self.module.target_config().default_call_conv;

        // __scrap_panic(msg_ptr: i64, msg_len: i64) -> !
        if !self.functions.contains_key("__scrap_panic") {
            let mut panic_sig = self.module.make_signature();
            panic_sig.call_conv = call_conv;
            panic_sig.params.push(AbiParam::new(ptr_ty)); // msg_ptr
            panic_sig.params.push(AbiParam::new(ptr_ty)); // msg_len
            let panic_func_id = self
                .module
                .declare_function("__scrap_panic", Linkage::Import, &panic_sig)
                .or_emit(self.db)?;
            self.functions
                .insert("__scrap_panic".to_string(), panic_func_id);
        }

        // __scrap_exit(exit_code: i64) -> !
        if !self.functions.contains_key("__scrap_exit") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            sig.params.push(AbiParam::new(ptr_ty));
            let fid = self
                .module
                .declare_function("__scrap_exit", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions.insert("__scrap_exit".to_string(), fid);
        }

        Some(())
    }

    /// Declare the GC runtime functions (imported from scrap_rt.lib).
    pub fn declare_gc_runtime(&mut self) -> Option<()> {
        let ptr_ty = types::I64;
        let call_conv = self.module.target_config().default_call_conv;

        // __scrap_gc_init()
        if !self.functions.contains_key("__scrap_gc_init") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            let fid = self
                .module
                .declare_function("__scrap_gc_init", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions.insert("__scrap_gc_init".to_string(), fid);
        }

        // __scrap_gc_alloc(shape: *const GcShape) -> *mut u8
        if !self.functions.contains_key("__scrap_gc_alloc") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            sig.params.push(AbiParam::new(ptr_ty)); // shape
            sig.returns.push(AbiParam::new(ptr_ty)); // pointer
            let fid = self
                .module
                .declare_function("__scrap_gc_alloc", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions.insert("__scrap_gc_alloc".to_string(), fid);
        }

        Some(())
    }

    /// Declare the spawn/coroutine runtime functions (imported from scrap_rt.lib).
    pub fn declare_spawn_runtime(&mut self) -> Option<()> {
        let ptr_ty = types::I64;
        let call_conv = self.module.target_config().default_call_conv;

        // __scrap_sched_init()
        if !self.functions.contains_key("__scrap_sched_init") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            let fid = self
                .module
                .declare_function("__scrap_sched_init", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions.insert("__scrap_sched_init".to_string(), fid);
        }

        // __scrap_sched_shutdown()
        if !self.functions.contains_key("__scrap_sched_shutdown") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            let fid = self
                .module
                .declare_function("__scrap_sched_shutdown", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions
                .insert("__scrap_sched_shutdown".to_string(), fid);
        }

        // __scrap_spawn(trampoline: i64, args_ptr: i64, nargs: i64)
        if !self.functions.contains_key("__scrap_spawn") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            sig.params.push(AbiParam::new(ptr_ty)); // trampoline fn pointer
            sig.params.push(AbiParam::new(ptr_ty)); // args_ptr
            sig.params.push(AbiParam::new(ptr_ty)); // nargs
            let fid = self
                .module
                .declare_function("__scrap_spawn", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions.insert("__scrap_spawn".to_string(), fid);
        }

        // __scrap_yield()
        if !self.functions.contains_key("__scrap_yield") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            let fid = self
                .module
                .declare_function("__scrap_yield", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions.insert("__scrap_yield".to_string(), fid);
        }

        Some(())
    }

    /// Get or create a GcShape data section for a given IR type.
    /// Returns the DataId for the shape.
    pub fn get_or_create_gc_shape(&mut self, ty: &ir::Ty<'db>) -> Option<DataId> {
        let key = format!("{:?}", ty);
        if let Some(&data_id) = self.gc_shapes.get(&key) {
            return Some(data_id);
        }

        // Compute shape: size, align, num_pointers, pointer_offsets
        let (size, align, pointer_offsets) = self.compute_type_layout(ty);

        // Build the data: [size: u64, align: u64, num_pointers: u64, offsets: [u64; N]]
        let num_pointers = pointer_offsets.len() as u64;
        let mut data = Vec::new();
        data.extend_from_slice(&size.to_le_bytes());
        data.extend_from_slice(&align.to_le_bytes());
        data.extend_from_slice(&num_pointers.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes()); // finalizer (none)
        for offset in &pointer_offsets {
            data.extend_from_slice(&offset.to_le_bytes());
        }

        let data_name = format!(".Lgcshape.{}", self.gc_shapes.len());
        let data_id = self
            .module
            .declare_data(&data_name, Linkage::Local, false, false)
            .or_emit(self.db)?;

        let mut desc = DataDescription::new();
        desc.define(data.into_boxed_slice());
        desc.set_align(8);
        self.module.define_data(data_id, &desc).or_emit(self.db)?;

        self.gc_shapes.insert(key, data_id);
        Some(data_id)
    }

    /// Compute the (size, align, pointer_offsets) for a type.
    fn compute_type_layout(&self, ty: &ir::Ty<'db>) -> (u64, u64, Vec<u64>) {
        match ty {
            ir::Ty::Bool => (1, 1, vec![]),
            ir::Ty::Int(k) => {
                let bytes = match k {
                    scrap_shared::types::IntTy::I8 => 1,
                    scrap_shared::types::IntTy::I16 => 2,
                    scrap_shared::types::IntTy::I32 => 4,
                    scrap_shared::types::IntTy::I64 | scrap_shared::types::IntTy::Isize => 8,
                    scrap_shared::types::IntTy::I128 => 16,
                };
                (bytes, bytes, vec![])
            }
            ir::Ty::Uint(k) => {
                let bytes = match k {
                    scrap_shared::types::UintTy::U8 => 1,
                    scrap_shared::types::UintTy::U16 => 2,
                    scrap_shared::types::UintTy::U32 => 4,
                    scrap_shared::types::UintTy::U64 | scrap_shared::types::UintTy::Usize => 8,
                    scrap_shared::types::UintTy::U128 => 16,
                };
                (bytes, bytes, vec![])
            }
            ir::Ty::Float(k) => {
                let bytes = match k {
                    scrap_shared::types::FloatTy::F16 => 2,
                    scrap_shared::types::FloatTy::F32 => 4,
                    scrap_shared::types::FloatTy::F64 => 8,
                    scrap_shared::types::FloatTy::F128 => 16,
                };
                (bytes, bytes, vec![])
            }
            ir::Ty::Str => (8, 8, vec![]),        // pointer
            ir::Ty::Ref(_, _) => (8, 8, vec![0]), // reference that the GC must trace
            ir::Ty::Ptr(_) => (8, 8, vec![0]),    // pointer that the GC must trace
            _ => (8, 8, vec![]),                  // default: pointer-sized
        }
    }

    /// Extract user stack maps from the just-compiled function.
    /// Must be called after `define_function()` but before `clear_context()`.
    pub(crate) fn collect_stack_maps(&mut self, func_id: FuncId) {
        let compiled = match self.ctx.compiled_code() {
            Some(c) => c,
            None => return,
        };
        for (code_offset, _frame_span, stack_map) in compiled.buffer.user_stack_maps() {
            let roots: Vec<u32> = stack_map
                .entries()
                .map(|(_ty, sp_offset)| sp_offset)
                .collect();
            if !roots.is_empty() {
                self.stack_map_entries.push((func_id, *code_offset, roots));
            }
        }
    }

    /// Extract Windows x64 unwind info from the just-compiled function.
    /// Must be called after `define_function()` but before `clear_context()`.
    pub(crate) fn collect_unwind_info(&mut self, func_id: FuncId) {
        let code_size = match self.ctx.compiled_code() {
            Some(compiled) => compiled.buffer.data().len() as u32,
            None => return,
        };

        #[allow(deprecated)]
        let unwind_info = match self.ctx.create_unwind_info(self.module.isa()) {
            Ok(Some(info)) => info,
            _ => return,
        };

        if let UnwindInfo::WindowsX64(ref win_info) = unwind_info {
            let mut buf = vec![0u8; win_info.emit_size()];
            win_info.emit(&mut buf);
            self.unwind_entries.push(UnwindEntry {
                func_id,
                code_size,
                unwind_bytes: buf,
            });
        }
    }

    /// Finalize the module and return the object file bytes.
    pub fn finalize(self) -> Option<Vec<u8>> {
        let mut object_product = self.module.finish();

        if !self.unwind_entries.is_empty() {
            Self::emit_unwind_tables(&mut object_product, &self.unwind_entries);
        }

        // Always emit stack map table (even if empty) so runtime symbols resolve.
        Self::emit_stack_map_table(&mut object_product, &self.stack_map_entries);

        object_product
            .emit()
            .map_err(|e| format!("failed to emit object file: {e}"))
            .or_emit(self.db)
    }

    /// Write `.pdata` and `.xdata` sections into the COFF object for Windows SEH.
    fn emit_unwind_tables(product: &mut ObjectProduct, entries: &[UnwindEntry]) {
        use cranelift_object::object::write::{Relocation, SymbolId};
        use cranelift_object::object::{SectionKind, pe};

        // Collect function symbols before taking &mut product.object
        let func_syms: Vec<SymbolId> = entries
            .iter()
            .map(|e| product.function_symbol(e.func_id))
            .collect();

        let obj = &mut product.object;

        // .xdata holds UNWIND_INFO structures
        let xdata_id = obj.add_section(vec![], b".xdata".to_vec(), SectionKind::ReadOnlyData);
        // .pdata holds RUNTIME_FUNCTION entries
        let pdata_id = obj.add_section(vec![], b".pdata".to_vec(), SectionKind::Linker);

        let xdata_sym = obj.section_symbol(xdata_id);

        for (entry, &func_sym) in entries.iter().zip(func_syms.iter()) {
            let xdata_offset = obj.append_section_data(xdata_id, &entry.unwind_bytes, 4);
            let pdata_offset = obj.append_section_data(pdata_id, &[0u8; 12], 4);

            // BeginAddress → RVA of function start
            obj.add_relocation(
                pdata_id,
                Relocation {
                    offset: pdata_offset,
                    symbol: func_sym,
                    addend: 0,
                    flags: cranelift_object::object::RelocationFlags::Coff {
                        typ: pe::IMAGE_REL_AMD64_ADDR32NB,
                    },
                },
            )
            .unwrap();

            // EndAddress → RVA of function end
            obj.add_relocation(
                pdata_id,
                Relocation {
                    offset: pdata_offset + 4,
                    symbol: func_sym,
                    addend: entry.code_size as i64,
                    flags: cranelift_object::object::RelocationFlags::Coff {
                        typ: pe::IMAGE_REL_AMD64_ADDR32NB,
                    },
                },
            )
            .unwrap();

            // UnwindData → RVA of UNWIND_INFO in .xdata
            obj.add_relocation(
                pdata_id,
                Relocation {
                    offset: pdata_offset + 8,
                    symbol: xdata_sym,
                    addend: xdata_offset as i64,
                    flags: cranelift_object::object::RelocationFlags::Coff {
                        typ: pe::IMAGE_REL_AMD64_ADDR32NB,
                    },
                },
            )
            .unwrap();
        }
    }

    /// Write stack map data sections into the COFF object.
    ///
    /// Emits three global symbols:
    ///   - `__scrap_stackmap_count`: u64 number of index entries
    ///   - `__scrap_stackmap_index`: sorted array of (return_addr: u64, roots_start: u32, roots_count: u32)
    ///   - `__scrap_stackmap_roots`: packed array of u32 SP-relative offsets
    ///
    /// The `return_addr` fields carry absolute 64-bit relocations so the linker
    /// fills in absolute code addresses (emitted as the correct per-format type).
    fn emit_stack_map_table(
        product: &mut ObjectProduct,
        entries: &[(FuncId, CodeOffset, Vec<u32>)],
    ) {
        use cranelift_object::object::write::{Relocation, Symbol};
        use cranelift_object::object::{
            RelocationEncoding, RelocationFlags, RelocationKind, SectionKind, SymbolFlags,
            SymbolKind, SymbolScope,
        };

        // Collect function symbols before borrowing product.object mutably.
        let func_syms: Vec<_> = entries
            .iter()
            .map(|(func_id, _, _)| product.function_symbol(*func_id))
            .collect();

        let obj = &mut product.object;

        // __scrap_stackmap_count
        let count_section = obj.add_section(
            vec![],
            b".scrap_smcount".to_vec(),
            SectionKind::ReadOnlyData,
        );
        let count_val = entries.len() as u64;
        obj.append_section_data(count_section, &count_val.to_le_bytes(), 8);
        // Add global symbol for the count
        obj.add_symbol(Symbol {
            name: b"__scrap_stackmap_count".to_vec(),
            value: 0,
            size: 8,
            kind: SymbolKind::Data,
            scope: SymbolScope::Linkage,
            weak: false,
            section: cranelift_object::object::write::SymbolSection::Section(count_section),
            flags: SymbolFlags::None,
        });

        // __scrap_stackmap_roots (packed u32 array)
        let roots_section = obj.add_section(
            vec![],
            b".scrap_smroots".to_vec(),
            SectionKind::ReadOnlyData,
        );
        let mut roots_data: Vec<u8> = Vec::new();
        let mut roots_offsets: Vec<(u32, u32)> = Vec::new(); // (start_index, count) per entry
        let mut root_idx: u32 = 0;
        for (_func_id, _offset, roots) in entries {
            let count = roots.len() as u32;
            roots_offsets.push((root_idx, count));
            for &sp_off in roots {
                roots_data.extend_from_slice(&sp_off.to_le_bytes());
            }
            root_idx += count;
        }
        if roots_data.is_empty() {
            // Emit at least one byte so the section/symbol is valid
            roots_data.push(0);
        }
        obj.append_section_data(roots_section, &roots_data, 4);
        obj.add_symbol(Symbol {
            name: b"__scrap_stackmap_roots".to_vec(),
            value: 0,
            size: roots_data.len() as u64,
            kind: SymbolKind::Data,
            scope: SymbolScope::Linkage,
            weak: false,
            section: cranelift_object::object::write::SymbolSection::Section(roots_section),
            flags: SymbolFlags::None,
        });

        // __scrap_stackmap_index (sorted array of IndexEntry)
        // Each IndexEntry: return_addr(u64) + roots_start(u32) + roots_count(u32) = 16 bytes
        let index_section = obj.add_section(
            vec![],
            b".scrap_smindex".to_vec(),
            SectionKind::Data, // writable so runtime can sort in-place
        );
        let mut index_data: Vec<u8> = Vec::new();
        for (roots_start, roots_count) in &roots_offsets {
            // return_addr placeholder (8 bytes, will be relocated)
            index_data.extend_from_slice(&0u64.to_le_bytes());
            index_data.extend_from_slice(&roots_start.to_le_bytes());
            index_data.extend_from_slice(&roots_count.to_le_bytes());
        }
        if index_data.is_empty() {
            // Emit a dummy entry so the section isn't discarded by the linker.
            // Count is 0 so the runtime won't read past this.
            index_data.extend_from_slice(&[0u8; 16]);
        }
        let index_base = obj.append_section_data(index_section, &index_data, 8);
        let index_sym_id = obj.add_symbol(Symbol {
            name: b"__scrap_stackmap_index".to_vec(),
            value: 0,
            size: index_data.len() as u64,
            kind: SymbolKind::Data,
            scope: SymbolScope::Linkage,
            weak: false,
            section: cranelift_object::object::write::SymbolSection::Section(index_section),
            flags: SymbolFlags::None,
        });
        let _ = index_sym_id;

        // Add relocations for each return_addr field
        for (i, ((_func_id, code_offset, _), &func_sym)) in
            entries.iter().zip(func_syms.iter()).enumerate()
        {
            let entry_offset = index_base + (i * 16) as u64;
            obj.add_relocation(
                index_section,
                Relocation {
                    offset: entry_offset,
                    symbol: func_sym,
                    addend: *code_offset as i64,
                    // Generic absolute 64-bit relocation; the object writer maps
                    // this to the right per-format type (IMAGE_REL_AMD64_ADDR64
                    // on COFF, R_X86_64_64 on ELF, the Mach-O equivalent).
                    flags: RelocationFlags::Generic {
                        kind: RelocationKind::Absolute,
                        encoding: RelocationEncoding::Generic,
                        size: 64,
                    },
                },
            )
            .unwrap();
        }
    }
}
