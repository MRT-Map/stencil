cask "stencil2" do
  version "2.2.8"
  sha256 "20a686d922753e31709f6c01aa81350b026b0cc6699031b0e703fee33a30a752"

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
