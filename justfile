default:
    @just -l

loc:
    find src/ -name "*.rs" | xargs cat | wc -l

build:
    cargo build --release

publish: build 
    cargo fmt
    cargo clippy -q -- -D warnings 
    cargo test -q
