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
    cargo build {{ OPTS }}

# Build sources in release mode (cargo build --release).
[group('build')]
build-release *OPTS: && (build OPTS "--release")

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

# Clean the cargo build artifacts.
[group('utility')]
clean:
    rm -rf target

# Wipe all non-versioned data.
[group("utility")]
mrproper:
    git clean -dffx
