#
# This file defines the rules that one can call from the `just` utility.
#
# Authors:
#   Julien Peeters <inthehack@mountainhacks.org>
#

set quiet := true

# Initialize environment.
[group('utility')]
mod init 'just/init.just'

# Print this message.
help:
    just --list

# Build sources (cargo build).
[group('build')]
build *OPTS:
    cargo build --workspace {{ OPTS }}

# Build sources (cargo build).
[group('build')]
build-pkg pkg *OPTS:
    cargo build -p {{ pkg }} {{ OPTS }}

# Check if sources are compliant with lint rules (cargo clippy).
[group('quality')]
lint *OPTS:
    cargo clippy --workspace {{ OPTS }} -- -D warnings

# Check if sources are compliant with lint rules (cargo clippy --fix).
[group('quality')]
fix *OPTS: && (lint OPTS "--fix")

# Format source code (nix fmt).
[group('quality')]
format *OPTS:
    nix fmt {{ OPTS }}

alias fmt := format

# Test all.
test *OPTS:
    cargo nextest run --workspace {{ OPTS }}

# Test package.
test-pkg pkg *OPTS:
    cargo nextest run -p {{ pkg }} {{ OPTS }}

# Clean the cargo build artifacts.
[group('utility')]
clean:
    rm -rf target

# Wipe all non-versioned data.
[group("utility")]
mrproper:
    git clean -dffx
