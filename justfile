default:
    @just -l

loc:
    find src/ -name "*.rs" | xargs cat | wc -l

build:
    cargo build --release

publish:
    cargo fmt
    cargo clippy -q -- -D warnings 
    cargo test -q

debug:
    RUST_BACKTRACE=1 RUST_LOG=info cargo run -- ~/pix/art/war.jpg
