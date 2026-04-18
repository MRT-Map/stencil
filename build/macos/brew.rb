cask "stencil3" do
  version "3.0.0-alpha.0"
  sha256 "0e8d849576f1ffb5a88237375a7b6cd198de81e25aa8a34fe84ec78ee46fe1dc"

  url "https://github.com/MRT-Map/stencil3/releases/download/v#{version}/stencil3.dmg"
  name "stencil3"
  desc "Map editor for MRT Map data"
  homepage "https://github.com/MRT-Map/stencil3"

  app "stencil3.app"
  binary "#{appdir}/stencil3.app/Contents/MacOS/stencil3"

  zap trash: [
    "~/Library/Application Support/stencil3",
    "~/Library/Caches/stencil3",
  ]
end
