cask "stencil3" do
  version "3.0.0-alpha.9"
  sha256 "104158d9178c9416c669f6d0cf673adcc01474c1dc2eabc8b0ec372fe7439ca4"

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
