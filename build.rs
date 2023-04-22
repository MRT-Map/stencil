use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Result};
use futures::{executor::block_on, future::join_all};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use zip::{write::FileOptions, ZipWriter};

macro_rules! p {
    ($($tt:tt)+) => {
        println!("cargo:warning={}", format!($($tt)+))
    }
}

#[derive(Deserialize, Serialize)]
pub struct CargoLicenseEntry {
    name: String,
    version: String,
    authors: Option<String>,
    repository: Option<String>,
    license: Option<String>,
    license_file: Option<String>,
    license_text: Option<Vec<String>>,
}

fn gather_licenses() -> Result<()> {
    Command::new("cargo")
        .args(["install", "cargo-license"])
        .spawn()?
        .wait()?;
    let raw = Command::new("cargo")
        .args(["license", "--json"])
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?
        .stdout;
    let data: Vec<CargoLicenseEntry> = serde_json::from_slice(&raw)?;

    let data = block_on(join_all(data.into_iter()
        .map(|mut a| async move {
            if &*a.name == "stencil2" {
                a.license_text = Some(vec![include_str!("LICENSE").to_string()]);
                return Ok(a)
            }
            let mut texts = vec![];
            let mut list = vec![
                (
                    false,
                    format!("https://docs.rs/crate/{}/{}/source/", a.name, a.version),
                    Regex::new(r#"<span class="text">(.*?)</span>"#)?,
                    format!("https://docs.rs/crate/{}/{}/source/", a.name, a.version),
                    Regex::new(r#"(?s)<span class="(?:syntax-text syntax-plain|syntax-text syntax-html syntax-markdown|syntax-source syntax-diff)">(.*?)</span>"#)?
                )
            ];
            if let Some(repository) = &a.repository {
                if repository.starts_with("https://github.com") || repository.starts_with("http://github.com") {
                    list.push((
                        true,
                        repository.to_owned(),
                        Regex::new(r#"title="(.*?)""#)?,
                        format!("{}{}", repository.replace("github.com", "raw.githubusercontent.com"),
                                if repository.ends_with('/') {""} else {"/"}),
                        Regex::new(r#"(?s)(.*)"#)?,
                        ));
                }
            }
            for (is_gh, url1, re1, url2, re2) in list {
                let res = surf::get(url1)
                    .await.map_err(|e| anyhow!("Error accessing source: {e}"))?
                    .body_string()
                    .await.map_err(|e| anyhow!("Error parsing as string: {e}"))?;
                let files = if &*a.name == "widestring" || &*a.name == "half" {
                    vec![
                        "LICENSES/Apache-2.0.txt",
                        "LICENSES/MIT.txt"
                    ]
                } else {
                    re1.captures_iter(&res)
                        .map(|a| {
                            Ok(a.get(1).ok_or_else(|| anyhow!("Regex is broken"))?.as_str())
                        })
                        .filter_ok(|a| a.to_lowercase().starts_with("licence")
                            || a.to_lowercase().starts_with("license")
                            || a.to_lowercase().ends_with("licence")
                            || a.to_lowercase().ends_with("license"))
                        .collect::<Result<Vec<_>>>()?
                };
                let branch = if is_gh {
                    Regex::new("data-menu-button>(.*?)</span>")?.captures(&res)
                        .and_then(|cap| cap.get(1))
                        .map(|m| m.as_str())
                        .unwrap_or_default()
                } else {""};
                for file in files {
                    let res = surf::get(if is_gh {
                        format!("{url2}{branch}/{file}")
                    } else {format!("{url2}{file}")})
                        .await.map_err(|e| anyhow!("Error accessing source: {e}"))?
                        .body_string()
                        .await.map_err(|e| anyhow!("Error parsing as string: {e}"))?;
                    let text = html_escape::decode_html_entities(re2
                        .captures(&res)
                        .ok_or_else(|| anyhow!("No text found {} {}", a.name, file))?
                        .get(1).ok_or_else(|| anyhow!("Regex is broken"))?
                        .as_str().trim()).to_string();
                    texts.push(text);
                }
                if !texts.is_empty() {
                    break
                }
            }
            if texts.is_empty() {
                //p!("No licenses detected for crate {} {}", a.name, a.version)
            }
            a.license_text = Some(texts);
            Ok(a)
        }))).into_iter().collect::<Result<Vec<_>>>().map_err(|e| anyhow!("Error: {e}"))?;

    std::fs::write(
        PathBuf::try_from(std::env::var("OUT_DIR")?)?.join("licenses.msgpack"),
        rmp_serde::to_vec(&data)?,
    )?;
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

fn inner() -> Result<()> {
    gather_licenses()?;
    zip_assets()?;

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

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets");

    Ok(())
}

fn main() {
    if let Err(e) = std::panic::catch_unwind(|| {
        if let Err(e) = inner() {
            p!("Error: {e:#?}");
            p!("{:?}", e.backtrace());
            panic!()
        }
    }) {
        p!("Error: {e:#?}");
        panic!()
    }
}
