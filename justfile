


build *args:
    cargo run -- {{args}}

run *args:
    cargo run -p scrap -- {{args}}

fmt *args:
    cargo run -p scrap_formatter -- {{args}}

build:
    cargo build -r

clean:
    rm -r ./target/scrap || true0

hello_world *args:
    just build E:/programming/rust/scrap/example/hello_world.sc --crate-name test --crate-type bin --unpretty-out ast --cache ./target/scrap/hello_world_db_snapshot.bin {{args}}

complex *args:
    just build E:/programming/rust/scrap/example/complex.sc \
    -i E:/programming/rust/scrap/example/external_module.sc \
    --crate-name complex --crate-type bin --cache ./target/scrap/complex/cache {{args}}

complex_quick *args:
    ./target/release/scrap.exe E:/programming/rust/scrap/example/complex.sc --crate-name test --crate-type bin --unpretty-out ast --cache ./target/scrap/complex/quick_cache {{args}}

basic *args:
    just build E:/programming/rust/scrap/tests/basic.sc --crate-name test --crate-type bin --unpretty-out sir --cache ./target/scrap/basic_cache {{args}}

test-types *args:
    rm ./target/scrap/types_cache.json || true
    just build E:/programming/rust/scrap/tests/types.sc --crate-name test --crate-type bin --pretty-out sir --cache ./target/scrap/types_cache {{args}}

compile-runtime:
    cargo build -p scrap_rt --release