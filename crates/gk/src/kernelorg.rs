//! What kernel.org publishes: the list of maintained lines and the SHA-256 lists.
//!
//! `releases.json` names the mainline rc, the stable line and every longterm line with its latest point release. `sha256sums.asc` in each major directory lists the hash of every tarball there. The list is signed. `gk fetch` checks the signature, and the hash in `kernels.toml` is what every fetch is held to.

use serde::Deserialize;
use std::collections::BTreeMap;

/// Where `releases.json` lives.
pub const RELEASES_URL: &str = "https://www.kernel.org/releases.json";

/// The base of every tarball URL.
pub const CDN: &str = "https://cdn.kernel.org/pub/linux/kernel";

/// One entry of `releases.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    /// `mainline`, `stable`, `longterm` or `linux-next`.
    pub moniker: String,
    /// The version, as in `7.2.8` or `7.3-rc5`.
    pub version: String,
    /// Whether kernel.org has marked the line end of life.
    #[serde(default)]
    pub iseol: bool,
}

#[derive(Deserialize)]
struct ReleasesFile {
    releases: Vec<Release>,
}

/// Parse the text of `releases.json`.
pub fn parse_releases(text: &str) -> Result<Vec<Release>, String> {
    let file: ReleasesFile =
        serde_json::from_str(text).map_err(|e| format!("releases.json: {e}"))?;
    Ok(file.releases)
}

/// The directory of a major version, as in `v7.x`. Versions before 3.0 live in `v2.6`, `v1.0` and so on, and are not asked for yet.
#[must_use]
pub fn major_dir(major: u32) -> String {
    format!("v{major}.x")
}

/// The tarball URL of a release.
#[must_use]
pub fn tarball_url(major: u32, version: &str) -> String {
    format!("{CDN}/{}/linux-{version}.tar.xz", major_dir(major))
}

/// The SHA-256 list of a major directory.
#[must_use]
pub fn sums_url(major: u32) -> String {
    format!("{CDN}/{}/sha256sums.asc", major_dir(major))
}

/// Parse a `sha256sums.asc`, file name to hash. The signature lines around the list do not look like a hash and a name and are skipped.
#[must_use]
pub fn parse_sums(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let (hash, name) = line.split_once("  ")?;
            (hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
                .then(|| (name.trim().to_owned(), hash.to_ascii_lowercase()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_follow_the_major_directory() {
        assert_eq!(
            tarball_url(6, "6.1.188"),
            "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.1.188.tar.xz"
        );
        assert_eq!(
            sums_url(7),
            "https://cdn.kernel.org/pub/linux/kernel/v7.x/sha256sums.asc"
        );
    }

    #[test]
    fn the_signed_list_reads_as_a_map() {
        let text = "-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA256\n\n\
12e8d5a973d1ad7c5a5c69882e4022b131ed715db7003fdcd760ddf8c3e51941  linux-7.2.8.tar.xz\n\
-----BEGIN PGP SIGNATURE-----\nabc\n";
        let sums = parse_sums(text);
        assert_eq!(sums.len(), 1);
        assert!(sums["linux-7.2.8.tar.xz"].starts_with("12e8d5a9"));
    }

    #[test]
    fn releases_json_reads() {
        let text = r#"{"latest_stable":{"version":"7.2.8"},"releases":[
            {"iseol":false,"version":"7.3-rc5","moniker":"mainline","source":"x"},
            {"iseol":false,"version":"6.12.111","moniker":"longterm"}]}"#;
        let releases = parse_releases(text).unwrap();
        assert_eq!(releases.len(), 2);
        assert_eq!(releases[1].moniker, "longterm");
    }
}
