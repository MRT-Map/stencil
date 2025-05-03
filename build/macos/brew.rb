cask "stencil2" do
  version "2.2.7"
  sha256 "c362ced334dab4e9ee49ef66eb169c3c06bb1da26e61e80ab9cdd2d6c2d78f91"

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
