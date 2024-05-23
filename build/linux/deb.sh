#!/usr/bin/env bash
dir=$( dirname -- "$0"; )
set -euxo pipefail
cd "$dir/../.." || exit

sudo apt-get update; sudo apt-get install --no-install-recommends libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
cargo +nightly build --release

cargo install cargo-bundle
cargo +nightly bundle --release --format deb
