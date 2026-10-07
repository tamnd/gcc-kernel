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

/// Whether a record is a unit: a C or assembly source compiled to an object, or a C source compiled to assembly. kbuild's prepare step builds `bounds.c`, `asm-offsets.c` and `devicetable-offsets.c` with `-S`, and a flag the compiler refuses fails there first.
#[must_use]
pub fn is_unit(record: &CompileRecord) -> bool {
    let has = |ext: &[&str]| {
        record.inputs.iter().any(|i| {
            Path::new(&i.path)
                .extension()
                .is_some_and(|e| ext.iter().any(|x| e == *x))
        })
    };
    !record.probe
        && ((record.argv.iter().any(|a| a == "-c") && has(&["c", "S"]))
            || (record.argv.iter().any(|a| a == "-S") && has(&["c"])))
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
    let line = stderr.lines().find(|l| says_error(l))?;
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

/// Every failed unit with its first error line, sorted, one per unit. GCC writes `error:` and gas writes `Error:`.
#[must_use]
pub fn failing_units(records: &[CompileRecord], tree: &Path) -> Vec<FailedUnit> {
    let mut units: Vec<FailedUnit> = records
        .iter()
        .filter(|r| is_unit(r) && !r.succeeded())
        .map(|r| FailedUnit {
            unit: unit_source(r, tree),
            error: unit_error(&r.stderr),
        })
        .collect();
    units.sort_by(|a, b| a.unit.cmp(&b.unit));
    units.dedup_by(|a, b| a.unit == b.unit);
    units
}

/// Whether a line says error as a word of its own, and not only in a flag such as `-Werror-implicit-function-declaration` that an echoed command line carries.
fn says_error(l: &str) -> bool {
    l.match_indices("error")
        .any(|(i, _)| !l[..i].ends_with(|c: char| c.is_ascii_alphanumeric()))
}

/// The error line of a unit's stderr. GCC before 3.3 printed an error as `file:line: message` and an option it did not know as `cc1: Invalid option`, with no `error:` in either, so when no line says error, the first such line that is not a warning counts.
fn unit_error(stderr: &str) -> String {
    stderr
        .lines()
        .find(|l| says_error(l) || l.contains("Error:"))
        .or_else(|| stderr.lines().find(|l| old_style_error(l)))
        .unwrap_or_default()
        .trim()
        .to_owned()
}

/// Whether a line is an old GCC's `file:line: message` or `tool: message` that is not a warning or a note.
pub(crate) fn old_style_error(l: &str) -> bool {
    let Some((place, rest)) = l.split_once(": ") else {
        return false;
    };
    let mut parts = place.split(':');
    let file = parts.next().unwrap_or_default();
    let numbers = parts.all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    let rest = rest.trim_start();
    !file.is_empty()
        && !file.contains(' ')
        && numbers
        && !rest.starts_with("warning")
        && !rest.starts_with("Warning")
        && !rest.starts_with("recipe for target")
        && !rest.starts_with("note")
        && !rest.starts_with("In function")
        && !rest.starts_with("At top level")
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

/// What `stopped` says of a build that ran out of its budget.
const OVER_BUDGET: &str = "the build went over its budget";

/// Whether the cell in `dir` stopped because its build went over the budget, from its `build.json`. On a loaded machine that says more about the machine than the cell, so a search runs such a cell again.
#[must_use]
pub fn over_budget(dir: &Path) -> bool {
    std::fs::read_to_string(dir.join("build.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|v| v.get("stopped")?.get(1)?.as_array().cloned())
        .is_some_and(|lines| lines.iter().any(|l| l.as_str() == Some(OVER_BUDGET)))
}

/// What the C library says when a write finds the disk full.
const DISK_FULL: &str = "No space left on device";

/// Whether the cell in `dir` failed because the build machine ran out of disk, from its `errors.jsonl` and `build.json`. The kernel and the compiler had nothing to do with it, so a search runs such a cell again.
#[must_use]
pub fn disk_full(dir: &Path) -> bool {
    ["errors.jsonl", "build.json"]
        .iter()
        .any(|f| std::fs::read_to_string(dir.join(f)).is_ok_and(|text| text.contains(DISK_FULL)))
}

/// Whether the build in `dir` failed with no make error in its logs. A build that fails on its own merits leaves make's `***` line behind, in make.log or, for a museum tree, in the log of `make dep` or `make scripts`, so one without it was stopped from outside, as when docker could not write to a full disk or the container died. A cell whose logs were not kept cannot tell, and does not count.
#[must_use]
pub fn cut_short(dir: &Path) -> bool {
    let Some(json) = std::fs::read_to_string(dir.join("build.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
    else {
        return false;
    };
    let flag = |k: &str| json.get(k).and_then(serde_json::Value::as_bool);
    let logs: Vec<String> = ["make.log", "dep.log", "scripts.log"]
        .iter()
        .map(|name| crate::classify::log_text(dir, name))
        .filter(|text| !text.is_empty())
        .collect();
    flag("configured") == Some(true)
        && flag("built") == Some(false)
        && !logs.is_empty()
        && !logs.iter().any(|text| text.contains("*** "))
}

/// Whether the cell in `dir` stopped for a reason of the machine's rather than of the cell's: over the budget, out of disk, or cut short.
#[must_use]
pub fn rig_failed(dir: &Path) -> bool {
    over_budget(dir) || disk_full(dir) || cut_short(dir)
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
        return Some(("make.log".into(), vec![OVER_BUDGET.into()]));
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

/// The warnings one unit got under one option, a line of `warnings.jsonl` (spec 11.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warnings {
    /// The source, relative to the tree.
    pub unit: String,
    /// The option GCC named for them, as `-Wunused-variable`, or empty when it named none.
    pub option: String,
    /// How many.
    pub count: usize,
    /// How many of them `-Werror` turned into errors.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fatal: usize,
    /// Whether the unit was compiled with `-Werror`, where a compiler that warns more than GCC breaks the build.
    pub werror: bool,
    /// The first of them, as the compiler wrote it.
    pub first: String,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// The option of a diagnostic that is a warning, and whether `-Werror` made it fatal, or `None` for any other line. A plain error has no warning option, and GCC 4.8 and later name the option of a fatal warning as `[-Werror=name]`.
#[must_use]
pub fn warning_option(line: &str) -> Option<(String, bool)> {
    let fatal = if line.contains(": warning: ") {
        false
    } else if line.contains(": error: ") {
        true
    } else {
        return None;
    };
    let option = line
        .trim_end()
        .strip_suffix(']')
        .and_then(|l| l.rsplit_once(" ["))
        .map(|(_, o)| o)
        .filter(|o| o.starts_with("-W"));
    match option {
        Some(o) if fatal => o.strip_prefix("-Werror=").map(|w| (format!("-W{w}"), true)),
        Some(o) => Some((o.to_owned(), false)),
        None if fatal => None,
        None => Some((String::new(), false)),
    }
}

/// The warning census of a build: one entry per unit and option, sorted by unit.
#[must_use]
pub fn warning_census(records: &[CompileRecord], tree: &Path) -> Vec<Warnings> {
    let mut census: BTreeMap<(String, String), Warnings> = BTreeMap::new();
    for r in records.iter().filter(|r| is_unit(r)) {
        let werror = r.argv.iter().any(|a| a == "-Werror");
        for line in r.stderr.lines() {
            let Some((option, fatal)) = warning_option(line) else {
                continue;
            };
            let unit = unit_source(r, tree);
            let w = census
                .entry((unit.clone(), option.clone()))
                .or_insert_with(|| Warnings {
                    unit,
                    option,
                    count: 0,
                    fatal: 0,
                    werror,
                    first: line.trim().to_owned(),
                });
            w.count += 1;
            w.fatal += usize::from(fatal);
        }
    }
    census.into_values().collect()
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
    fn a_build_over_its_budget_is_told_from_build_json() {
        let dir = std::env::temp_dir().join(format!("gk-over-budget-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!over_budget(&dir));
        let (log, lines) = stopped(&dir, true, false, 0, true).unwrap();
        let json = serde_json::json!({ "stopped": [log, lines] });
        std::fs::write(dir.join("build.json"), json.to_string()).unwrap();
        assert!(over_budget(&dir));
        let json =
            serde_json::json!({ "stopped": ["make.log", ["make: *** [Makefile:1] Error 2"]] });
        std::fs::write(dir.join("build.json"), json.to_string()).unwrap();
        assert!(!over_budget(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_unit_that_found_the_disk_full_is_the_rigs_failure() {
        let dir = std::env::temp_dir().join(format!("gk-disk-full-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("errors.jsonl"),
            "{\"unit\":\"mm/slub.o\",\"stderr\":\"mm/slub.c:9:1: error: 'y' undeclared\"}\n",
        )
        .unwrap();
        assert!(!rig_failed(&dir));
        std::fs::write(dir.join("errors.jsonl"), "{\"unit\":\"mm/slub.o\",\"stderr\":\"fatal error: error writing to /tmp/ccALpihd.s: No space left on device\"}\n").unwrap();
        assert!(disk_full(&dir));
        assert!(rig_failed(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_old_gcc_error_has_no_error_in_it() {
        let gcc2 = "init/main.c: In function `checksetup':\ninit/main.c:224: fixed or forbidden register was spilled.\nThis may be due to a compiler bug or to impossible asm\n";
        assert_eq!(
            unit_error(gcc2),
            "init/main.c:224: fixed or forbidden register was spilled."
        );
        let warned = "In file included from sched.c:12:\n/out/include/linux/sys.h:145: warning: function declaration isn't a prototype\n/out/include/asm/io.h:82: inconsistent operand constraints in an `asm'\n";
        assert_eq!(
            unit_error(warned),
            "/out/include/asm/io.h:82: inconsistent operand constraints in an `asm'"
        );
        let option = "cc1: Invalid option `-fno-strict-aliasing'\n";
        assert_eq!(
            unit_error(option),
            "cc1: Invalid option `-fno-strict-aliasing'"
        );
        assert_eq!(unit_error("init/main.c: In function `start_kernel':\n"), "");
        let new = "/src/a.c:3:1: warning: x\n/src/a.c:9:2: error: y undeclared\n";
        assert_eq!(unit_error(new), "/src/a.c:9:2: error: y undeclared");
        assert_eq!(unit_error("/src/a.c:3:1: warning: x\n"), "");
    }

    #[test]
    fn a_flag_that_says_error_is_no_error() {
        let echoed = "\"i386\" \"y\" \"\" \"/gk/bin/gk-cc -Wall -Werror-implicit-function-declaration -Os\"\nld:arch/x86/kernel/vmlinux.lds:432: parse error\n";
        assert_eq!(
            error_key(echoed).unwrap(),
            "ld:arch/xN/kernel/vmlinux.lds:N: parse error"
        );
        assert!(error_key("gcc -Werror=date-time -c a.c\n").is_none());
    }

    #[test]
    fn a_failed_build_with_no_make_error_was_cut_short() {
        let dir = std::env::temp_dir().join(format!("gk-cut-short-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!cut_short(&dir));
        let json = serde_json::json!({ "configured": true, "built": false });
        std::fs::write(dir.join("build.json"), json.to_string()).unwrap();
        let log = "  CC      init/main.o\n  CC      init/do_mounts.o\n";
        std::fs::write(dir.join("make.log"), log).unwrap();
        assert!(cut_short(&dir));
        assert!(rig_failed(&dir));
        std::fs::write(dir.join("make.log"), "init/main.c:9:1: error: 'y' undeclared\nmake[1]: *** [init/main.o] Error 1\nmake: *** [init] Error 2\n").unwrap();
        assert!(!cut_short(&dir));
        std::fs::remove_file(dir.join("make.log")).unwrap();
        assert!(!cut_short(&dir));
        std::fs::write(dir.join("dep.log"), "make[2]: *** [dep] Error 1\n").unwrap();
        assert!(!cut_short(&dir));
        let json = serde_json::json!({ "configured": true, "built": true });
        std::fs::write(dir.join("build.json"), json.to_string()).unwrap();
        std::fs::write(dir.join("make.log"), "  LD      vmlinux\n").unwrap();
        assert!(!cut_short(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
    }

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
    fn the_prepare_step_compiles_to_assembly_and_counts() {
        let log = r#"{"started":1,"argv":["gk-cc","-S","-o","scripts/mod/devicetable-offsets.s","/src/scripts/mod/devicetable-offsets.c"],"compiler":"/g","cwd":"/out","inputs":[{"path":"/src/scripts/mod/devicetable-offsets.c","sha256":"a"}],"wall-seconds":0.1,"exit":1,"stderr":"/src/include/linux/compiler.h:187:1: error: '-mindirect-branch' and '-fcf-protection' are not compatible\n"}
{"started":1,"argv":["gk-cc","-E","-o","x.lds","/src/x.lds.S"],"compiler":"/g","cwd":"/out","inputs":[{"path":"/src/x.lds.S","sha256":"a"}],"wall-seconds":0.1,"exit":0}
"#;
        let (records, _) = parse_log(log);
        let failed = failing_units(&records, Path::new("/src"));
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].unit, "scripts/mod/devicetable-offsets.c");
        assert_eq!(count(&records, 0).units, 1);
    }

    #[test]
    fn a_unit_that_fails_in_the_assembler_has_the_gas_error_line() {
        let log = r#"{"started":1,"argv":["gk-cc","-c","-o","arch/i386/kernel/vsyscall.o","/src/arch/i386/kernel/vsyscall.S"],"compiler":"/g","cwd":"/out","inputs":[{"path":"/src/arch/i386/kernel/vsyscall.S","sha256":"a"}],"wall-seconds":0.1,"exit":1,"stderr":"/tmp/ccZnGQvD.s: Assembler messages:\n/tmp/ccZnGQvD.s:1699: Error: Unknown pseudo-op:  `.incbin'\n"}
"#;
        let (records, _) = parse_log(log);
        let failed = failing_units(&records, Path::new("/src"));
        assert_eq!(
            failed[0].error,
            "/tmp/ccZnGQvD.s:1699: Error: Unknown pseudo-op:  `.incbin'"
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

    #[test]
    fn warnings_are_counted_by_unit_and_option() {
        assert_eq!(
            warning_option("/src/a.c:3:7: warning: unused variable 'x' [-Wunused-variable]"),
            Some(("-Wunused-variable".into(), false))
        );
        assert_eq!(
            warning_option("/src/a.c:9:1: error: no return statement [-Werror=return-type]"),
            Some(("-Wreturn-type".into(), true))
        );
        assert_eq!(warning_option("/src/a.c:9:1: error: expected ';'"), None);
        assert_eq!(
            warning_option("/src/a.c:1:1: warning: #warning hello"),
            Some((String::new(), false))
        );
        assert_eq!(warning_option("/src/a.c:3:7: note: declared here"), None);
    }
}
