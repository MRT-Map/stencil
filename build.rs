use std::{fs::File, io::Write, path::PathBuf};

use eyre::Result;
use license_retriever::{Config, LicenseRetriever};
use zip::{write::SimpleFileOptions, ZipWriter};

macro_rules! p {
    ($($tt:tt)+) => {
        println!("cargo:warning={}", format!($($tt)+))
    }
}

fn gather_licenses() -> Result<()> {
    let config = Config::default()
        .panic_if_no_license_found()
        .override_license_url(
            "accesskit",
            [
                "https://raw.githubusercontent.com/AccessKit/accesskit/main/LICENSE-APACHE",
                "https://raw.githubusercontent.com/AccessKit/accesskit/main/LICENSE-MIT",
                "https://raw.githubusercontent.com/AccessKit/accesskit/main/LICENSE.chromium",
            ],
        )
        .copy_license("accesskit_consumer", "accesskit")
        .copy_license("accesskit_macos", "accesskit")
        .copy_license("accesskit_windows", "accesskit")
        .copy_license("accesskit_winit", "accesskit")
        .copy_license("bevy_mouse_tracking_plugin", "block")
        .copy_license("bevy_eventlistener_derive", "bevy_eventlistener")
        .override_license_url(
            "bevy-inspector-egui",
            ["https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/main/LICENSE-MIT.md",
                    "https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/main/LICENSE-APACHE.md"],
        )
        .override_license_url(
            "bevy-inspector-egui-derive",
            ["https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/main/LICENSE-MIT.md",
                    "https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/main/LICENSE-APACHE.md"],
        )
        .copy_license("bevy_picking_core", "bevy_mod_picking")
        .copy_license("bevy_picking_input", "bevy_mod_picking")
        .copy_license("bevy_picking_raycast", "bevy_mod_picking")
        .copy_license("bevy_picking_highlight", "bevy_mod_picking")
        .copy_license("bevy_picking_selection", "bevy_mod_picking")
        .override_license_url(
            "block",
            ["https://raw.githubusercontent.com/spdx/license-list-data/main/text/MIT.txt"],
        )
        .override_license_url(
            "block2",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
        .override_license_url(
            "block-sys",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
        .override_license_url(
            "cesu8",
            ["https://raw.githubusercontent.com/emk/cesu8-rs/master/COPYRIGHT-RUST.txt"],
        )
        .override_license_url(
            "clipboard-win",
            ["https://raw.githubusercontent.com/DoumanAsh/clipboard-win/master/LICENSE"],
        )
        .override_license_url(
            "codespan-reporting",
            ["https://raw.githubusercontent.com/brendanzab/codespan/master/LICENSE"],
        )
        .override_license_url(
            "color-spantrace",
            [
                "https://raw.githubusercontent.com/yaahc/color-spantrace/master/LICENSE-APACHE",
                "https://raw.githubusercontent.com/yaahc/color-spantrace/master/LICENSE-MIT",
            ],
        )
        .override_license_url(
            "com_macros",
            ["https://raw.githubusercontent.com/microsoft/com-rs/master/LICENSE"],
        )
        .copy_license("com_macros_support", "com_macros")
        .override_license_url(
            "constgebra",
            [
                "https://raw.githubusercontent.com/spdx/license-list-data/main/text/MIT.txt",
                "https://raw.githubusercontent.com/spdx/license-list-data/main/text/Apache-2.0.txt",
            ],
        )
        .override_license_url(
            "core-graphics-types",
            [
                "https://raw.githubusercontent.com/servo/core-foundation-rs/master/LICENSE-MIT",
                "https://raw.githubusercontent.com/servo/core-foundation-rs/master/LICENSE-APACHE",
            ],
        )
        .copy_license("crunchy", "block")
        .override_license_url(
            "d3d12",
            [
                "https://raw.githubusercontent.com/spdx/license-list-data/main/text/MIT.txt",
                "https://raw.githubusercontent.com/spdx/license-list-data/main/text/Apache-2.0.txt",
            ],
        )
        .copy_license("dispatch", "block")
        .copy_license("ecolor", "egui")
        .override_license_url(
            "egui",
            [
                "https://raw.githubusercontent.com/emilk/egui/master/LICENSE-APACHE",
                "https://raw.githubusercontent.com/emilk/egui/master/LICENSE-MIT",
            ],
        )
        .copy_license("egui_extras", "egui")
        .copy_license("emath", "egui")
        .copy_license("encase_derive", "encase")
        .copy_license("encase_derive_impl", "encase")
        .copy_license("epaint", "egui")
        .override_license_url(
            "error-code",
            ["https://github.com/DoumanAsh/error-code/blob/master/LICENSE"],
        )
        .copy_license("fdeflate", "d3d12")
        .override_license_url(
            "gl_generator",
            ["https://raw.githubusercontent.com/brendanzab/gl-rs/master/LICENSE"],
        )
        .override_license_url(
            "gloo-timers",
            [
                "https://raw.githubusercontent.com/rustwasm/gloo/master/LICENSE-MIT",
                "https://raw.githubusercontent.com/rustwasm/gloo/master/LICENSE-APACHE",
            ],
        )
        .override_license_url(
            "gpu-alloc",
            ["https://github.com/zakarumych/gpu-alloc/blob/master/COPYING"],
        )
        .copy_license("gpu-alloc-types", "gpu-alloc")
        .override_license_url(
            "gpu-descriptor",
            ["https://github.com/zakarumych/gpu-descriptor/blob/master/COPYING"],
        )
        .copy_license("gpu-descriptor-types", "gpu-descriptor")
        .override_license_url(
            "half",
            [
                "https://docs.rs/crate/half/latest/source/LICENSES/MIT.txt",
                "https://docs.rs/crate/half/latest/source/LICENSES/Apache-2.0.txt",
            ],
        )
        .override_license_url(
            "hassle-rs",
            ["https://raw.githubusercontent.com/Traverse-Research/hassle-rs/main/LICENSE"],
        )
        .override_license_url(
            "hexf-parse",
            ["https://raw.githubusercontent.com/spdx/license-list-data/main/text/CC0-1.0.txt"],
        )
        .copy_license("icrate", "block2")
        .override_license_url("indenter", [
            "https://raw.githubusercontent.com/eyre-rs/indenter/master/LICENSE-MIT",
            "https://raw.githubusercontent.com/eyre-rs/indenter/master/LICENSE-APACHE"
        ])
        .override_license_url(
            "khronos_api",
            ["https://github.com/brendanzab/gl-rs/blob/master/LICENSE"],
        )
        .copy_license("lazy-regex-proc_macros", "lazy-regex")
        .ignore("license-retriever")
        .override_license_url(
            "lyon_algorithms",
            [
                "https://raw.githubusercontent.com/nical/lyon/master/LICENSE-APACHE",
                "https://raw.githubusercontent.com/nical/lyon/master/LICENSE-MIT",
            ],
        )
        .copy_license("lyon_geom", "lyon_algorithms")
        .copy_license("lyon_path", "lyon_algorithms")
        .copy_license("lyon_tessellation", "lyon_algorithms")
        .override_license_url(
            "malloc_buf",
            ["https://raw.githubusercontent.com/SSheldon/malloc_buf/master/LICENSE"],
        )
        .copy_license("naga", "wgpu")
        .override_license_url(
            "ndk",
            [
                "https://raw.githubusercontent.com/rust-mobile/ndk/master/LICENSE-MIT",
                "https://raw.githubusercontent.com/rust-mobile/ndk/master/LICENSE-APACHE",
            ],
        )
        .copy_license("ndk-context", "ndk")
        .copy_license("ndk-sys", "ndk")
        .override_license_url(
            "objc2-sys",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
        .override_license_url(
            "objc-sys",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
        .override_license_url(
            "objc2",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
         .override_license_url(
            "objc2-app-kit",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
         .override_license_url(
            "objc2-core-data",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
         .override_license_url(
            "objc2-core-image",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
        .override_license_url(
            "objc2-encode",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
         .override_license_url(
            "objc2-foundation",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
         .override_license_url(
            "objc2-metal",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
         .override_license_url(
            "objc2-quartz-core",
            ["https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE.txt"],
        )
        .copy_license("objc_exception", "block")
        .override_license_url(
            "profiling",
            [
                "https://raw.githubusercontent.com/aclysma/profiling/master/LICENSE-APACHE",
                "https://raw.githubusercontent.com/aclysma/profiling/master/LICENSE-MIT",
            ],
        )
        .copy_license("profiling-procmacros", "profiling")
        .override_license_url(
            "pulldown-cmark",
            ["https://raw.githubusercontent.com/pulldown-cmark/pulldown-cmark/master/LICENSE"],
        )
        .override_license_url(
            "simd_helpers",
            ["https://raw.githubusercontent.com/lu-zero/simd_helpers/master/LICENSE"],
        )
        .override_license_url(
            "siphasher",
            ["https://raw.githubusercontent.com/jedisct1/rust-siphash/master/COPYING"],
        )
        .override_license_url(
            "spirv",
            ["https://raw.githubusercontent.com/gfx-rs/rspirv/master/LICENSE"],
        )
        .copy_license("stdweb-derive", "stdweb")
        .copy_license("stdweb-internal-macros", "stdweb")
        .copy_license("stdweb-internal-runtime", "stdweb")
        .override_license_url(
            "str-buf",
            ["https://raw.githubusercontent.com/DoumanAsh/str-buf/master/LICENSE"],
        )
        .override_license_url(
            "svg_fmt",
            ["https://raw.githubusercontent.com/nical/rust_debug/master/LICENSE"],
        )
        .override_license_url(
            "taffy",
            ["https://raw.githubusercontent.com/DioxusLabs/taffy/main/LICENSE.md"],
        )
        .override_license_url(
            "valuable",
            ["https://raw.githubusercontent.com/tokio-rs/valuable/master/LICENSE"],
        )
        .override_license_url(
            "wayland-protocols-plasma",
            ["https://raw.githubusercontent.com/Smithay/wayland-rs/master/LICENSE.txt"],
        )
        .copy_license("wayland-protocols-wlr", "wayland-protocols-plasma")
        .override_license_url(
            "widestring",
            [
                "https://docs.rs/crate/widestring/latest/source/LICENSES/Apache-2.0.txt",
                "https://docs.rs/crate/widestring/latest/source/LICENSES/MIT.txt",
            ],
        )
        .copy_license("winapi-i686-pc-windows-gnu", "winapi")
        .copy_license("winapi-x86_64-pc-windows-gnu", "winapi")
        .override_license_url(
            "zune-inflate",
            ["https://raw.githubusercontent.com/etemesi254/zune-image/main/LICENSE.md"],
        )
        .override_license_url(
            "xi-unicode",
            ["https://github.com/xi-editor/xi-editor/blob/master/LICENSE"],
        )
        .override_license_url(
            "zune-core",
            [
                "https://raw.githubusercontent.com/etemesi254/zune-image/dev/LICENSE.md",
                "https://raw.githubusercontent.com/etemesi254/zune-image/dev/LICENSE-ZLIB",
            ],
        )
        .copy_license("zune-jpeg", "zune-core")
        .override_license_url(
            "zopfli",
            ["https://raw.githubusercontent.com/zopfli-rs/zopfli/main/COPYING"],
        )
        .override_license_text("stencil2", [include_str!("LICENSE")]);
    LicenseRetriever::from_config(&config)?.save_in_out_dir("licenses")?;
    Ok(())
}

fn zip_assets() -> Result<()> {
    let buf = File::create(PathBuf::from(std::env::var("OUT_DIR")?).join("assets.zip"))?;
    let mut zip_file = ZipWriter::new(buf);
    let options = SimpleFileOptions::default();
    for file in PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?)
        .join("assets")
        .read_dir()?
    {
        let file = file?.path();
        zip_file.start_file(file.file_name().unwrap().to_string_lossy(), options)?;
        let contents = std::fs::read(file)?;
        zip_file.write_all(&contents)?;
    }
    zip_file.finish()?;
    Ok(())
}

fn embed_resource() -> Result<()> {
    if std::env::var("TARGET")?.contains("windows") {
        let root_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
        let icons_dir = root_dir.join("icons");
        std::fs::copy(icons_dir.join("icon.rc"), root_dir.join("icon.rc"))?;
        std::fs::copy(icons_dir.join("icon.ico"), root_dir.join("icon.ico"))?;
        embed_resource::compile("icon.rc", embed_resource::NONE);
        std::fs::remove_file(root_dir.join("icon.rc"))?;
        std::fs::remove_file(root_dir.join("icon.ico"))?;
    }
    Ok(())
}

fn inner() -> Result<()> {
    if std::env::var("PROFILE")? != "debug" {
        gather_licenses()?;
    }
    zip_assets()?;
    embed_resource()?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets");

    Ok(())
}

fn main() {
    if let Err(e) = std::panic::catch_unwind(|| {
        inner()
            .map_err(|a| {
                p!("Backtrace: {:?}", a);
                a
            })
            .unwrap()
    }) {
        p!("Error: {e:#?}");
        panic!()
    }
}
