//! `sets.toml`: the rules that decide which kernel trees and which binutils releases are pinned (spec 03.1).
//!
//! The sets are rules, not lists. `current` follows what kernel.org maintains this week, and `releases` is every mainline release from some version on. `gk pins` applies the rules to kernel.org and the GNU mirror and writes the result to `kernels.toml` and `binutils.toml`, which then go through review like any other change.

use crate::version::Version;
use serde::{Deserialize, Serialize};

/// The file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sets {
    /// Every kernel set.
    #[serde(rename = "set", default)]
    pub sets: Vec<Set>,
    /// The rules for the GNU side.
    #[serde(default)]
    pub gnu: Gnu,
}

/// One kernel set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Set {
    /// The name, as in `current`.
    pub name: String,
    /// The `releases.json` monikers the set takes, as in `stable` and `longterm`, at their latest point.
    #[serde(default)]
    pub monikers: Vec<String>,
    /// Whether lines kernel.org has marked end of life are kept.
    #[serde(default)]
    pub include_eol: bool,
    /// For a set of mainline releases, the first one. Every `X.Y` release from it on is in the set.
    #[serde(default)]
    pub releases_from: Option<Version>,
}

/// The rules for binutils.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Gnu {
    /// The oldest binutils pinned. Every release on the GNU mirror from it on is pinned.
    #[serde(default)]
    pub binutils_from: Option<Version>,
}

impl Sets {
    /// The sets a `releases.json` entry falls in.
    #[must_use]
    pub fn for_moniker(&self, moniker: &str, eol: bool) -> Vec<String> {
        self.sets
            .iter()
            .filter(|s| s.monikers.iter().any(|m| m == moniker) && (s.include_eol || !eol))
            .map(|s| s.name.clone())
            .collect()
    }

    /// The sets a mainline release falls in.
    #[must_use]
    pub fn for_release(&self, version: &Version) -> Vec<String> {
        self.sets
            .iter()
            .filter(|s| s.releases_from.as_ref().is_some_and(|from| version >= from))
            .map(|s| s.name.clone())
            .collect()
    }

    /// The oldest mainline release any set asks for.
    #[must_use]
    pub fn first_release(&self) -> Option<&Version> {
        self.sets
            .iter()
            .filter_map(|s| s.releases_from.as_ref())
            .min()
    }

    /// Whether a set name is one of these rules. A pin whose sets are all rules is rewritten by `gk pins`; any other pin is kept as it is.
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.sets.iter().any(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETS: &str = r#"
[[set]]
name = "current"
monikers = ["stable", "longterm"]

[[set]]
name = "releases"
releases-from = "5.0"

[gnu]
binutils-from = "2.30"
"#;

    #[test]
    fn the_rules_read_and_apply() {
        let sets: Sets = toml::from_str(SETS).unwrap();
        assert_eq!(sets.for_moniker("stable", false), ["current"]);
        assert!(sets.for_moniker("stable", true).is_empty());
        assert!(sets.for_moniker("mainline", false).is_empty());
        assert_eq!(sets.for_release(&"5.4".parse().unwrap()), ["releases"]);
        assert!(sets.for_release(&"4.20".parse().unwrap()).is_empty());
        assert_eq!(sets.first_release().unwrap().as_str(), "5.0");
        assert_eq!(sets.gnu.binutils_from.unwrap().as_str(), "2.30");
    }
}
