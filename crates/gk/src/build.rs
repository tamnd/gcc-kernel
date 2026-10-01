//! `build.json`: the record of one build in rucc-kernel's schema, with `cell` and `rung` added (spec 10.4).
//!
//! The struct and the counting over `compile.jsonl` are copied from rucc-kernel's `rk build`, so that its tools read a gcc-kernel cell directory without change. The persona fields stay empty here, because a cell only ever runs a real GCC.

use gk_cc::record::CompileRecord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The compiler of a build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Compiler {
    /// The absolute path, as the build saw it.
    pub path: PathBuf,
    /// The first line of `--version`.
    pub version: String,
    /// The SHA-256 of the driver.
    pub sha256: String,
    /// Always false here. The field is part of the schema.
    pub rucc: bool,
}

impl Compiler {
    /// A bundle's GCC, asked for its version on the machine running gk, since bundles are static. `inside` is its path in the container.
    #[must_use]
    pub fn of(bundle: &Path, target: &str, inside: &str) -> Self {
        let driver = bundle.join("bin").join(format!("{target}-gcc"));
        let version = Command::new(&driver)
            .arg("--version")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_owned()
            })
            .unwrap_or_default();
        Compiler {
            path: PathBuf::from(inside),
            version,
            sha256: crate::net::sha256_file(&driver).unwrap_or_default(),
            rucc: false,
        }
    }
}

/// What happened, written to `build.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Outcome {
    /// The kernel version.
    pub version: String,
    /// The row, which is the platform here.
    pub row: String,
    /// The configuration.
    pub config: String,
    /// The era.
    pub era: String,
    /// Empty: no persona.
    #[serde(default)]
    pub gnuc: String,
    /// Empty: no persona.
    #[serde(default)]
    pub std: String,
    /// Where the tree was, as the build saw it.
    #[serde(default)]
    pub source: PathBuf,
    /// The compiler.
    pub compiler: Compiler,
    /// Empty: no persona.
    pub persona: Vec<String>,
    /// The make targets.
    pub targets: Vec<String>,
    /// Nothing is ever passed in `KCFLAGS` by a graded cell.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kcflags: Vec<String>,
    /// The fragment merged after the configuration target.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fragment: String,
    /// The fragment's requests that did not take effect.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fragment_missed: Vec<String>,
    /// Whether the configuration step succeeded.
    pub configured: bool,
    /// Whether the build step succeeded.
    pub built: bool,
    /// The SHA-256 of the `.config`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub config_sha256: String,
    /// The SHA-256 of the boot image.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub image_sha256: String,
    /// Seconds spent in the whole cell.
    pub wall_seconds: f64,
    /// Counts over `compile.jsonl`.
    pub calls: Calls,
    /// The most common error messages of failed units, normalized, with their counts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<(String, usize)>,
    /// Failed units, relative to the tree.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed_units: Vec<String>,
    /// Whether the result can be graded.
    pub graded: bool,
    /// When a step failed for a reason that is not a failed unit, the log and its last lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped: Option<(String, Vec<String>)>,
    /// The identity of the cell.
    pub cell: String,
    /// The highest rung passed.
    pub rung: String,
}

/// Counts over the compile log.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Calls {
    /// Every call.
    pub total: usize,
    /// Probes.
    pub probes: usize,
    /// Probes the compiler said no to.
    pub probes_failed: usize,
    /// Units: C or `.S` files compiled to objects.
    pub units: usize,
    /// Units that failed.
    pub units_failed: usize,
    /// Always 0 here. The field is part of the schema.
    pub delegated: usize,
    /// Units whose second compile wrote different bytes.
    pub nondeterministic: usize,
    /// Lines of the log that did not parse.
    pub unreadable: usize,
}

/// One failed unit, a line of `errors.jsonl`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedUnit {
    /// The source, relative to the tree.
    pub unit: String,
    /// The first error line, as the compiler wrote it.
    pub error: String,
}

