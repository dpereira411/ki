set shell := ["zsh", "-cu"]

default:
    @just --list

build:
    cargo build

release:
    cargo build --release

test:
    cargo test

run *args:
    cargo run -- {{args}}

fmt:
    cargo fmt

check:
    cargo check

install:
    cargo install --path .
