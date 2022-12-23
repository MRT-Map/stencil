$dir = $PSScriptRoot
Set-Location "$dir\..\.."
#Invoke-WebRequest -Uri https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311.exe -OutFile wix311.exe
cargo install cargo-wix
cargo wix init -o "$dir" --force --product-icon ".\build\macos\AppIcon.iconset\icon_512x512@2x.png" -v -e ".\LICENSE" -l ".\LICENSE"
cargo wix -I ".\build\windows\main.wxs" -v