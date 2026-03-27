set positional-arguments

# Show available commands
help:
    @just --list --unsorted

# Run all checks
@all: build fmt clippy test

# Run cargo build
@build:
    cargo build

# Run cargo fmt
@fmt:
    cargo fmt

# Run cargo clippy
@clippy:
    cargo clippy

# Run cargo test
@test:
    cargo test
