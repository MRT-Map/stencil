cask "stencil3" do
  version "2.2.12"
  sha256 "a8f9d424c9b820adcb7587a817a7ebde0dc26336a3c4c99a73711327f7b36a03"

  url "https://github.com/MRT-Map/stencil2/releases/download/v#{version}/stencil3.dmg"
  name "stencil3"
  desc "Map editor for MRT Map data"
  homepage "https://github.com/MRT-Map/stencil2"

  app "stencil3.app"
  binary "#{appdir}/stencil3.app/Contents/MacOS/stencil3"

  zap trash: [
    "~/Library/Application Support/stencil3",
    "~/Library/Caches/stencil3",
  ]
end
