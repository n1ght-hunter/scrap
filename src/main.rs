use scrap_codegen::{jit::JitCompiler, object::ObjectCompiler};

fn main() -> anyhow::Result<()> {
    println!("Scrap Programming Language - Code Generation Demo");
    
    // Create a JIT compiler
    let _jit_compiler = JitCompiler::new()?;
    println!("✓ Created JIT compiler");
    
    // Create an object compiler
    let _object_compiler = ObjectCompiler::new()?;
    println!("✓ Created object compiler");
    
    println!("\nCode generation infrastructure ready!");
    println!("Available backends:");
    println!("  - JIT compilation for runtime execution");
    println!("  - Object file generation for static compilation");
    
    Ok(())
}
