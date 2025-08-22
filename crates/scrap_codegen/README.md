# Scrap Codegen

Code generation backend for the Scrap programming language using Cranelift.

## Features

- **JIT Compilation**: Compile and execute Scrap code at runtime
- **Object File Generation**: Generate native object files for static compilation
- **Cranelift Backend**: Uses Cranelift as the code generation backend for high performance
- **Multiple Targets**: Support for multiple target architectures through Cranelift

## Components

### CodeGenerator

The main code generator that provides a high-level interface for compiling Scrap programs.

```rust
use scrap_codegen::CodeGenerator;

let mut codegen = CodeGenerator::new()?;
// codegen.compile_program(&ast)?;
// codegen.finalize()?;
```

### JIT Compiler

Provides just-in-time compilation for runtime execution:

```rust
use scrap_codegen::jit::JitCompiler;

let mut jit = JitCompiler::new()?;
// jit.compile_function(&function)?;
// jit.finalize()?;
// let result = jit.execute_function_i64("function_name")?;
```

### Object Compiler

Generates object files for static compilation:

```rust
use scrap_codegen::object::ObjectCompiler;

let mut compiler = ObjectCompiler::new()?;
// compiler.compile_program(&ast)?;
// compiler.write_object_file(Path::new("output.o"))?;
```

## Dependencies

- **cranelift**: Core code generation library
- **cranelift-jit**: JIT compilation support
- **cranelift-object**: Object file generation
- **cranelift-native**: Native target detection
- **cranelift-module**: Module management

## Current Status

This crate is in early development. The following features are planned:

- [ ] Function compilation
- [ ] Expression compilation
- [ ] Type system integration
- [ ] Memory management
- [ ] Optimization passes
- [ ] Debug information generation

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
scrap_codegen = { path = "crates/scrap_codegen" }
```
