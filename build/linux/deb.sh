#!/usr/bin/env bash
dir=$( dirname -- "$0"; )

cd "$dir/../.." || exit
cargo install cargo-bundle
cargo +nightly bundle --release --format deb
