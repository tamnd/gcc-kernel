//! Kernel and toolchain version numbers, ordered the way their releases came out.
//!
//! Kernel versions have more shapes than a semver parser accepts: `0.96c`, `0.99.15`, `1.3.100`, `2.3.99-pre9`, `2.6.32.71` and `7.3-rc5` are all real. A version is its numeric parts, an optional letter after the last one, and an optional pre-release tag. A pre-release sorts before the release it leads to, and a letter sorts after the plain number.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// A version number.
///
/// Equality follows the order, so `5.0` and `5.0.0` are the same version even though they print differently.
#[derive(Debug, Clone)]
pub struct Version {
    text: String,
    parts: Vec<u32>,
    letter: Option<char>,
    pre: Option<(String, u32)>,
}

impl Version {
    /// The numeric parts, as in `[2, 6, 32, 71]`.
    #[must_use]
    pub fn parts(&self) -> &[u32] {
        &self.parts
    }

    /// The first `n` numeric parts, padded with zeros.
    #[must_use]
    pub fn series(&self, n: usize) -> Vec<u32> {
        (0..n)
            .map(|i| self.parts.get(i).copied().unwrap_or(0))
            .collect()
    }

    /// Whether this is a release candidate or pre-release.
    #[must_use]
    pub fn is_pre(&self) -> bool {
        self.pre.is_some()
    }

    /// The version as it was written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Whether `self` is at or above `low` and below `high`, where a missing bound is open.
    #[must_use]
    pub fn within(&self, low: Option<&Version>, high: Option<&Version>) -> bool {
        low.is_none_or(|l| self >= l) && high.is_none_or(|h| self < h)
    }
}

/// The error for text that is not a version.
#[derive(Debug, PartialEq, Eq)]
pub struct BadVersion(pub String);

impl fmt::Display for BadVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a version: {:?}", self.0)
    }
}

impl std::error::Error for BadVersion {}

impl FromStr for Version {
    type Err = BadVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bad = || BadVersion(s.to_owned());
        let text = s.trim();
        let (main, pre) = match text.split_once('-') {
            Some((main, tag)) => {
                let split = tag.find(|c: char| c.is_ascii_digit()).ok_or_else(bad)?;
                let (kind, n) = tag.split_at(split);
                if kind.is_empty() || !kind.bytes().all(|b| b.is_ascii_lowercase()) {
                    return Err(bad());
                }
                (main, Some((kind.to_owned(), n.parse().map_err(|_| bad())?)))
            }
            None => (text, None),
        };
        let mut parts = Vec::new();
        let mut letter = None;
        let pieces: Vec<&str> = main.split('.').collect();
        for (i, piece) in pieces.iter().enumerate() {
            let digits = piece.trim_end_matches(|c: char| c.is_ascii_lowercase());
            let rest = &piece[digits.len()..];
            if digits.is_empty() || rest.len() > 1 || (!rest.is_empty() && i + 1 != pieces.len()) {
                return Err(bad());
            }
            parts.push(digits.parse().map_err(|_| bad())?);
            letter = rest.chars().next();
        }
        Ok(Version {
            text: text.to_owned(),
            parts,
            letter,
            pre,
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let len = self.parts.len().max(other.parts.len());
        let pad = |v: &Version| {
            (0..len)
                .map(|i| v.parts.get(i).copied().unwrap_or(0))
                .collect::<Vec<_>>()
        };
        pad(self)
            .cmp(&pad(other))
            .then_with(|| self.letter.cmp(&other.letter))
            .then_with(|| match (&self.pre, &other.pre) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            })
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

impl std::hash::Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let end = self
            .parts
            .iter()
            .rposition(|&p| p != 0)
            .map_or(0, |i| i + 1);
        self.parts[..end].hash(state);
        self.letter.hash(state);
        self.pre.hash(state);
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl Serialize for Version {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.text)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        s.parse().unwrap()
    }

    #[test]
    fn every_shape_the_kernel_used_parses() {
        for s in [
            "0.01",
            "0.96c",
            "0.99.15",
            "1.3.100",
            "2.3.99-pre9",
            "2.6.32.71",
            "7.3-rc5",
            "16.2.0",
        ] {
            assert_eq!(v(s).to_string(), s);
        }
        for s in ["", "7.x", "1..2", "7.3-5", "0.9cc", "v7.2"] {
            assert!(s.parse::<Version>().is_err(), "{s}");
        }
    }

    #[test]
    fn releases_sort_in_release_order() {
        let mut list = vec![
            v("7.3"),
            v("2.6.32.71"),
            v("7.3-rc5"),
            v("0.96c"),
            v("0.96"),
            v("2.6.32"),
            v("0.99.15"),
            v("7.2.8"),
            v("2.3.99-pre9"),
            v("2.4.0"),
        ];
        list.sort();
        let got: Vec<String> = list.iter().map(ToString::to_string).collect();
        assert_eq!(
            got,
            [
                "0.96",
                "0.96c",
                "0.99.15",
                "2.3.99-pre9",
                "2.4.0",
                "2.6.32",
                "2.6.32.71",
                "7.2.8",
                "7.3-rc5",
                "7.3"
            ]
        );
    }

    #[test]
    fn trailing_zeros_do_not_matter() {
        assert_eq!(v("5.0").cmp(&v("5.0.0")), Ordering::Equal);
        assert!(v("6.15").within(Some(&v("6.15")), None));
        assert!(!v("6.14.11").within(Some(&v("6.15")), None));
        assert!(v("6.14.11").within(None, Some(&v("6.15"))));
    }
}
