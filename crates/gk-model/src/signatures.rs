//! `signatures.toml`: the failure catalog of spec 08.2.
//!
//! A signature names a pattern over a failed cell's first error and its context, says what kind of failure it is, and lists the kernel commits that made it go away. The ranges limit where it may apply: a match outside them is a new finding, not a classification.

use crate::version::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// `signatures.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signatures {
    /// Every signature, in the order the catalog lists them.
    #[serde(rename = "signature", default)]
    pub signatures: Vec<Signature>,
}

/// One signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Signature {
    /// The short unique name, as `gnu23-bool`.
    pub class: String,
    /// Where it shows: one rung as `L3`, or a span as `L5..L7`.
    pub rung: String,
    /// The patterns, all of which must match.
    #[serde(rename = "match")]
    pub patterns: Patterns,
    /// One of [`KINDS`], or `binutils-` and a word.
    pub kind: String,
    /// One sentence.
    pub summary: String,
    /// The kernel commits that made it go away.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixed_by: Vec<Fix>,
    /// The GCC versions it can apply to, as a [`Range`].
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub gcc: String,
    /// The GCC flavors it can apply to, `upstream` or a distribution. Empty means any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flavor: Vec<String>,
    /// The binutils versions it can apply to.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub binutils: String,
    /// The platforms it can apply to. Empty means any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub platform: Vec<String>,
    /// The kernel versions it can apply to.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kernel: String,
}

/// The regular expressions of a signature, each over one part of a failed cell.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Patterns {
    /// The first error: the refusal for L1, the first failing unit's error for L3, the lines kbuild stopped on otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_error: Option<String>,
    /// The first failing unit, relative to the tree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// The console of the boots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub console: Option<String>,
    /// The `.config`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    /// The command line of the first failing unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

impl Patterns {
    /// Every pattern with the name of the part it reads.
    #[must_use]
    pub fn all(&self) -> Vec<(&'static str, &str)> {
        [
            ("first-error", &self.first_error),
            ("unit", &self.unit),
            ("console", &self.console),
            ("config", &self.config),
            ("command", &self.command),
        ]
        .into_iter()
        .filter_map(|(name, p)| p.as_deref().map(|p| (name, p)))
        .collect()
    }
}

/// A kernel commit that fixed a signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Fix {
    /// The abbreviated commit id.
    pub commit: String,
    /// The first tag that contains it, as `v4.2-rc1`, or empty for a policy that was never undone.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub first_tag: String,
    /// The stable branches it was backported to, as `4.19`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stable: Vec<String>,
    /// Anything the commit and tag do not say.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// Whether `gk bisect-kernel` found this commit.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bisected: bool,
}

impl Fix {
    /// Whether a kernel at `version` has the fix: it is at or after the first tag, or on a stable branch that got it. `None` when the fix has no tag to compare with.
    #[must_use]
    pub fn in_tree(&self, version: &Version) -> Option<bool> {
        let tag: Version = self.first_tag.trim_start_matches('v').parse().ok()?;
        if *version >= tag {
            return Some(true);
        }
        let series = version.series(2);
        Some(self.stable.iter().any(|s| {
            s.parse::<Version>()
                .is_ok_and(|b| b.series(2) == series && version.parts().len() > 2)
        }))
    }
}

/// The kinds of spec 08.3, apart from the `binutils-` family.
pub const KINDS: [&str; 14] = [
    "refusal",
    "blacklist",
    "new-error",
    "new-warning-werror",
    "default-change",
    "miscompile",
    "miscompile-by-assumption",
    "ice",
    "objtool",
    "modpost",
    "too-old",
    "host-tool",
    "emulator",
    "timeout",
];

/// A set of versions, written as alternatives joined by `|`, each a list of bounds joined by `,`.
///
/// A bound is `>=X`, `>X`, `<=X`, `<X`, or `=X` (or just `X`). `X` names a series as much as a release: `=4.1` is every 4.1 release, `<=4.8` takes in 4.8.5, and `>4.8` starts after the last 4.8. So `4.1.0 | 4.1.1` is two releases and `>=4.5, <=4.8.1` is a span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range(Vec<Vec<(Op, Version)>>);

/// A bound's comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Ge,
    Gt,
    Le,
    Lt,
    Eq,
}

/// Whether `v` is in the series `x`: its first parts are `x`'s.
fn in_series(v: &Version, x: &Version) -> bool {
    v.series(x.parts().len()) == x.parts()
}

impl Range {
    /// Read a range. The empty string is every version.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut any = Vec::new();
        for alternative in text.split('|').filter(|a| !a.trim().is_empty()) {
            let mut all = Vec::new();
            for bound in alternative.split(',') {
                let bound = bound.trim();
                let (op, rest) = [
                    (">=", Op::Ge),
                    ("<=", Op::Le),
                    (">", Op::Gt),
                    ("<", Op::Lt),
                    ("=", Op::Eq),
                ]
                .iter()
                .find_map(|(p, op)| bound.strip_prefix(p).map(|r| (*op, r)))
                .unwrap_or((Op::Eq, bound));
                let v: Version = rest
                    .trim()
                    .parse()
                    .map_err(|e| format!("range {text:?}: {e}"))?;
                all.push((op, v));
            }
            any.push(all);
        }
        Ok(Range(any))
    }

    /// Whether `v` is in the range.
    #[must_use]
    pub fn contains(&self, v: &Version) -> bool {
        self.0.is_empty()
            || self.0.iter().any(|all| {
                all.iter().all(|(op, x)| match op {
                    Op::Ge => v >= x,
                    Op::Gt => v > x && !in_series(v, x),
                    Op::Le => v <= x || in_series(v, x),
                    Op::Lt => v < x,
                    Op::Eq => in_series(v, x),
                })
            })
    }
}

