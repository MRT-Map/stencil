cask "stencil3" do
  version "3.0.0-alpha.1"
  sha256 "19e3e5be64770777c5c0c4ba1840b1a350cadc0347f147ece5212b170bd2cd7e"

  url "https://github.com/MRT-Map/stencil3/releases/download/v#{version}/stencil3-#{version}.dmg"
  name "stencil3"
  desc "Map editor for MRT Map data"
  homepage "https://github.com/MRT-Map/stencil3"

  app "stencil3.app"
  binary "#{appdir}/stencil3.app/Contents/MacOS/stencil3"

  zap trash: [
    "~/Library/Application Support/io.github.mrt-map.stencil3",
    "~/Library/Caches/io.github.mrt-map.stencil3",
    "~/Library/Preferences/io.github.mrt-map.stencil3",
  ]
end
