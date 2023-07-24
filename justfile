default:
    @just -l

loc:
    find src/ -name "*.rs" | xargs cat | wc -l

build:
    cargo build --release

publish: build
    cargo clippy --forbid all
    cargo test

# vim: set ft=make :
