//! `gk fetch`: download kernel trees and toolchain tarballs into the cache, and check them.
//!
//! A kernel tarball is held to its pin, and the pin is held to kernel.org's signed `sha256sums.asc`, checked against `keys/kernel.org.asc`. A GCC or binutils tarball is held to its pin and to its detached signature, checked against `keys/gnu.asc`. A tarball already in the cache is only hashed again.

use crate::gpg::Keyring;
use crate::{kernelorg, net};
use gk_model::repo::Repo;
use std::path::{Path, PathBuf};

/// What to fetch.
#[derive(Debug, Default)]
pub struct Request {
    /// Kernel versions.
    pub kernels: Vec<String>,
    /// GCC ids or versions.
    pub gccs: Vec<String>,
    /// Binutils ids or versions.
    pub binutils: Vec<String>,
    /// Everything that is pinned.
    pub all: bool,
}

/// The cache: `GK_CACHE`, or `~/.cache/gk`.
#[must_use]
pub fn cache_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("GK_CACHE") {
        return PathBuf::from(dir);
    }
    let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
    home.join(".cache/gk")
}

/// Run `gk fetch`.
pub fn run(repo: &Repo, req: &Request) -> Result<(), String> {
    let cache = cache_dir();
    let mut kernels: Vec<&gk_model::kernels::Kernel> = Vec::new();
    for v in &req.kernels {
        let version = v.parse().map_err(|e| format!("kernel {v}: {e:?}"))?;
        kernels.push(
            repo.kernels
                .get(&version)
                .ok_or_else(|| format!("kernel {v} is not in kernels.toml"))?,
        );
    }
    let mut tarballs: Vec<(String, String, String)> = Vec::new();
    for g in &req.gccs {
        let gcc = repo
            .gccs
            .gccs
            .iter()
            .find(|c| c.id == *g || c.version.as_str() == g)
            .ok_or_else(|| format!("gcc {g} is not in gccs.toml"))?;
        tarballs.push((gcc.id.clone(), gcc.url.clone(), gcc.sha256.clone()));
    }
    for b in &req.binutils {
        let bu = repo
            .binutils
            .releases
            .iter()
            .find(|r| r.id == *b || r.version.as_str() == b)
            .ok_or_else(|| format!("binutils {b} is not in binutils.toml"))?;
        tarballs.push((bu.id.clone(), bu.url.clone(), bu.sha256.clone()));
    }
    if req.all {
        kernels = repo.kernels.kernels.iter().collect();
        tarballs = repo
            .gccs
            .gccs
            .iter()
            .filter(|g| !g.url.is_empty())
            .map(|g| (g.id.clone(), g.url.clone(), g.sha256.clone()))
            .chain(
                repo.binutils
                    .releases
                    .iter()
                    .map(|b| (b.id.clone(), b.url.clone(), b.sha256.clone())),
            )
            .collect();
    }
    if kernels.is_empty() && tarballs.is_empty() {
        return Err("nothing to fetch: give --kernel, --gcc, --binutils or --all".to_owned());
    }
    if !kernels.is_empty() {
        let keys = Keyring::open(&repo.root.join("keys/kernel.org.asc"), &cache)?;
        let mut sums: std::collections::BTreeMap<
            String,
            std::collections::BTreeMap<String, String>,
        > = std::collections::BTreeMap::new();
        for k in kernels {
            let path = cache.join("kernels").join(k.file_name());
            if held(&path, &k.sha256)? {
                println!("linux {}: cached", k.version);
                continue;
            }
            let dir = k.url.rsplit_once('/').map_or("", |(d, _)| d).to_owned();
            if !sums.contains_key(&dir) {
                let asc = cache.join("kernels").join(format!(
                    "sha256sums-{}.asc",
                    dir.rsplit('/').next().unwrap_or("dir")
                ));
                net::download(&format!("{dir}/sha256sums.asc"), &asc)?;
                let (text, _) = keys.verify_clear(&asc)?;
                sums.insert(dir.clone(), kernelorg::parse_sums(&text));
            }
            let signed = sums[&dir].get(k.file_name());
            if signed != Some(&k.sha256) {
                return Err(format!(
                    "linux {}: the pin does not match kernel.org's signed sha256sums.asc",
                    k.version
                ));
            }
            fetch_one(&k.url, &path, &k.sha256)?;
            println!("linux {}: fetched, pin matches the signed list", k.version);
        }
    }
    if !tarballs.is_empty() {
        let keys = Keyring::open(&repo.root.join("keys/gnu.asc"), &cache)?;
        for (id, url, sha256) in tarballs {
            let name = url.rsplit('/').next().unwrap_or("download");
            let path = cache.join("toolchains").join(name);
            if held(&path, &sha256)? {
                println!("{id}: cached");
                continue;
            }
            let sig = path.with_file_name(format!("{name}.sig"));
            net::download(&format!("{url}.sig"), &sig)?;
            fetch_one(&url, &path, &sha256)?;
            let fpr = keys.verify_detached(&sig, &path)?;
            println!("{id}: fetched, signed by {fpr}");
        }
    }
    Ok(())
}

/// Whether the cache already has a file with this hash. A file with another hash is removed.
fn held(path: &Path, sha256: &str) -> Result<bool, String> {
    if !path.is_file() {
        return Ok(false);
    }
    if net::sha256_file(path)? == sha256 {
        return Ok(true);
    }
    std::fs::remove_file(path).map_err(|e| format!("removing {}: {e}", path.display()))?;
    Ok(false)
}

fn fetch_one(url: &str, path: &Path, sha256: &str) -> Result<(), String> {
    net::download(url, path)?;
    let got = net::sha256_file(path)?;
    if got != sha256 {
        let _ = std::fs::remove_file(path);
        return Err(format!("{url}: SHA-256 is {got}, the pin says {sha256}"));
    }
    Ok(())
}
