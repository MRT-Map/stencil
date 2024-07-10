#!/usr/bin/env bash
dir=$( dirname -- "$0"; )
set -euxo pipefail
cd "$dir/../.." || exit

sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
brew install llvm
rustup +nightly target add x86_64-apple-darwin
rustup +nightly target add aarch64-apple-darwin

cargo +nightly build --release --target x86_64-apple-darwin
cargo +nightly build --release --target aarch64-apple-darwin

cargo install cargo-bundle
cargo +nightly bundle --release --target x86_64-apple-darwin

mkdir "$dir/app"
cp -R "$dir/../../target/x86_64-apple-darwin/release/bundle/osx/stencil2.app" "$dir/app/stencil2.app"

lipo "$dir/../../target/x86_64-apple-darwin/release/stencil2" \
     "$dir/../../target/aarch64-apple-darwin/release/stencil2" \
     -create -output "$dir/app/stencil2.app/Contents/MacOS/stencil2"

cargo clean

ln -s "/Applications" "$dir/app"
hdiutil create stencil2.dmg -volname stencil2 -srcfolder "$dir/app" -ov
unlink "$dir/app/Applications"
rm -r "$dir/app/stencil2.app"
