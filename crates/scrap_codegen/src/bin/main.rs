use cranelift::prelude::*;
use cranelift_module::{DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use target_lexicon::Triple;

/// Creates the `main` function which handles the core logic of printing the message.
/// This function is equivalent to `int main()` in C.
fn create_main_function(
    module: &mut ObjectModule,
    data_id: cranelift_module::DataId,
    message_len: usize,
) -> Result<cranelift_module::FuncId, String> {
    let mut func_ctx = FunctionBuilderContext::new();
    let mut ctx = module.make_context();

    let mut main_sig = module.make_signature();
    main_sig.returns.push(AbiParam::new(types::I32));
    main_sig.call_conv = module.target_config().default_call_conv;

    let main_func_id = module
        .declare_function("main", Linkage::Local, &main_sig)
        .map_err(|e| e.to_string())?;
    ctx.func.signature = main_sig;

    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
    let entry_block = builder.create_block();
    builder.switch_to_block(entry_block);

    // --- IR to call GetStdHandle ---
    let mut get_std_handle_sig = module.make_signature();
    get_std_handle_sig.params.push(AbiParam::new(types::I32));
    get_std_handle_sig.returns.push(AbiParam::new(types::I64));
    let get_std_handle_func_id = module
        .declare_function("GetStdHandle", Linkage::Import, &get_std_handle_sig)
        .map_err(|e| e.to_string())?;

    // FIX: Convert FuncId to a local FuncRef before calling.
    let get_std_handle_ref = module.declare_func_in_func(get_std_handle_func_id, builder.func);

    let std_output_handle_const = builder.ins().iconst(types::I32, -11i64);
    let call_get_handle = builder
        .ins()
        .call(get_std_handle_ref, &[std_output_handle_const]);
    let stdout_handle = builder.inst_results(call_get_handle)[0];

    // --- IR to call WriteFile ---
    let mut write_file_sig = module.make_signature();
    // FIX: Use push for each parameter instead of append.
    write_file_sig.params.push(AbiParam::new(types::I64)); // hFile
    write_file_sig.params.push(AbiParam::new(types::I64)); // lpBuffer
    write_file_sig.params.push(AbiParam::new(types::I32)); // nNumberOfBytesToWrite
    write_file_sig.params.push(AbiParam::new(types::I64)); // lpNumberOfBytesWritten
    write_file_sig.params.push(AbiParam::new(types::I64)); // lpOverlapped
    write_file_sig.returns.push(AbiParam::new(types::I32));
    let write_file_func_id = module
        .declare_function("WriteFile", Linkage::Import, &write_file_sig)
        .map_err(|e| e.to_string())?;

    // FIX: Convert FuncId to a local FuncRef.
    let write_file_ref = module.declare_func_in_func(write_file_func_id, builder.func);

    // Prepare arguments
    let message_data_addr = module.declare_data_in_func(data_id, builder.func);
    let message_ptr = builder.ins().symbol_value(types::I64, message_data_addr);

    // FIX: Use the known message length passed into this function.
    let message_len_val = builder.ins().iconst(types::I32, message_len as i64);

    // FIX: Use `create_sized_stack_slot` and a simpler `StackSlotData`.
    let stack_slot =
        builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 4, 4));
    let bytes_written_ptr = builder.ins().stack_addr(types::I64, stack_slot, 0);

    let null_ptr = builder.ins().iconst(types::I64, 0);

    // Generate the call
    builder.ins().call(
        write_file_ref,
        &[
            stdout_handle,
            message_ptr,
            message_len_val,
            bytes_written_ptr,
            null_ptr,
        ],
    );

    let return_code = builder.ins().iconst(types::I32, 0);
    builder.ins().return_(&[return_code]);

    builder.seal_all_blocks();
    builder.finalize();

    module
        .define_function(main_func_id, &mut ctx)
        .map_err(|e| e.to_string())?;

    Ok(main_func_id)
}

