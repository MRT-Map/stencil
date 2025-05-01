cask "stencil2" do
  version "2.2.6"
  sha256 "1b6542714e5229072a06b70815276d62cca0d112b3349dd207e8af56dd67fe1c"

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
