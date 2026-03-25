# Scrap

A compiled programming language with Rust-like syntax and Go-like development experience.

**This language is highly experimental. Many features are incomplete or broken. Expect rough edges everywhere.**

## Features

- Structs, enums, methods, pattern matching
- References (`&T`, `&mut T`) and GC pointers (`*T`)
- `spawn` for concurrent tasks on an M:N scheduler
- Growable coroutine stacks (8 KiB initial, up to 1 MiB)
- Stop-the-world garbage collector with Cranelift stack maps
- Incremental compilation via Salsa
- `extern "C"` FFI for calling C APIs directly

## Example

```
fn main() {
    print("Hello, world!");
}
```

```
fn main() {
    spawn worker(42);
}

fn worker(n: i64) {
    print_int(n);
}
```

## Usage

```sh
# compile and run a .sc file
cargo run -- path/to/file.sc --crate-name myapp --crate-type bin
```

## Contributing

No contributions are being accepted at this time.
Issues can be opened and discussed, but no code contributions will be accepted until the language is more stable and the codebase is more mature.

## Disclaimer

The Rowan-based parser (`scrap_parser_rowan`) was written by LLMs as an experiment and largely does not work. It is not used by the main compiler pipeline. Various other components are partly written or modified by LLMs. As the compiler gets more complete, the LLM-written code will be replaced with human-written code. Currently the hand-written recursive descent parser is almost completely hand written.

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE).
