# stencil2

![image](https://user-images.githubusercontent.com/61975820/197353273-73204a75-7ee3-410c-9d96-3a1d77fd8786.png)

The better stencil. Map editor for MRT Map data

## Installation
* **Universal**
  * Cargo
    * Run `cargo toolchain install nightly` if the nightly compiler isn't installed
    * Run `cargo +nightly install --git https://github.com/MRT-Map/stencil2`
    * Launch stencil2 with `stencil2`
* **Windows**
  * Scoop (untested)
    * `scoop install "https://raw.githubusercontent.com/MRT-Map/stencil2/master/build/windows/scoop.json"`
  * msi
    * Download the .msi file in the assets folder of the latest GitHub release
    * Open the .msi file and follow the instructions
    * Launch stencil2 in file explorer in either `[Drive]:\Program Files\stencil2\bin` or `[Drive]:\Program Files (x86)\stencil2\bin`
    * Remember to pin it to start/taskbar if it isn't indexed in the search bar ;)
* **MacOS**
  * dmg
    * Download the .dmg file in the assets folder of the latest GitHub release
    * Open the .dmg file and drag the application into the folder
    * Launch stencil2
  * Homebrew
    * `brew install --cask mrt-map/mrt-map/stencil2`
* **Linux**
  * deb
    * Download the .deb file in the assets folder of the latest GitHub release
    * Extract & install the file... TODO write instructions
  * via PKGBUILD (Arch et al)
    * Create a new folder name and `cd` into it
    * `curl "https://raw.githubusercontent.com/MRT-Map/stencil2/master/build/linux/PKGBUILD" -Lo PKGBUILD`
    * `makepkg -si
