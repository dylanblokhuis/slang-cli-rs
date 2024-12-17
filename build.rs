use core::panic;
use std::{
    fs,
    io::{self, Write},
};

use curl::easy::Easy;

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

    let bin_folder = target_dir.join("bin/slangc");
    println!("cargo:rustc-env=SLANGC_BIN_PATH={}", bin_folder.display());
    println!("cargo:rerun-if-changed=build.rs");
}

fn req(url: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut easy = Easy::new();
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

mod serde {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GithubReleaseOverviewItem {
        pub url: String,
        #[serde(rename = "assets_url")]
        pub assets_url: String,
        #[serde(rename = "upload_url")]
        pub upload_url: String,
        #[serde(rename = "html_url")]
        pub html_url: String,
        pub id: i64,
        pub author: Author,
        #[serde(rename = "node_id")]
        pub node_id: String,
        #[serde(rename = "tag_name")]
        pub tag_name: String,
        #[serde(rename = "target_commitish")]
        pub target_commitish: String,
        pub name: String,
        pub draft: bool,
        pub prerelease: bool,
        #[serde(rename = "created_at")]
        pub created_at: String,
        #[serde(rename = "published_at")]
        pub published_at: String,
        pub assets: Vec<Asset>,
        #[serde(rename = "tarball_url")]
        pub tarball_url: String,
        #[serde(rename = "zipball_url")]
        pub zipball_url: String,
        pub body: String,
        pub reactions: Reactions,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Author {
        pub login: String,
        pub id: i64,
        #[serde(rename = "node_id")]
        pub node_id: String,
        #[serde(rename = "avatar_url")]
        pub avatar_url: String,
        #[serde(rename = "gravatar_id")]
        pub gravatar_id: String,
        pub url: String,
        #[serde(rename = "html_url")]
        pub html_url: String,
        #[serde(rename = "followers_url")]
        pub followers_url: String,
        #[serde(rename = "following_url")]
        pub following_url: String,
        #[serde(rename = "gists_url")]
        pub gists_url: String,
        #[serde(rename = "starred_url")]
        pub starred_url: String,
        #[serde(rename = "subscriptions_url")]
        pub subscriptions_url: String,
        #[serde(rename = "organizations_url")]
        pub organizations_url: String,
        #[serde(rename = "repos_url")]
        pub repos_url: String,
        #[serde(rename = "events_url")]
        pub events_url: String,
        #[serde(rename = "received_events_url")]
        pub received_events_url: String,
        #[serde(rename = "type")]
        pub type_field: String,
        #[serde(rename = "user_view_type")]
        pub user_view_type: String,
        #[serde(rename = "site_admin")]
        pub site_admin: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Asset {
        pub url: String,
        pub id: i64,
        #[serde(rename = "node_id")]
        pub node_id: String,
        pub name: String,
        pub label: String,
        pub uploader: Uploader,
        #[serde(rename = "content_type")]
        pub content_type: String,
        pub state: String,
        pub size: i64,
        #[serde(rename = "download_count")]
        pub download_count: i64,
        #[serde(rename = "created_at")]
        pub created_at: String,
        #[serde(rename = "updated_at")]
        pub updated_at: String,
        #[serde(rename = "browser_download_url")]
        pub browser_download_url: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Uploader {
        pub login: String,
        pub id: i64,
        #[serde(rename = "node_id")]
        pub node_id: String,
        #[serde(rename = "avatar_url")]
        pub avatar_url: String,
        #[serde(rename = "gravatar_id")]
        pub gravatar_id: String,
        pub url: String,
        #[serde(rename = "html_url")]
        pub html_url: String,
        #[serde(rename = "followers_url")]
        pub followers_url: String,
        #[serde(rename = "following_url")]
        pub following_url: String,
        #[serde(rename = "gists_url")]
        pub gists_url: String,
        #[serde(rename = "starred_url")]
        pub starred_url: String,
        #[serde(rename = "subscriptions_url")]
        pub subscriptions_url: String,
        #[serde(rename = "organizations_url")]
        pub organizations_url: String,
        #[serde(rename = "repos_url")]
        pub repos_url: String,
        #[serde(rename = "events_url")]
        pub events_url: String,
        #[serde(rename = "received_events_url")]
        pub received_events_url: String,
        #[serde(rename = "type")]
        pub type_field: String,
        #[serde(rename = "user_view_type")]
        pub user_view_type: String,
        #[serde(rename = "site_admin")]
        pub site_admin: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Reactions {
        pub url: String,
        #[serde(rename = "total_count")]
        pub total_count: i64,
        #[serde(rename = "+1")]
        pub n1: i64,
        #[serde(rename = "-1")]
        pub n12: i64,
        pub laugh: i64,
        pub hooray: i64,
        pub confused: i64,
        pub heart: i64,
        pub rocket: i64,
        pub eyes: i64,
    }
}
