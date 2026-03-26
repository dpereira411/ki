set shell := ["zsh", "-cu"]

bin_name := "ki"
install_dir := if env_var_or_default("INSTALL_DIR", "") == "" { "/usr/local/bin" } else { env_var("INSTALL_DIR") }
install_path := if env_var_or_default("INSTALL_PATH", "") == "" { install_dir + "/" + bin_name } else { env_var("INSTALL_PATH") }

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

install: release
    install -d "$(dirname {{install_path}})"
    install -m 0755 target/release/{{bin_name}} "{{install_path}}"
    @echo "installed {{bin_name}} to {{install_path}}"
