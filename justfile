#!/usr/bin/env just --justfile

crate := "speq"

# Show all available recipes
default:
    @just --list

# Compile the crate
build:
    cargo build

# Compile with optimisations
build-release:
    cargo build --release

# Run tests
test:
    cargo test

# Format the source
fmt:
    cargo fmt

# Check formatting without changing anything
fmt-check:
    cargo fmt --check

# Lint with clippy, treating warnings as errors
lint:
    cargo clippy --all-targets -- -D warnings

# Run against a spec file (defaults to the Petstore fixture)
run spec="fixtures/petstore.yaml":
    cargo run -- {{spec}}

# Install into ~/.cargo/bin from this working tree
install:
    cargo install --path . --force

# Force a fresh compile of this crate, sidestepping stale fingerprints
rebuild:
    cargo clean -p {{crate}}
    cargo build

# Full pre-release check: fresh compile, lints, tests
verify: rebuild lint test
    @echo "✓ verify passed"

# Print the current version from Cargo.toml
version:
    @grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/'

# Bump the version in Cargo.toml — level is patch, minor or major
bump level="patch":
    cargo set-version --bump {{level}}
    @echo "✓ version is now $(just version)"

# Set an explicit version, e.g. just set-version 0.2.0
set-version version:
    cargo set-version {{version}}
    @echo "✓ version is now $(just version)"

# Log in to crates.io (needed once per machine)
login:
    cargo login

# Dry-run a publish without uploading anything
publish-dry:
    cargo publish --dry-run --allow-dirty

# Bump, verify, tag, push and publish to crates.io
release level="patch":
    #!/usr/bin/env bash
    set -euo pipefail

    branch=$(git rev-parse --abbrev-ref HEAD)
    if [ "$branch" != "main" ]; then
        echo "✗ not on main (on $branch)" >&2
        exit 1
    fi

    if [ -n "$(git status --porcelain)" ]; then
        echo "✗ working tree is dirty — commit or stash first" >&2
        exit 1
    fi

    git fetch --quiet origin main
    if [ -n "$(git rev-list HEAD..origin/main)" ]; then
        echo "✗ origin/main is ahead — pull first" >&2
        exit 1
    fi

    just verify

    old=$(just version)
    cargo set-version --bump {{level}}
    new=$(just version)
    echo "▶ releasing {{crate}} $old → $new"

    cargo publish --dry-run --allow-dirty

    read -r -p "Publish {{crate}} v$new to crates.io? This cannot be undone. [y/N] " reply
    if [ "$reply" != "y" ] && [ "$reply" != "Y" ]; then
        git checkout -- Cargo.toml Cargo.lock
        echo "✗ aborted — version reverted to $old"
        exit 1
    fi

    git add Cargo.toml Cargo.lock
    git commit -m "Release v$new"
    git tag -a "v$new" -m "v$new"
    git push origin main --follow-tags

    cargo publish

    echo "✓ published {{crate}} v$new"
    echo "  https://crates.io/crates/{{crate}}/$new"