/// Whether a record is a unit: a C or assembly source compiled to an object.
#[must_use]
pub fn is_unit(record: &CompileRecord) -> bool {
    !record.probe
        && record.argv.iter().any(|a| a == "-c")
        && record.inputs.iter().any(|i| {
            Path::new(&i.path)
                .extension()
                .is_some_and(|e| e == "c" || e == "S")
        })
}

/// Count the log.
#[must_use]
pub fn count(records: &[CompileRecord], unreadable: usize) -> Calls {
    let mut calls = Calls {
        total: records.len(),
        unreadable,
        ..Calls::default()
    };
    for record in records {
        if record.probe {
            calls.probes += 1;
            if !record.succeeded() {
                calls.probes_failed += 1;
            }
        } else if is_unit(record) {
            calls.units += 1;
            if !record.succeeded() {
                calls.units_failed += 1;
            }
            if record.twice.as_ref().is_some_and(|t| !t.identical) {
                calls.nondeterministic += 1;
            }
        }
    }
    calls
}

/// The message of the first `error:` line in a compiler's standard error, with quoted names and numbers taken out, so that failures group by cause.
#[must_use]
pub fn error_key(stderr: &str) -> Option<String> {
    let line = stderr.lines().find(|l| l.contains("error"))?;
    let message = line
        .split_once("error: ")
        .map_or(line, |(_, m)| m)
        .trim()
        .to_string();
    let mut out = String::new();
    let mut quote: Option<char> = None;
    for c in message.chars() {
        match (quote, c) {
            (None, '\'' | '`' | '"') => {
                quote = Some(if c == '`' { '\'' } else { c });
                out.push_str("'_'");
            }
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, c) if c.is_ascii_digit() => {
                if !out.ends_with('N') {
                    out.push('N');
                }
            }
            (None, c) => out.push(c),
        }
    }
    Some(out.chars().take(160).collect())
}

/// The error keys of failed units, most common first.
#[must_use]
pub fn error_census(records: &[CompileRecord]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for record in records.iter().filter(|r| is_unit(r) && !r.succeeded()) {
        let key = error_key(&record.stderr).unwrap_or_else(|| {
            record.signal.map_or_else(
                || "(no error line)".to_string(),
                |s| format!("(killed by signal {s})"),
            )
        });
        *counts.entry(key).or_default() += 1;
    }
    let mut out: Vec<(String, usize)> = counts.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    out
}

/// The source file of a unit, relative to the tree when it is inside it.
#[must_use]
pub fn unit_source(record: &CompileRecord, tree: &Path) -> String {
    let input = record
        .inputs
        .iter()
        .find(|i| {
            Path::new(&i.path)
                .extension()
                .is_some_and(|e| e == "c" || e == "S")
        })
        .map_or("", |i| i.path.as_str());
    // Resolve `..` by the words alone, since an out of tree build names its sources through it.
    let mut full = PathBuf::new();
    for part in Path::new(&record.cwd).join(input).components() {
        match part {
            std::path::Component::ParentDir => {
                full.pop();
            }
            std::path::Component::CurDir => {}
            other => full.push(other),
        }
    }
    full.strip_prefix(tree)
        .map_or_else(|_| input.to_string(), |p| p.display().to_string())
}

/// Every failed unit with its first error line, sorted, one per unit.
#[must_use]
pub fn failing_units(records: &[CompileRecord], tree: &Path) -> Vec<FailedUnit> {
    let mut units: Vec<FailedUnit> = records
        .iter()
        .filter(|r| is_unit(r) && !r.succeeded())
        .map(|r| FailedUnit {
            unit: unit_source(r, tree),
            error: r
                .stderr
                .lines()
                .find(|l| l.contains("error"))
                .unwrap_or_default()
                .trim()
                .to_owned(),
        })
        .collect();
    units.sort_by(|a, b| a.unit.cmp(&b.unit));
    units.dedup_by(|a, b| a.unit == b.unit);
    units
}

