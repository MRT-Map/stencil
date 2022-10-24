#!/usr/bin/env bash
dir=$( dirname -- "$0"; )

cd "$dir/../.." || exit
cargo install cargo-bundle
cargo +nightly bundle --release

cp -R "$dir/../../target/release/bundle/osx/stencil2.app" "$dir/app/stencil2.app"
ln -s "/Applications" "$dir/app"
hdiutil create stencil2.dmg -volname stencil2 -srcfolder "$dir/app" -ov
unlink "$dir/app/Applications"
rm -r "$dir/app/stencil2.app"