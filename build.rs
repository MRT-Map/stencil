use std::{fs::File, io::Write, path::PathBuf};

use eyre::Result;
use license_retriever::{Config, LicenseRetriever};
use zip::{ZipWriter, write::SimpleFileOptions};

fn gather_licenses() -> Result<()> {
    let config = Config {
        error_for_no_license: true,
        ..Config::default()
    };
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
        let _ = embed_resource::compile("icon.rc", embed_resource::NONE);
        std::fs::remove_file(root_dir.join("icon.rc"))?;
        std::fs::remove_file(root_dir.join("icon.ico"))?;
    }
    Ok(())
}

fn main() -> Result<()> {
    if std::env::var("PROFILE")? != "debug" {
        gather_licenses()?;
    }
    zip_assets()?;
    embed_resource()?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets");

    Ok(())
}
