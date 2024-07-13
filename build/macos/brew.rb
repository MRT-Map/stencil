cask "stencil2" do
  version "2.2.1"
  sha256 "308cf73c3fc66a8765e37f75347e3a98bd78eb715e88e5e0f56adb4d4dd6f352"

  url "https://github.com/MRT-Map/stencil2/releases/download/v#{version}/stencil2.dmg"
  name "stencil2"
  desc "Map editor for MRT Map data"
  homepage "https://github.com/MRT-Map/stencil2"

  app "stencil2.app"
  binary "#{appdir}/stencil2.app/Contents/MacOS/stencil2"

  zap trash: [
    "~/Library/Application Support/stencil2",
    "~/Library/Caches/stencil2",
  ]
end
