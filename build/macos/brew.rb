cask "stencil2" do
  version "2.2.9"
  sha256 "d971690c4dfe3c2019ae49252190c7bf83db58f78db33c93160bee3299fd5aa7"

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
