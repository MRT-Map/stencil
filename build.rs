use std::{fs::File, io::Write, path::PathBuf};

use anyhow::Result;
use license_retriever::{Config, LicenseRetriever};
use zip::{write::FileOptions, ZipWriter};

macro_rules! p {
    ($($tt:tt)+) => {
        println!("cargo:warning={}", format!($($tt)+))
    }
}

fn gather_licenses() -> Result<()> {
    let config = Config::default()
        .override_license_text(
            "widestring",
            ["LICENSES/Apache-2.0.txt", "LICENSES/MIT.txt"],
        )
        .override_license_text("half", ["LICENSES/Apache-2.0.txt", "LICENSES/MIT.txt"])
        .override_license_text("stencil2", [include_str!("LICENSE")]);
    LicenseRetriever::from_config(&config)?.save_in_out_dir("licenses")?;
    Ok(())
}

fn zip_assets() -> Result<()> {
    let buf = File::create(PathBuf::try_from(std::env::var("OUT_DIR")?)?.join("assets.zip"))?;
    let mut zip_file = ZipWriter::new(buf);
    let options = FileOptions::default();
    for file in PathBuf::try_from(std::env::var("CARGO_MANIFEST_DIR")?)?
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
        embed_resource::compile(
            {
                let mut path = PathBuf::try_from(std::env::var("CARGO_MANIFEST_DIR")?)?;
                path.push("build");
                path.push("windows");
                path.push("icon.rc");
                path
            },
            embed_resource::NONE,
        );
    }
    Ok(())
}

fn inner() -> Result<()> {
    gather_licenses()?;
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
                p!("Backtrace: {:?}", a.backtrace());
                a
            })
            .unwrap()
    }) {
        p!("Error: {e:#?}");
        panic!()
    }
}
