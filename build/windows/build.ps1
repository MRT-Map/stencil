$dir = $PSScriptRoot
Set-Location "$dir\..\.."
#Invoke-WebRequest -Uri https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311.exe -OutFile wix311.exe
cargo install cargo-wix
cargo wix -I ".\build\windows\main.wxs" -v