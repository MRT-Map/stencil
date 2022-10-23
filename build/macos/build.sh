#!/usr/bin/env bash
dir=$( dirname -- "$0"; )

iconutil --convert icns "$dir/AppIcon.iconset"
mkdir "$dir/app/stencil2.app/Contents/Resources"
mv "$dir/AppIcon.icns" "$dir/app/stencil2.app/Contents/Resources"

mkdir "$dir/app/stencil2.app/Contents/MacOS"
cp "$dir/../../target/release/stencil2" "$dir/app/stencil2.app/Contents/MacOS/stencil2"
ln -s "/Applications" "$dir/app"
hdiutil create stencil2.dmg -volname stencil2 -srcfolder "$dir/app" -ov
unlink "$dir/app/Applications"
rm "$dir/app/stencil2.app/Contents/MacOS/stencil2"
rm "$dir/app/stencil2.app/Contents/Resources/AppIcon.icns"