cask "stencil3" do
  version "3.0.0-alpha.10"
  sha256 "7137a6f5e293f9c65dda06d35c163364d4c765377fcbcf13d0f6386b4b1a02f5"

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
