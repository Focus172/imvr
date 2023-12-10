default:
    @just -l

loc:
    find src/ -name "*.rs" | xargs cat | wc -l

publish:
    cargo fmt
    cargo clippy -q -- -D warnings 
    cargo test -q

debug:
    RUST_BACKTRACE=1 RUST_LOG=error,imvr=trace cargo run -- ~/pix/art/war.jpg

install:
    cargo build --release
    cp ./target/release/imvr ~/.local/bin/