/// Creates the `_start` function, which is the true entry point of the program.
fn create_start_function(
    module: &mut ObjectModule,
    main_func_id: cranelift_module::FuncId,
) -> Result<(), String> {
    let mut func_ctx = FunctionBuilderContext::new();
    let mut ctx = module.make_context();

    let mut start_sig = module.make_signature();
    start_sig.call_conv = module.target_config().default_call_conv;

    let start_func_id = module
        .declare_function("_start", Linkage::Export, &start_sig)
        .map_err(|e| e.to_string())?;
    ctx.func.signature = start_sig;

    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
    let entry_block = builder.create_block();
    builder.switch_to_block(entry_block);

    // --- IR to call main() ---
    let main_ref = module.declare_func_in_func(main_func_id, builder.func);
    let _call_main = builder.ins().call(main_ref, &[]);
    // let main_exit_code = builder.inst_results(call_main)[0];
    let main_exit_code = builder.ins().iconst(types::I32, 30); // FIX: Placeholder for main's return value

    // --- IR to call ExitProcess ---
    let mut exit_process_sig = module.make_signature();
    exit_process_sig.params.push(AbiParam::new(types::I32));
    let exit_process_func_id = module
        .declare_function("ExitProcess", Linkage::Import, &exit_process_sig)
        .map_err(|e| e.to_string())?;

    // FIX: Convert FuncId to a local FuncRef.
    let exit_process_ref = module.declare_func_in_func(exit_process_func_id, builder.func);

    builder.ins().call(exit_process_ref, &[main_exit_code]);

    builder.ins().return_(&[]);
    builder.seal_all_blocks();
    builder.finalize();

    module
        .define_function(start_func_id, &mut ctx)
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() -> Result<(), String> {
    let output_dir = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("target/scrap");

    if !output_dir.exists() {
        std::fs::create_dir(&output_dir).map_err(|e| e.to_string())?;
    }

    // FIX: Use `from_str` to parse the triple string.
    let target_triple = Triple::from_str("x86_64-pc-windows-msvc").map_err(|e| e.to_string())?;

    let shared_builder = settings::builder();
    let shared_flags = settings::Flags::new(shared_builder);
    let isa = cranelift::codegen::isa::lookup(target_triple)
        .map_err(|e| e.to_string())?
        .finish(shared_flags)
        .map_err(|e| e.to_string())?;

    let object_builder =
        ObjectBuilder::new(isa, "hello", cranelift_module::default_libcall_names()).unwrap();
    let mut module = ObjectModule::new(object_builder);

    // FIX: Use `DataContext` (from the use statement, which is the new `DataDescription`)
    let message = "Hello, world from a corrected Cranelift binary!\n";
    let mut data_context = DataDescription::new();
    data_context.define(message.as_bytes().to_vec().into_boxed_slice());
    let data_id = module
        .declare_data("message_data", Linkage::Local, false, false)
        .map_err(|e| e.to_string())?;
    module
        .define_data(data_id, &data_context)
        .map_err(|e| e.to_string())?;

    let main_func_id = create_main_function(&mut module, data_id, message.len())?;
    create_start_function(&mut module, main_func_id)?;

    let object_product = module.finish();
    let object_bytes = object_product.emit().map_err(|e| e.to_string())?;

    let obj_path = output_dir.join("hello.obj");

    let mut output_file = File::create(&obj_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&object_bytes)
        .map_err(|e| e.to_string())?;

    println!("✅ Successfully created hello.obj");
    println!("Next, link it using the command:");
    println!("   link.exe hello.obj kernel32.lib /SUBSYSTEM:CONSOLE /ENTRY:_start /OUT:hello.exe");
    //  lld-link.exe  hello.obj kernel32.lib ucrt.lib /SUBSYSTEM:CONSOLE /OUT:hello.exe

    let exe = output_dir.join("hello.exe");

    let res = std::process::Command::new("lld-link.exe")
        .args([
            obj_path.to_str().unwrap(),
            "kernel32.lib",
            "/SUBSYSTEM:CONSOLE",
            "/ENTRY:_start",
            &format!("/OUT:{}", exe.display()),
        ])
        .status()
        .map_err(|e| e.to_string())?;

    if res.success() {
        println!("✅ Successfully linked to create hello.exe");
    }

    if !res.success() {
        return Err("Linking failed".to_string());
    }

    let res = std::process::Command::new(exe)
        .status()
        .map_err(|e| e.to_string())?;

    if res.success() {
        println!("✅ Successfully ran hello.exe");
    }

    if !res.success() {
        return Err(format!(
            "Execution of hello.exe failed with exit code: {}",
            res.code().unwrap_or(-1)
        ));
    }

    Ok(())
}
