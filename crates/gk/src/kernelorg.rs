//! What kernel.org publishes: the list of maintained lines and the SHA-256 lists.
//!
//! `releases.json` names the mainline rc, the stable line and every longterm line with its latest point release. `sha256sums.asc` in each major directory lists the hash of every tarball there. The list is signed. `gk fetch` checks the signature, and the hash in `kernels.toml` is what every fetch is held to.

use gk_model::Version;
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

/// The directories that hold the tarballs of a major version, as `v7.x`. The 2.6 tree is in `v2.6`, and the points of its longterm lines that came after the move to kernel.org's longterm area are in `v2.6/longterm` and one directory per line under it. The museum's lines before 2.6 each have a directory of their own, and only the stable ones are asked for.
#[must_use]
pub fn dirs(major: u32) -> Vec<String> {
    if major >= 3 {
        return vec![format!("v{major}.x")];
    }
    if major == 1 {
        return vec!["v1.0".to_owned(), "v1.2".to_owned()];
    }
    if major < 1 {
        return Vec::new();
    }
    let mut dirs = vec!["v2.6".to_owned(), "v2.6/longterm".to_owned()];
    dirs.extend(
        ["27", "32", "33", "34", "35"]
            .iter()
            .map(|l| format!("v2.6/longterm/v2.6.{l}")),
    );
    dirs.extend(["v2.0", "v2.2", "v2.4"].map(str::to_owned));
    dirs
}

/// How many parts name the line a version belongs to: `7.2` and `2.4`, but `2.6.32`.
#[must_use]
pub fn line_depth(version: &Version) -> usize {
    match version.parts() {
        [2, 6, ..] => 3,
        _ => 2,
    }
}

/// Whether a version is from before 2.6, where the last point of a line can be its release, as for 1.0, and a point can have four parts, as 2.4.37.11.
#[must_use]
pub fn is_museum(version: &Version) -> bool {
    matches!(version.parts(), [0 | 1, ..] | [2, 0..=5, ..])
}

/// How many parts a mainline release of a major version has: `7.2` but `2.6.32`. A point release has one more.
#[must_use]
pub fn release_depth(major: u32) -> usize {
    if major >= 3 { 2 } else { 3 }
}

/// The tarball URL of a release in a directory.
#[must_use]
pub fn tarball_url(dir: &str, version: &str) -> String {
    format!("{CDN}/{dir}/linux-{version}.tar.xz")
}

/// The SHA-256 list of a directory.
#[must_use]
pub fn sums_url(dir: &str) -> String {
    format!("{CDN}/{dir}/sha256sums.asc")
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
            tarball_url("v6.x", "6.1.188"),
            "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.1.188.tar.xz"
        );
        assert_eq!(
            sums_url("v7.x"),
            "https://cdn.kernel.org/pub/linux/kernel/v7.x/sha256sums.asc"
        );
        assert_eq!(dirs(7), ["v7.x"]);
        assert_eq!(dirs(2).len(), 10);
        assert_eq!(dirs(1), ["v1.0", "v1.2"]);
        assert!(dirs(0).is_empty());
        assert_eq!(release_depth(6), 2);
        assert_eq!(release_depth(2), 3);
    }

    #[test]
    fn museum_lines_are_two_parts_deep() {
        let v = |s: &str| s.parse::<Version>().unwrap();
        assert_eq!(line_depth(&v("2.6.32.71")), 3);
        assert_eq!(line_depth(&v("2.4.37.11")), 2);
        assert_eq!(line_depth(&v("6.1.188")), 2);
        assert!(is_museum(&v("1.0")));
        assert!(is_museum(&v("2.4.37.11")));
        assert!(!is_museum(&v("2.6.0")));
        assert!(!is_museum(&v("3.0")));
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
