cask "stencil3" do
  version "3.0.0-alpha.2"
  sha256 "7698126092aece35ff49333301769b7c9d60100c8a79d8b542e19b83c67a2d30"

  url "https://github.com/MRT-Map/stencil3/releases/download/v#{version}/stencil3-#{version}.dmg"
  name "stencil3"
  desc "Map editor for MRT Map data"
  homepage "https://github.com/MRT-Map/stencil3"

  depends_on :macos

  app "stencil3.app"
  binary "#{appdir}/stencil3.app/Contents/MacOS/stencil3"

  zap trash: [
    "~/Library/Application Support/io.github.mrt-map.stencil3",
    "~/Library/Caches/io.github.mrt-map.stencil3",
    "~/Library/Preferences/io.github.mrt-map.stencil3",
  ]
end
