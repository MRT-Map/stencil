cask "stencil3" do
  version "3.0.0-alpha.7"
  sha256 "e89a758017e909052a8ab241a31ae7d4740e8398de47bdf936b54ca0628e5061"

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
