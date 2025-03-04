use core::panic;
use std::{
    fs,
    io::{self, Write},
};

#[derive(Debug)]
enum SlangOs {
    Linux,
    Windows,
    MacOs,
}

impl SlangOs {
    fn from_str(s: &str) -> Self {
        match s {
            "linux" => SlangOs::Linux,
            "windows" => SlangOs::Windows,
            "darwin" => SlangOs::MacOs,
            _ => panic!("Unknown OS: {}", s),
        }
    }

    fn to_str(&self) -> &str {
        match self {
            SlangOs::Linux => "linux",
            SlangOs::Windows => "windows",
            SlangOs::MacOs => "macos",
        }
    }
}

enum SlangArch {
    X86_64,
    Aarch64,
}

impl SlangArch {
    fn from_str(s: &str) -> Self {
        match s {
            "x86_64" => SlangArch::X86_64,
            "aarch64" => SlangArch::Aarch64,
            _ => panic!("Unknown arch: {}", s),
        }
    }

    fn to_str(&self) -> &str {
        match self {
            SlangArch::X86_64 => "x86_64",
            SlangArch::Aarch64 => "aarch64",
        }
    }
}

fn main() {
    #[cfg(any(feature = "use-curl", feature = "use-reqwest"))]
    let slang_folder = download_slang();

    #[cfg(feature = "use-vulkan-sdk")]
    let slang_folder = std::env::var("VULKAN_SDK").unwrap();

    println!(
        "cargo:rustc-env=SLANGC_BIN_PATH={}/bin/slangc",
        slang_folder
    );    
    println!("cargo:rustc-link-search=native={}/lib", slang_folder);
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(any(feature = "use-curl", feature = "use-reqwest"))]
fn download_slang() -> String {
    let target = std::env::var("TARGET").unwrap();
    let parts = target.split('-').collect::<Vec<_>>();
    let arch = SlangArch::from_str(parts[0]);
    let os = SlangOs::from_str(parts[2]);

    let str = String::from_utf8(req(
        "https://api.github.com/repos/shader-slang/slang/releases/latest",
    ))
    .unwrap();
    let Ok(root): Result<serde::GithubReleaseOverviewItem, serde_json::Error> =
        serde_json::from_str(&str)
    else {
        panic!(
            "Failed to deserialize from json, the str input was: {}",
            str
        );
    };

    let to_find = format!("{}-{}.zip", os.to_str(), arch.to_str());
    // panic!("Looking for: {}", to_find);
    let asset = root
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(&to_find))
        .expect("Failed to find release asset with supported arch and os combination");

    let download_url = &asset.browser_download_url;

    let out_path = std::env::var("OUT_DIR").unwrap();
    let out_path = std::path::Path::new(&out_path).join("slang.zip");

    {
        let mut file = std::fs::File::create(&out_path).unwrap();
        let bytes = req(&download_url);
        file.write_all(&bytes).unwrap();
    }

    // lets unzip
    let target_dir = std::path::Path::new(&out_path)
        .parent()
        .unwrap()
        .join("slang");
    fs::create_dir_all(&target_dir).unwrap();

    let mut archive = zip::ZipArchive::new(std::fs::File::open(&out_path).unwrap()).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    // let bin_folder = target_dir.join("bin/slangc");
    return target_dir.display().to_string();
}

#[cfg(feature = "use-curl")]
fn req(url: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut easy = curl::easy::Easy::new();
        easy.url(url).unwrap();
        easy.follow_location(true).unwrap();
        // required by GitHub API
        easy.useragent("Rust Build Script (slang-cli-rs)").unwrap();

        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                bytes.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    assert!(!bytes.is_empty());
    bytes
}

#[cfg(feature = "use-reqwest")]
fn req(url: &str) -> Vec<u8> {
    let res = reqwest::blocking::get(url).unwrap();
    assert!(res.status().is_success());
    res.bytes().unwrap().to_vec()
}

#[cfg(any(feature = "use-curl", feature = "use-reqwest"))]
mod serde {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GithubReleaseOverviewItem {
        pub assets: Vec<Asset>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Asset {
        pub id: i64,
        pub name: String,
        #[serde(rename = "browser_download_url")]
        pub browser_download_url: String,
    }
}
