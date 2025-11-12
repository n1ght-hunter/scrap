use clap::{Parser, ValueEnum};

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(true)
        .init();
}



fn main() -> anyhow::Result<()> {
    setup_logging();


    let ast = scrap_parser::parse_files(files)?;

    if let Some(UnPrettyOut::Ast) = args.unpretty_out {
        println!("{:#?}", ast);
        return Ok(());
    }

    let mir = scrap_ir::mir_builder::MirBuilder::new().lower_can(ast)?;

    if let Some(UnPrettyOut::Mir) = args.unpretty_out {
        println!("{:#?}", mir);
        return Ok(());
    }

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
