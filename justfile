


run *args:
    cargo run -- {{args}}

hello_world *args:
    just run E:/programming/rust/scrap/example/hello_world.sc --crate-name test --crate-type bin --unpretty-out ast --cache ./target/scrap/hello_world_db_snapshot.bin {{args}}