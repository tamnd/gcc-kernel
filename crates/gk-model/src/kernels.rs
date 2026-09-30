//! `kernels.toml`: every kernel tree that is a row of the matrix (spec 03).

use crate::version::Version;
use serde::{Deserialize, Serialize};

/// The file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kernels {
    /// Every tree, oldest first.
    #[serde(rename = "kernel", default)]
    pub kernels: Vec<Kernel>,
}

/// One pinned tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Kernel {
    /// The release, as in `7.2.8`.
    pub version: Version,
    /// The tarball.
    pub url: String,
    /// Its SHA-256.
    pub sha256: String,
    /// The sets the tree belongs to (spec 03.1).
    pub sets: Vec<String>,
    /// What kernel.org called its line when the pin was taken, for trees of the `current` set.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub moniker: String,
}

impl Kernels {
    /// A tree by version.
    #[must_use]
    pub fn get(&self, version: &Version) -> Option<&Kernel> {
        self.kernels.iter().find(|k| k.version == *version)
    }

    /// Every tree in a set, oldest first.
    #[must_use]
    pub fn in_set(&self, set: &str) -> Vec<&Kernel> {
        self.kernels
            .iter()
            .filter(|k| k.sets.iter().any(|s| s == set))
            .collect()
    }

    /// Sort oldest first, which is the order the file is written in.
    pub fn sort(&mut self) {
        self.kernels.sort_by(|a, b| a.version.cmp(&b.version));
    }
}

impl Kernel {
    /// The tarball's file name, as in `linux-7.2.8.tar.xz`.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.url.rsplit('/').next().unwrap_or_default()
    }
}
