fn setup_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(true)
        .init();
}

fn main() -> anyhow::Result<()> {
    setup_logging();

    let files = vec!["example/basic.sc"];
    let ast = scrap_parser::parse_files(files)?;
    std::fs::write("ast.ron", format!("{:#?}", ast))?;

    // println!("Scrap Programming Language - Code Generation Demo");

    // // Create a JIT compiler
    // let _jit_compiler = JitCompiler::new()?;
    // println!("✓ Created JIT compiler");

    // // Create an object compiler
    // let _object_compiler = ObjectCompiler::new()?;
    // println!("✓ Created object compiler");

    // println!("\nCode generation infrastructure ready!");
    // println!("Available backends:");
    // println!("  - JIT compilation for runtime execution");
    // println!("  - Object file generation for static compilation");

    Ok(())
}
