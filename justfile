


run *args:
    cargo run -- {{args}}

build:
    cargo build -r

hello_world *args:
    just run E:/programming/rust/scrap/example/hello_world.sc --crate-name test --crate-type bin --unpretty-out ast --cache ./target/scrap/hello_world_db_snapshot.bin {{args}}

complex *args:
    RUST_LOG=info just run E:/programming/rust/scrap/example/complex.sc --crate-name test --crate-type bin --cache ./target/scrap/complex/cache {{args}}

complex_quick *args:
    ./target/release/scrap.exe E:/programming/rust/scrap/example/complex.sc --crate-name test --crate-type bin --unpretty-out ast --cache ./target/scrap/complex/quick_cache {{args}}