/// Whether a rung span such as `L5..L7` covers `rung`.
#[must_use]
pub fn rung_covers(span: &str, rung: &str) -> bool {
    let number = |r: &str| {
        r.trim()
            .strip_prefix('L')
            .and_then(|n| n.parse::<u8>().ok())
    };
    let (low, high) = span.split_once("..").unwrap_or((span, span));
    match (number(low), number(high), number(rung)) {
        (Some(l), Some(h), Some(r)) => l <= r && r <= h,
        _ => false,
    }
}

impl Signatures {
    /// A signature by class.
    #[must_use]
    pub fn get(&self, class: &str) -> Option<&Signature> {
        self.signatures.iter().find(|s| s.class == class)
    }

    /// Every problem with the catalog: duplicate classes, unknown kinds, rungs, ranges and patterns that do not read, and signatures with no pattern or no summary.
    #[must_use]
    pub fn check(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let mut seen = BTreeSet::new();
        for s in &self.signatures {
            let name = &s.class;
            if !seen.insert(name.as_str()) {
                problems.push(format!("signatures.toml: {name} is there twice"));
            }
            if !KINDS.contains(&s.kind.as_str()) && !s.kind.starts_with("binutils-") {
                problems.push(format!(
                    "signatures.toml: {name} has an unknown kind {:?}",
                    s.kind
                ));
            }
            let (low, high) = s.rung.split_once("..").unwrap_or((&s.rung, &s.rung));
            if !rung_covers(&s.rung, low) || !rung_covers(&s.rung, high) {
                problems.push(format!(
                    "signatures.toml: {name} has a rung {:?} that does not read",
                    s.rung
                ));
            }
            for (what, text) in [
                ("gcc", &s.gcc),
                ("binutils", &s.binutils),
                ("kernel", &s.kernel),
            ] {
                if let Err(e) = Range::parse(text) {
                    problems.push(format!("signatures.toml: {name} {what}: {e}"));
                }
            }
            if s.patterns.all().is_empty() {
                problems.push(format!("signatures.toml: {name} has nothing to match"));
            }
            for (part, pattern) in s.patterns.all() {
                if let Err(e) = regex::Regex::new(pattern) {
                    problems.push(format!("signatures.toml: {name} {part}: {e}"));
                }
            }
            if s.summary.trim().is_empty() {
                problems.push(format!("signatures.toml: {name} has no summary"));
            }
        }
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        s.parse().unwrap()
    }

    #[test]
    fn ranges_read_series_as_well_as_releases() {
        let r = Range::parse(">=4.5, <=4.8.1").unwrap();
        assert!(r.contains(&v("4.5.0")));
        assert!(r.contains(&v("4.8.1")));
        assert!(!r.contains(&v("4.8.2")));
        assert!(!r.contains(&v("4.4.7")));
        let r = Range::parse("4.1.0 | 4.1.1").unwrap();
        assert!(r.contains(&v("4.1.1")));
        assert!(!r.contains(&v("4.1.2")));
        assert!(Range::parse("<=4.8").unwrap().contains(&v("4.8.5")));
        assert!(!Range::parse(">4.8").unwrap().contains(&v("4.8.5")));
        assert!(Range::parse(">4.8").unwrap().contains(&v("4.9.0")));
        assert!(Range::parse("=10").unwrap().contains(&v("10.5.0")));
        assert!(Range::parse("").unwrap().contains(&v("1.0")));
        assert!(Range::parse(">=x").is_err());
    }

    #[test]
    fn rung_spans_cover_their_ends() {
        assert!(rung_covers("L5..L7", "L6"));
        assert!(rung_covers("L3", "L3"));
        assert!(!rung_covers("L3", "L4"));
        assert!(!rung_covers("L9x", "L3"));
    }

    #[test]
    fn a_fix_is_in_the_tree_from_its_tag_or_on_its_stable_branches() {
        let fix = Fix {
            commit: "a9a3ed1eff36".into(),
            first_tag: "v5.7".into(),
            stable: vec!["4.19".into(), "5.4".into()],
            note: String::new(),
            bisected: false,
        };
        assert_eq!(fix.in_tree(&v("5.7")), Some(true));
        assert_eq!(fix.in_tree(&v("5.4.290")), Some(true));
        assert_eq!(fix.in_tree(&v("5.4")), Some(false));
        assert_eq!(fix.in_tree(&v("5.6.19")), Some(false));
        let policy = Fix {
            first_tag: String::new(),
            ..fix
        };
        assert_eq!(policy.in_tree(&v("5.7")), None);
    }

    #[test]
    fn the_catalog_checks_itself() {
        let s: Signatures = toml::from_str(
            r#"
[[signature]]
class = "x"
rung = "L3"
match = { first-error = "boom" }
kind = "new-error"
summary = "It goes boom"

[[signature]]
class = "x"
rung = "L3..Lz"
match = {}
kind = "odd"
summary = ""
gcc = ">=what"
"#,
        )
        .unwrap();
        let p = s.check();
        assert_eq!(p.len(), 6, "{p:?}");
        let bad: Signatures = toml::from_str(
            r#"
[[signature]]
class = "y"
rung = "L3"
match = { first-error = "(" }
kind = "ice"
summary = "Bad pattern"
"#,
        )
        .unwrap();
        assert_eq!(bad.check().len(), 1);
    }
}
