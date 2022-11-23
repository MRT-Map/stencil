use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Result};
use futures::{executor::block_on, future::join_all};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

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

fn inner() -> Result<()> {
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
            let res = surf::get(format!("https://docs.rs/crate/{}/{}/source/", a.name, a.version))
                .await.map_err(|e| anyhow!("Error accessing source: {e}"))?
                .body_string()
                .await.map_err(|e| anyhow!("Error parsing as string: {e}"))?;
            let files = if &*a.name == "widestring" {
                vec![
                    "LICENSES/Apache-2.0.txt",
                    "LICENSES/MIT.txt"
                ]
            } else {
                Regex::new(r#"<span class="text">(.*?)</span>"#)?.captures_iter(&res)
                .map(|a| {
                    Ok(a.get(1).ok_or_else(|| anyhow!("Regex is broken"))?.as_str())
                })
                .filter_ok(|a| a.to_lowercase().starts_with("licence")
                || a.to_lowercase().starts_with("license")
                || a.to_lowercase().ends_with("licence")
                || a.to_lowercase().ends_with("license"))
                .collect::<Result<Vec<_>>>()?
            };
            let mut texts = vec![];
            for file in files {
                let res = surf::get(format!("https://docs.rs/crate/{}/{}/source/{}", a.name, a.version, file))
                    .await.map_err(|e| anyhow!("Error accessing source: {e}"))?
                    .body_string()
                    .await.map_err(|e| anyhow!("Error parsing as string: {e}"))?;
                let text = html_escape::decode_html_entities(Regex::new(r#"(?s)<span class="syntax-text (?:syntax-plain|syntax-html syntax-markdown)">(.*?)</span>"#)?
                    .captures(&res)
                    .ok_or_else(|| anyhow!("No text found {} {}", a.name, file))?
                    .get(1).ok_or_else(|| anyhow!("Regex is broken"))?
                    .as_str().trim()).to_string();
                texts.push(text);
            }
            if texts.is_empty() {
                p!("No licenses detected for crate {} {}", a.name, a.version)
            }
            a.license_text = Some(texts);
            Ok(a)
        }))).into_iter().collect::<Result<Vec<_>>>().map_err(|e| anyhow!("Error: {e}"))?;

    std::fs::write(
        {
            let mut path = PathBuf::try_from(std::env::var("OUT_DIR")?)?;
            path.push("licenses.msgpack");
            path
        },
        rmp_serde::to_vec(&data)?,
    )?;

    print!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn main() {
    if let Err(e) = std::panic::catch_unwind(|| {
        if let Err(e) = inner() {
            println!("cargo:warning=Error: {e:?}");
            panic!()
        }
    }) {
        println!("cargo:warning=Error: {e:?}");
        panic!()
    }
}