/// The last lines of a log that say why kbuild stopped: not make's own lines, and not kbuild's indented progress lines.
#[must_use]
pub fn failure_lines(log: &str, n: usize) -> Vec<String> {
    let lines: Vec<&str> = log
        .lines()
        .filter(|l| {
            !l.trim().is_empty()
                && !l.starts_with("make:")
                && !l.starts_with("make[")
                && !l.starts_with("  ")
        })
        .collect();
    lines[lines.len().saturating_sub(n)..]
        .iter()
        .map(|l| (*l).to_string())
        .collect()
}

/// Why a build stopped, when no failed unit explains it.
#[must_use]
pub fn stopped(
    dir: &Path,
    configured: bool,
    built: bool,
    units_failed: usize,
    timed_out: bool,
) -> Option<(String, Vec<String>)> {
    if timed_out {
        return Some((
            "make.log".into(),
            vec!["the build went over its budget".into()],
        ));
    }
    let log = if !configured {
        "config.log"
    } else if !built && units_failed == 0 {
        "make.log"
    } else {
        return None;
    };
    let text = std::fs::read_to_string(dir.join(log)).ok()?;
    Some((log.to_string(), failure_lines(&text, 6)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gk_cc::record::parse_log;

    const LOG: &str = r#"{"started":1,"argv":["gk-cc","-Werror","-c","-x","c","/dev/null","-o",".tmp_1/tmp"],"compiler":"/g","cwd":"/out","wall-seconds":0.1,"exit":1,"probe":true}
{"started":1,"argv":["gk-cc","-c","-o","kernel/fork.o","/src/kernel/fork.c"],"compiler":"/g","cwd":"/out","inputs":[{"path":"/src/kernel/fork.c","sha256":"a"}],"wall-seconds":0.1,"exit":0}
{"started":1,"argv":["gk-cc","-c","-o","kernel/exit.o","../src/kernel/exit.c"],"compiler":"/g","cwd":"/out","inputs":[{"path":"../src/kernel/exit.c","sha256":"a"}],"wall-seconds":0.1,"exit":1,"stderr":"/src/kernel/exit.c:12:3: error: 'x' undeclared on line 40\n"}
{"started":1,"argv":["gk-cc","-c","-o","mm/slub.o","/src/mm/slub.c"],"compiler":"/g","cwd":"/out","inputs":[{"path":"/src/mm/slub.c","sha256":"a"}],"wall-seconds":0.1,"exit":1,"stderr":"/src/mm/slub.c:99:1: error: 'y' undeclared on line 7\n"}
"#;

    #[test]
    fn the_log_is_counted_by_kind() {
        let (records, skipped) = parse_log(LOG);
        let calls = count(&records, skipped);
        assert_eq!(calls.total, 4);
        assert_eq!(calls.probes, 1);
        assert_eq!(calls.probes_failed, 1);
        assert_eq!(calls.units, 3);
        assert_eq!(calls.units_failed, 2);
    }

    #[test]
    fn failed_units_are_named_inside_the_tree_with_their_first_error() {
        let (records, _) = parse_log(LOG);
        let failed = failing_units(&records, Path::new("/src"));
        assert_eq!(failed.len(), 2);
        assert_eq!(failed[0].unit, "kernel/exit.c");
        assert!(failed[0].error.contains("undeclared"));
        assert_eq!(
            error_census(&records),
            [("'_' undeclared on line N".to_string(), 2)]
        );
    }

    #[test]
    fn the_reason_kbuild_stopped_is_not_make_noise() {
        let log = "  HOSTLD  scripts/kconfig/conf
scripts/Kconfig.include:51: Sorry, this assembler is not supported.
make[5]: *** [scripts/kconfig/Makefile:85: defconfig] Error 1
make: *** [Makefile:248: __sub-make] Error 2
";
        assert_eq!(
            failure_lines(log, 6),
            ["scripts/Kconfig.include:51: Sorry, this assembler is not supported."]
        );
    }
}
