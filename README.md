# stencil2

![image](https://user-images.githubusercontent.com/61975820/197353273-73204a75-7ee3-410c-9d96-3a1d77fd8786.png)


The better stencil. Map editor for MRT Map data

Still WIP!

## Installation
* Command line: (`<version>` is the version number (with `v`) and `<os>` is one of `windows`, `macos`, `ubuntu`)
  * Windows Powershell: `Invoke-WebRequest -Uri "https://github.com/MRT-Map/stencil2/releases/download/<version>/stencil2-<os>" -OutFile "stencil2.exe"`
  * Mac / Linux: `curl "https://github.com/MRT-Map/stencil2/releases/download/<version>/stencil2-<os>" -Lo stencil2` (needs curl)
* Cargo: `cargo install --git https://github.com/MRT-Map/stencil2` (omit `./` from this step onwards in this case)
* As an executable (see the releases for downloads for windows, mac and ubuntu) (save it as `stencil2` / `stencil2.exe`), you would then have to navigate in the command line to the same directory as where you downloaded the executable to
* For mac/linux you may have to `chmod +x ./stencil2`, unless you downloaded it via Cargo

## Installation
* **Universal**
  * Cargo
    * Run `cargo toolchain install nightly` if the nightly compiler isn't installed
    * Run `cargo +nightly install --git https://github.com/MRT-Map/stencil2`
    * Launch stencil2 with `stencil2`
* **Windows**
  * Command Line (Invoke-WebRequest)
    * Run `Invoke-WebRequest -Uri "https://github.com/MRT-Map/stencil2/releases/download/<version>/stencil2-windows" -OutFile "stencil2.exe"`
      * where `<version>` is the version number (with `v`) (eg `v2.0.0`)
    * Launch Stencil from File Explorer
  * msi (soon)
  * scoop, chocolatey etc (soon)
* **MacOS**
  * As .dmg
    * Download the .dmg file in the assets folder of the latest GitHub release
    * Open the .dmg file and drag the application into the folder
    * Launch stencil2
  * Command Line (curl)
    * Run `curl "https://github.com/MRT-Map/stencil2/releases/download/<version>/stencil2-macos" -Lo stencil2`
      * where `<version>` is the version number (with `v`) (eg `v2.0.0`)
    * Run `chmod +x stencil2`
    * Launch stencil2 with `./stencil2`
  * brew (soon)
* **Linux**
  * Command Line (curl)
    * Run `curl "https://github.com/MRT-Map/stencil2/releases/download/<version>/stencil2-ubuntu" -Lo stencil2`
      * where `<version>` is the version number (with `v`) (eg `v2.0.0`)
    * Run `chmod +x stencil2`
    * Launch stencil2 with `./stencil2`
  * snap, .deb, flatpak, appimage etc (soon)
