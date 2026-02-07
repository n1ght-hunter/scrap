//! CodegenContext — holds the Cranelift module and compilation state.

use cranelift::prelude::*;
use cranelift::codegen::isa::unwind::UnwindInfo;
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use scrap_ir as ir;
use std::collections::HashMap;
use std::str::FromStr;
use target_lexicon::Triple;

use super::emit_codegen_err;
use super::ResultExt;

/// Per-function unwind metadata collected during compilation.
pub(crate) struct UnwindEntry {
    pub func_id: FuncId,
    pub code_size: u32,
    pub unwind_bytes: Vec<u8>,
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
}

impl<'db> CodegenContext<'db> {
    /// Create a new code generation context targeting x86_64-pc-windows-msvc.
    pub fn new(db: &'db dyn scrap_shared::Db) -> Option<Self> {
        let target_triple =
            Triple::from_str("x86_64-pc-windows-msvc").map_err(|e| {
                format!("failed to parse target triple: {e}")
            }).or_emit(db)?;

        let shared_builder = settings::builder();
        let shared_flags = settings::Flags::new(shared_builder);
        let isa = cranelift::codegen::isa::lookup(target_triple)
            .map_err(|e| format!("ISA lookup failed: {e}"))
            .or_emit(db)?
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
        })
    }

    /// Compile an entire IR module (declare then define).
    pub fn compile_module(&mut self, module: ir::Module<'db>) -> Option<()> {
        self.declare_items(module)?;
        self.declare_panic_runtime()?;
        self.define_functions(module)?;
        self.define_panic_function()?;
        Some(())
    }

    /// Generate the `_start` entry point that calls `main`.
    pub fn generate_start(&mut self) -> Option<()> {
        let main_func_id = match self.functions.get("main").copied() {
            Some(id) => id,
            None => {
                emit_codegen_err(self.db, "function 'main' not found");
                return None;
            }
        };

        // Declare _start: no params, no returns
        let mut start_sig = self.module.make_signature();
        start_sig.call_conv = self.module.target_config().default_call_conv;

        let start_func_id = self
            .module
            .declare_function("_start", Linkage::Export, &start_sig)
            .or_emit(self.db)?;

        self.ctx.func.signature = start_sig;

        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.func_ctx);
            let entry_block = builder.create_block();
            builder.switch_to_block(entry_block);

            // Call main
            let main_ref =
                self.module.declare_func_in_func(main_func_id, builder.func);
            builder.ins().call(main_ref, &[]);

            // If main returns (it shouldn't if it returns !), trap
            builder.ins().trap(TrapCode::user(1).unwrap());

            builder.seal_all_blocks();
            builder.finalize();
        }

        self.module
            .define_function(start_func_id, &mut self.ctx)
            .or_emit(self.db)?;

        self.collect_unwind_info(start_func_id);
        self.module.clear_context(&mut self.ctx);

        Some(())
    }

    /// Declare the panic runtime: `__scrap_panic` and its Windows API dependencies.
    /// Must be called before `define_functions()` so user code can reference `__scrap_panic`.
    pub fn declare_panic_runtime(&mut self) -> Option<()> {
        let ptr_ty = types::I64;
        let call_conv = self.module.target_config().default_call_conv;

        // Ensure Windows API imports exist
        // GetStdHandle(nStdHandle: i64) -> i64
        if !self.functions.contains_key("GetStdHandle") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            sig.params.push(AbiParam::new(ptr_ty));
            sig.returns.push(AbiParam::new(ptr_ty));
            let fid = self
                .module
                .declare_function("GetStdHandle", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions.insert("GetStdHandle".to_string(), fid);
        }

        // WriteFile(hFile, lpBuffer, nBytes, lpBytesWritten, lpOverlapped) -> i64
        if !self.functions.contains_key("WriteFile") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            sig.params.push(AbiParam::new(ptr_ty)); // hFile
            sig.params.push(AbiParam::new(ptr_ty)); // lpBuffer
            sig.params.push(AbiParam::new(ptr_ty)); // nNumberOfBytesToWrite
            sig.params.push(AbiParam::new(ptr_ty)); // lpNumberOfBytesWritten
            sig.params.push(AbiParam::new(ptr_ty)); // lpOverlapped
            sig.returns.push(AbiParam::new(ptr_ty));
            let fid = self
                .module
                .declare_function("WriteFile", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions.insert("WriteFile".to_string(), fid);
        }

        // ExitProcess(exit_code: i64) -> !  (no return)
        if !self.functions.contains_key("ExitProcess") {
            let mut sig = self.module.make_signature();
            sig.call_conv = call_conv;
            sig.params.push(AbiParam::new(ptr_ty));
            let fid = self
                .module
                .declare_function("ExitProcess", Linkage::Import, &sig)
                .or_emit(self.db)?;
            self.functions.insert("ExitProcess".to_string(), fid);
        }

        // Declare __scrap_panic(msg_ptr: i64, msg_len: i64) -> !
        let mut panic_sig = self.module.make_signature();
        panic_sig.call_conv = call_conv;
        panic_sig.params.push(AbiParam::new(ptr_ty)); // msg_ptr
        panic_sig.params.push(AbiParam::new(ptr_ty)); // msg_len

        let panic_func_id = self
            .module
            .declare_function("__scrap_panic", Linkage::Local, &panic_sig)
            .or_emit(self.db)?;
        self.functions
            .insert("__scrap_panic".to_string(), panic_func_id);

        Some(())
    }

    /// Define the `__scrap_panic` function body.
    /// Must be called after `define_functions()`.
    ///
    /// Implementation:
    ///   1. GetStdHandle(STD_ERROR_HANDLE) → handle
    ///   2. WriteFile(handle, msg_ptr, msg_len, 0, 0)
    ///   3. ExitProcess(101)
    pub fn define_panic_function(&mut self) -> Option<()> {
        let ptr_ty = types::I64;
        let call_conv = self.module.target_config().default_call_conv;

        let panic_func_id = match self.functions.get("__scrap_panic").copied() {
            Some(id) => id,
            None => {
                emit_codegen_err(self.db, "function '__scrap_panic' not declared");
                return None;
            }
        };

        let mut panic_sig = self.module.make_signature();
        panic_sig.call_conv = call_conv;
        panic_sig.params.push(AbiParam::new(ptr_ty));
        panic_sig.params.push(AbiParam::new(ptr_ty));

        self.ctx.func.signature = panic_sig;

        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.func_ctx);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);

            let params = builder.block_params(entry_block).to_vec();
            let msg_ptr = params[0];
            let msg_len = params[1];

            // 1. handle = GetStdHandle(STD_ERROR_HANDLE)
            //    STD_ERROR_HANDLE = (DWORD)-12 = 0xFFFFFFF4 = 4294967284
            let get_std_handle_id = self.functions["GetStdHandle"];
            let get_std_handle_ref = self
                .module
                .declare_func_in_func(get_std_handle_id, builder.func);
            let stderr_const = builder.ins().iconst(ptr_ty, 4294967284_i64);
            let call_gsh = builder.ins().call(get_std_handle_ref, &[stderr_const]);
            let handle = builder.inst_results(call_gsh)[0];

            // 2. WriteFile(handle, msg_ptr, msg_len, 0, 0)
            let write_file_id = self.functions["WriteFile"];
            let write_file_ref = self
                .module
                .declare_func_in_func(write_file_id, builder.func);
            let zero = builder.ins().iconst(ptr_ty, 0);
            builder
                .ins()
                .call(write_file_ref, &[handle, msg_ptr, msg_len, zero, zero]);

            // 3. ExitProcess(101)
            let exit_process_id = self.functions["ExitProcess"];
            let exit_process_ref = self
                .module
                .declare_func_in_func(exit_process_id, builder.func);
            let exit_code = builder.ins().iconst(ptr_ty, 101);
            builder.ins().call(exit_process_ref, &[exit_code]);

            // Trap as fallback (ExitProcess never returns)
            builder.ins().trap(TrapCode::user(1).unwrap());

            builder.seal_all_blocks();
            builder.finalize();
        }

        self.module
            .define_function(panic_func_id, &mut self.ctx)
            .or_emit(self.db)?;

        self.collect_unwind_info(panic_func_id);
        self.module.clear_context(&mut self.ctx);

        Some(())
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

        object_product
            .emit()
            .map_err(|e| format!("failed to emit object file: {e}"))
            .or_emit(self.db)
    }

    /// Write `.pdata` and `.xdata` sections into the COFF object for Windows SEH.
    fn emit_unwind_tables(product: &mut ObjectProduct, entries: &[UnwindEntry]) {
        use cranelift_object::object::write::{Relocation, SymbolId};
        use cranelift_object::object::{pe, SectionKind};

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
            obj.add_relocation(pdata_id, Relocation {
                offset: pdata_offset,
                symbol: func_sym,
                addend: 0,
                flags: cranelift_object::object::RelocationFlags::Coff {
                    typ: pe::IMAGE_REL_AMD64_ADDR32NB,
                },
            })
            .unwrap();

            // EndAddress → RVA of function end
            obj.add_relocation(pdata_id, Relocation {
                offset: pdata_offset + 4,
                symbol: func_sym,
                addend: entry.code_size as i64,
                flags: cranelift_object::object::RelocationFlags::Coff {
                    typ: pe::IMAGE_REL_AMD64_ADDR32NB,
                },
            })
            .unwrap();

            // UnwindData → RVA of UNWIND_INFO in .xdata
            obj.add_relocation(pdata_id, Relocation {
                offset: pdata_offset + 8,
                symbol: xdata_sym,
                addend: xdata_offset as i64,
                flags: cranelift_object::object::RelocationFlags::Coff {
                    typ: pe::IMAGE_REL_AMD64_ADDR32NB,
                },
            })
            .unwrap();
        }
    }
}
