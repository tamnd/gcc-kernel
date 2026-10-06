//! `gk classify`, `gk triage` and `gk explain`: the failure catalog of spec 08 run over the store.
//!
//! A failed cell is read into a [`Failure`]: the rung it failed at and the five parts a signature can match. The first signature, in catalog order, whose rung covers the failure and whose patterns all match names the cell's class. A signature that matches but whose ranges leave the cell out is a finding instead: either the signature is too narrow or the cell shows something new.

use crate::build;
use crate::cell::CellRecord;
use crate::store;
use gk_model::Version;
use gk_model::repo::Repo;
use gk_model::signatures::{Range, Signature, rung_covers};
use regex::Regex;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What a failed cell shows, part by part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Failure {
    /// The rung it failed at, as `L3`.
    pub rung: String,
    /// The refusal for L1, the first error of the first failing unit for L3, the first bad console line for L5 to L8, and otherwise the lines kbuild stopped on.
    pub first_error: String,
    /// The first failing unit, relative to the tree.
    pub unit: String,
    /// The console of the boot that failed.
    pub console: String,
    /// The `.config`, which the cell directory keeps as `config`.
    pub config: String,
    /// The command line of the first failing unit.
    pub command: String,
}

impl Failure {
    fn part(&self, name: &str) -> &str {
        match name {
            "first-error" => &self.first_error,
            "unit" => &self.unit,
            "console" => &self.console,
            "config" => &self.config,
            "command" => &self.command,
            _ => "",
        }
    }
}

/// The lines of a build log that say something went wrong, in the order they came.
fn error_lines(log: &str) -> Vec<&str> {
    log.lines().filter(|l| is_error_line(l)).collect()
}

fn is_error_line(l: &str) -> bool {
    !l.starts_with("make:")
        && !l.starts_with("make[")
        && !l.starts_with('#')
        && !l.starts_with("  ")
        && (l.contains("error:")
            || l.contains("Error:")
            || l.contains("ERROR:")
            || l.contains("objtool:")
            || l.contains("modpost:")
            || l.contains("undefined reference")
            || l.contains("multiple definition")
            || l.contains("LOAD segment with RWX"))
}

/// The error line that names the target make first gave up on, as `arch/x86/entry/thunk_64.o: warning: objtool: missing symbol table` before `*** [arch/x86/entry/thunk_64.o] Error 1`. Under `-j` that says more than the lines kbuild stopped on, which are whatever the other jobs printed last.
fn gave_up_on(make: &str) -> Option<String> {
    let lines: Vec<&str> = make.lines().collect();
    let at = lines
        .iter()
        .position(|l| (l.starts_with("make:") || l.starts_with("make[")) && l.contains("*** ["))?;
    let target = lines[at].split("*** [").nth(1)?.split(']').next()?;
    let target = target.rsplit(": ").next()?;
    lines[..at]
        .iter()
        .rev()
        .find(|l| is_error_line(l) && l.contains(target))
        .map(|l| l.trim().to_owned())
}

/// A log of a cell directory, decompressed when only the `.zst` is left.
pub(crate) fn log_text(dir: &Path, name: &str) -> String {
    if let Ok(text) = std::fs::read_to_string(dir.join(name)) {
        return text;
    }
    let zst = dir.join(format!("{name}.zst"));
    if !zst.is_file() {
        return String::new();
    }
    Command::new("zstd")
        .args(["-dcq"])
        .arg(&zst)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// The first error of a build log: its first error line, or else the lines it stopped on.
fn first_of(log: &str) -> String {
    error_lines(log).first().map_or_else(
        || build::failure_lines(log, 6).join("\n"),
        |l| l.trim().to_owned(),
    )
}

/// The first bad line of a boot, from its `boot-N.json`: the panic, the first splat, or the time running out. Falls back to the first failed KUnit test, then to the last line of the console.
fn first_bad_line(dir: &Path, log: &str, console: &str) -> String {
    let json = Path::new(log).with_extension("json");
    let outcome: serde_json::Value = std::fs::read_to_string(dir.join(json))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    if let Some(p) = outcome.get("panic").and_then(|p| p.as_str()) {
        return p.to_owned();
    }
    if let Some(s) = outcome
        .get("splats")
        .and_then(|s| s.as_array())
        .and_then(|s| s.first())
        .and_then(|s| s.as_str())
    {
        return s.to_owned();
    }
    if outcome
        .get("timed-out")
        .or_else(|| outcome.get("timed_out"))
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        let last = console.lines().rev().find(|l| !l.trim().is_empty());
        return format!("timed out after: {}", last.unwrap_or("nothing"));
    }
    console
        .lines()
        .find(|l| l.trim_start().starts_with("not ok"))
        .or_else(|| console.lines().rev().find(|l| !l.trim().is_empty()))
        .unwrap_or_default()
        .trim()
        .to_owned()
}

/// What kept a cell from L7, from `kunit.json`: the graded suites that failed, or the lack of a reference to grade against.
fn kunit(dir: &Path, console: &str) -> String {
    let Some(k) = std::fs::read_to_string(dir.join("kunit.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<crate::cell::KunitRecord>(&t).ok())
    else {
        return first_bad_line(dir, "kunit-1.log", console);
    };
    if let Some((run, suites)) = k.failed.iter().enumerate().find(|(_, f)| !f.is_empty()) {
        return format!("KUnit run {}: {} failed", run + 1, suites.join(", "));
    }
    if k.reference.is_empty() {
        return "no reference cell to grade KUnit against".into();
    }
    first_bad_line(dir, "kunit-1.log", console)
}

/// What kept a cell that reached L7 from L8: the first `objtool` warning, then the first smoke splat, then the first new KUnit splat.
fn unclean(dir: &Path) -> String {
    let splats: crate::cell::SplatRecord = match std::fs::read_to_string(dir.join("splats.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
    {
        Some(s) => s,
        None => return String::new(),
    };
    splats
        .objtool
        .iter()
        .chain(splats.smoke.iter().flatten())
        .chain(splats.kunit.iter().flatten())
        .next()
        .cloned()
        .unwrap_or_default()
}

/// Why kbuild stopped, from `build.json`, when it says.
fn stopped(dir: &Path) -> Option<String> {
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("build.json")).ok()?).ok()?;
    let lines = json.get("stopped")?.get(1)?.as_array()?;
    let text: Vec<&str> = lines.iter().filter_map(|l| l.as_str()).collect();
    (!text.is_empty()).then(|| text.join("\n"))
}

/// The log of the L3 step that failed. That is make.log, except where `make dep` or `make scripts` failed first and make.log was never written.
fn build_log(dir: &Path) -> String {
    let make = log_text(dir, "make.log");
    if !make.trim().is_empty() {
        return make;
    }
    ["dep.log", "scripts.log"]
        .iter()
        .map(|name| log_text(dir, name))
        .find(|t| t.contains("***"))
        .unwrap_or_default()
}

/// Write errors.jsonl again from compile.jsonl when its first unit has no error line and compile.jsonl now finds one, as for a unit gas failed before its `Error:` lines counted. A store copied without compile.jsonl keeps the line then.
fn repair_errors(dir: &Path) -> Result<(), String> {
    let path = dir.join("errors.jsonl");
    let Ok(errors) = std::fs::read_to_string(&path) else {
        return Ok(());
    };
    let stored: Option<build::FailedUnit> =
        errors.lines().find_map(|l| serde_json::from_str(l).ok());
    if !stored.is_some_and(|u| u.error.is_empty()) {
        return Ok(());
    }
    let Ok((records, _)) = gk_cc::record::read_log(&dir.join("compile.jsonl")) else {
        return Ok(());
    };
    let units = build::failing_units(&records, Path::new("/src"));
    if units.first().is_none_or(|u| u.error.is_empty()) {
        return Ok(());
    }
    let mut lines = String::new();
    for u in &units {
        lines.push_str(&serde_json::to_string(u).map_err(|e| e.to_string())?);
        lines.push('\n');
    }
    std::fs::write(&path, lines).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// The first error of a build that failed with no failing unit, as when a link fails. The lines kbuild stopped on come first when they say what went wrong, but under `-j` they are often just the last commands of other jobs, and then the log's first error line is the one to go by.
fn no_unit(stopped: Option<String>, make: &str) -> String {
    if let Some(line) = gave_up_on(make) {
        return line;
    }
    match stopped {
        Some(s) if !error_lines(&s).is_empty() || error_lines(make).is_empty() => s,
        _ => first_of(make),
    }
}

/// The first failing unit and its command line, from `errors.jsonl` and `compile.jsonl`. A cell run before the prepare step's `-S` units counted has an empty `errors.jsonl` when one of them failed, so `compile.jsonl` is asked again then.
fn failing_unit(dir: &Path) -> Option<(build::FailedUnit, String)> {
    let records = gk_cc::record::read_log(&dir.join("compile.jsonl"))
        .ok()
        .map(|(r, _)| r);
    let errors = std::fs::read_to_string(dir.join("errors.jsonl")).unwrap_or_default();
    let stored: Option<build::FailedUnit> =
        errors.lines().find_map(|l| serde_json::from_str(l).ok());
    let first = match stored {
        Some(u) if !u.error.is_empty() => u,
        // A cell run before gas's `Error:` lines counted stored a unit that failed in the assembler with no error, and compile.jsonl still has its stderr.
        stored => records
            .as_deref()
            .and_then(|r| {
                build::failing_units(r, Path::new("/src"))
                    .into_iter()
                    .next()
            })
            .or(stored)?,
    };
    let command = records
        .and_then(|records| {
            records
                .into_iter()
                .filter(|r| build::is_unit(r) && !r.succeeded())
                .find(|r| build::unit_source(r, Path::new("/src")) == first.unit)
        })
        .map(|r| r.argv.join(" "))
        .unwrap_or_default();
    Some((first, command))
}

/// Read a failed cell, or `None` when every rung it climbed passed.
#[must_use]
pub fn failure(dir: &Path, r: &CellRecord) -> Option<Failure> {
    let rung = if r.probe.passes() {
        r.steps.iter().find(|s| !s.passed).map(|s| s.rung.clone())?
    } else {
        "L1".to_owned()
    };
    let mut f = Failure {
        config: std::fs::read_to_string(dir.join("config"))
            .or_else(|_| std::fs::read_to_string(dir.join(".config")))
            .unwrap_or_default(),
        ..Failure::default()
    };
    match rung.as_str() {
        "L1" => {
            let log = if r.probe.step == "init/main.i" {
                "probe-main.log"
            } else {
                "probe-config.log"
            };
            let text = std::fs::read_to_string(dir.join(log)).unwrap_or_default();
            let mut lines: Vec<&str> = r.probe.why.iter().map(String::as_str).collect();
            lines.extend(
                text.lines()
                    .filter(|l| l.contains("error") || l.starts_with("***")),
            );
            f.first_error = lines.join("\n");
        }
        "L2" => f.first_error = first_of(&log_text(dir, "config.log")),
        "L3" => {
            if let Some((unit, command)) = failing_unit(dir) {
                f.first_error = unit.error;
                f.unit = unit.unit;
                f.command = command;
            } else {
                f.first_error = no_unit(stopped(dir), &build_log(dir));
            }
        }
        "L4" => {
            let make = log_text(dir, "make.log");
            let modules = log_text(dir, "modules.log");
            let lines = error_lines(&make);
            let more = error_lines(&modules);
            f.first_error = lines
                .first()
                .or_else(|| more.first())
                .map_or_else(|| first_of(&modules), |l| l.trim().to_owned());
        }
        "L8" => f.first_error = unclean(dir),
        "L7" if r.steps.iter().any(|s| !s.passed && s.log == "kunit.json") => {
            f.console = std::fs::read_to_string(dir.join("kunit-1.log")).unwrap_or_default();
            f.first_error = kunit(dir, &f.console);
        }
        _ => {
            let log = r
                .steps
                .iter()
                .find(|s| !s.passed)
                .map(|s| s.log.clone())
                .unwrap_or_default();
            f.console = std::fs::read_to_string(dir.join(&log)).unwrap_or_default();
            f.first_error = first_bad_line(dir, &log, &f.console);
        }
    }
    f.rung = rung;
    Some(f)
}

/// The version at the end of a name such as `gcc-14.2.0` or `linux-7.2.8`.
fn version_of(name: &str) -> Option<Version> {
    name.rsplit('-').next()?.parse().ok()
}

/// A signature's patterns, compiled once.
pub struct Compiled<'a> {
    /// The signature.
    pub signature: &'a Signature,
    patterns: Vec<(&'static str, Regex)>,
}

/// Compile the catalog. `gk check` has already said which patterns do not compile, so those signatures are left out here.
#[must_use]
pub fn compile(repo: &Repo) -> Vec<Compiled<'_>> {
    repo.signatures
        .signatures
        .iter()
        .filter_map(|s| {
            let patterns = s
                .patterns
                .all()
                .into_iter()
                .map(|(part, p)| Regex::new(p).ok().map(|r| (part, r)))
                .collect::<Option<Vec<_>>>()?;
            Some(Compiled {
                signature: s,
                patterns,
            })
        })
        .collect()
}

/// What the catalog says about one failure.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Verdict {
    /// The class of the first error, if a signature names it.
    pub class: Option<String>,
    /// The signatures whose patterns match but whose ranges leave the cell out, with why.
    pub findings: Vec<String>,
}

/// The ranges of a signature that leave a cell out, by name.
fn outside(repo: &Repo, s: &Signature, r: &CellRecord) -> Vec<String> {
    let c = &r.coordinates;
    let mut out = Vec::new();
    for (what, text, name) in [
        ("gcc", &s.gcc, &c.gcc.name),
        ("binutils", &s.binutils, &c.binutils.name),
        ("kernel", &s.kernel, &c.kernel.name),
    ] {
        let (Ok(range), Some(v)) = (Range::parse(text), version_of(name)) else {
            continue;
        };
        if !range.contains(&v) {
            out.push(format!("{what} {v} is not {text}"));
        }
    }
    if !s.platform.is_empty() && !s.platform.contains(&c.platform) {
        out.push(format!(
            "platform {} is not {}",
            c.platform,
            s.platform.join(" or ")
        ));
    }
    if !s.flavor.is_empty() {
        let flavor = repo
            .gccs
            .get(&c.gcc.name)
            .map_or("upstream", |g| g.flavor.as_str());
        if !s.flavor.iter().any(|f| f == flavor) {
            out.push(format!("flavor {flavor} is not {}", s.flavor.join(" or ")));
        }
    }
    out
}

/// Run the catalog over one failure.
#[must_use]
pub fn judge(repo: &Repo, catalog: &[Compiled<'_>], r: &CellRecord, f: &Failure) -> Verdict {
    let mut v = Verdict::default();
    for c in catalog {
        if !rung_covers(&c.signature.rung, &f.rung)
            || !c
                .patterns
                .iter()
                .all(|(part, re)| re.is_match(f.part(part)))
        {
            continue;
        }
        let out = outside(repo, c.signature, r);
        if out.is_empty() {
            if v.class.is_none() {
                v.class = Some(c.signature.class.clone());
            }
        } else {
            v.findings
                .push(format!("{}: {}", c.signature.class, out.join(", ")));
        }
    }
    v
}

/// The classes of a failed cell: the first error's, and after a `make -k` run those of every failing unit, each once, with `unclassified` for a unit no signature names (spec 09.5).
#[must_use]
pub fn classes(
    repo: &Repo,
    catalog: &[Compiled<'_>],
    dir: &Path,
    r: &CellRecord,
    f: &Failure,
    v: &Verdict,
) -> Vec<String> {
    let mut out: Vec<String> = v.class.iter().cloned().collect();
    if f.rung != "L3" || r.units_failed.is_none() {
        return out;
    }
    let errors = std::fs::read_to_string(dir.join("errors.jsonl")).unwrap_or_default();
    for unit in errors
        .lines()
        .filter_map(|l| serde_json::from_str::<build::FailedUnit>(l).ok())
    {
        let each = Failure {
            rung: f.rung.clone(),
            first_error: unit.error,
            unit: unit.unit,
            config: f.config.clone(),
            ..Failure::default()
        };
        let class = judge(repo, catalog, r, &each)
            .class
            .unwrap_or_else(|| "unclassified".into());
        if !out.contains(&class) {
            out.push(class);
        }
    }
    out
}

/// `gk classify`: judge every failed cell in the store and write `classes` and `findings` into its `cell.json`. Returns the report.
pub fn classify(repo: &Repo, write: bool) -> Result<String, String> {
    let catalog = compile(repo);
    let mut out = String::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut failed = 0;
    let cells = store::cells()?;
    for (dir, r) in &cells {
        let (dir, mut r) = (dir.clone(), r.clone());
        if write {
            repair_errors(&dir)?;
            if crate::cell::regrade(repo, &dir, &mut r, &cells)? {
                let c = &r.coordinates;
                let _ = writeln!(
                    out,
                    "{} {} {} {}: graded again, {}",
                    c.kernel.name, c.gcc.name, c.platform, c.config.name, r.rung
                );
                let json = serde_json::to_string_pretty(&r).map_err(|e| e.to_string())?;
                std::fs::write(dir.join("cell.json"), json + "\n")
                    .map_err(|e| format!("writing {}/cell.json: {e}", dir.display()))?;
            }
        }
        let Some(f) = failure(&dir, &r) else {
            continue;
        };
        failed += 1;
        let v = judge(repo, &catalog, &r, &f);
        let class = v.class.clone().unwrap_or_else(|| "unclassified".into());
        *counts.entry(class.clone()).or_default() += 1;
        let c = &r.coordinates;
        let _ = writeln!(
            out,
            "{} {} {} {}: fails {}, {class}",
            c.kernel.name, c.gcc.name, c.platform, c.config.name, f.rung
        );
        for finding in &v.findings {
            let _ = writeln!(out, "  finding: {finding}");
        }
        let classes = classes(repo, &catalog, &dir, &r, &f, &v);
        if write && (r.classes != classes || r.findings != v.findings) {
            r.classes = classes;
            r.findings = v.findings;
            let json = serde_json::to_string_pretty(&r).map_err(|e| e.to_string())?;
            std::fs::write(dir.join("cell.json"), json + "\n")
                .map_err(|e| format!("writing {}/cell.json: {e}", dir.display()))?;
        }
    }
    let _ = writeln!(out, "\n{failed} failed cells");
    for (class, n) in &counts {
        let _ = writeln!(out, "{n:>5}  {class}");
    }
    Ok(out)
}

/// The text a first error clusters by: [`build::error_key`] of its first line that says error, or the line itself with numbers taken out.
#[must_use]
pub fn cluster_key(first_error: &str) -> String {
    build::error_key(first_error).unwrap_or_else(|| {
        let line = first_error.lines().next().unwrap_or("(nothing)");
        let mut out = String::new();
        for c in line.chars() {
            if c.is_ascii_digit() {
                if !out.ends_with('N') {
                    out.push('N');
                }
            } else {
                out.push(c);
            }
        }
        out.chars().take(160).collect()
    })
}

/// `gk triage [--all]`: the unclassified first errors, clustered by [`cluster_key`], largest cluster first. Only the newest cell of each kernel, GCC, platform and configuration counts, as in the matrix. With `all` every cell of a cluster is listed with its first error and its directory, not just the first five.
pub fn triage(repo: &Repo, all: bool) -> Result<String, String> {
    let catalog = compile(repo);
    let mut newest: BTreeMap<(String, String, String, String), (PathBuf, CellRecord)> =
        BTreeMap::new();
    for (dir, r) in store::cells()? {
        let c = &r.coordinates;
        let key = (
            c.kernel.name.clone(),
            c.gcc.name.clone(),
            c.platform.clone(),
            c.config.name.clone(),
        );
        if newest
            .get(&key)
            .is_none_or(|(_, old)| old.started < r.started)
        {
            newest.insert(key, (dir, r));
        }
    }
    let mut clusters: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for (dir, r) in newest.into_values() {
        let Some(f) = failure(&dir, &r) else {
            continue;
        };
        if judge(repo, &catalog, &r, &f).class.is_some() {
            continue;
        }
        let c = &r.coordinates;
        let mut line = format!(
            "{} {} {} {}",
            c.kernel.name, c.gcc.name, c.platform, c.config.name
        );
        if all {
            let first: String = f
                .first_error
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(160)
                .collect();
            let _ = write!(line, "\n          {first}\n          {}", dir.display());
        }
        clusters
            .entry((f.rung.clone(), cluster_key(&f.first_error)))
            .or_default()
            .push(line);
    }
    let mut sorted: Vec<_> = clusters.into_iter().collect();
    sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(&b.0)));
    let mut out = String::new();
    for ((rung, key), cells) in &sorted {
        let _ = writeln!(out, "{:>4}  {rung}  {key}", cells.len());
        let shown = if all { cells.len() } else { 5 };
        for cell in cells.iter().take(shown) {
            let _ = writeln!(out, "        {cell}");
        }
        if cells.len() > shown {
            let _ = writeln!(out, "        and {} more", cells.len() - 5);
        }
    }
    if sorted.is_empty() {
        out.push_str("Every failed cell has a class.\n");
    }
    Ok(out)
}

/// Whether a kernel has a signature's fix, as words: `in the tree`, `not in the tree` or `no tag to compare`.
fn fixed_words(fix: &gk_model::signatures::Fix, kernel: Option<&Version>) -> &'static str {
    match kernel.and_then(|k| fix.in_tree(k)) {
        Some(true) => "in the tree",
        Some(false) => "not in the tree",
        None => "no tag to compare",
    }
}

/// `gk explain K G P [--config C]`: the newest cell with those coordinates, its class, and whether the fix is in its tree.
pub fn explain(repo: &Repo, args: &[String]) -> Result<String, String> {
    let mut words = Vec::new();
    let mut config = "defconfig+gk".to_owned();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--config" {
            config.clone_from(it.next().ok_or("--config needs a configuration")?);
        } else {
            words.push(a.as_str());
        }
    }
    let [kernel, gcc, platform] = words[..] else {
        return Err("usage: gk explain K G P [--config C]".into());
    };
    let kernel = format!("linux-{}", kernel.trim_start_matches("linux-"));
    let (dir, r) = store::cells()?
        .into_iter()
        .rev()
        .find(|(_, r)| {
            let c = &r.coordinates;
            c.kernel.name == kernel
                && (c.gcc.name == gcc || c.gcc.name.strip_prefix("gcc-") == Some(gcc))
                && c.platform == platform
                && c.config.name == config
        })
        .ok_or_else(|| format!("no cell for {kernel} {gcc} {platform} {config} in the store"))?;
    let gcc = &r.coordinates.gcc.name;
    let mut out = format!(
        "{kernel} x {gcc} on {platform} ({config}): {}, reached {}\n",
        r.verdict,
        if r.rung.is_empty() {
            "nothing"
        } else {
            &r.rung
        }
    );
    let Some(f) = failure(&dir, &r) else {
        out.push_str("Every rung it climbed passed, so there is nothing to explain.\n");
        return Ok(out);
    };
    let _ = writeln!(
        out,
        "fails {}: {}",
        f.rung,
        f.first_error.lines().next().unwrap_or("")
    );
    if !f.unit.is_empty() {
        let _ = writeln!(out, "first failing unit: {}", f.unit);
    }
    let catalog = compile(repo);
    let v = judge(repo, &catalog, &r, &f);
    for finding in &v.findings {
        let _ = writeln!(out, "finding, outside a signature's ranges: {finding}");
    }
    let Some(class) = v.class else {
        out.push_str(
            "class: unclassified. `gk triage` shows it with the cells that fail the same way.\n",
        );
        return Ok(out);
    };
    let s = repo
        .signatures
        .get(&class)
        .ok_or("the class vanished from the catalog")?;
    let _ = writeln!(out, "class: {class} ({})\n{}", s.kind, s.summary);
    let version = version_of(&kernel);
    let mut in_tree = false;
    for fix in &s.fixed_by {
        let words = fixed_words(fix, version.as_ref());
        in_tree |= words == "in the tree";
        let tag = if fix.first_tag.is_empty() {
            String::new()
        } else {
            format!(" in {}", fix.first_tag)
        };
        let stable = if fix.stable.is_empty() {
            String::new()
        } else {
            format!(", backported to {}", fix.stable.join(", "))
        };
        let note = if fix.note.is_empty() {
            String::new()
        } else {
            format!(" ({})", fix.note)
        };
        let _ = writeln!(out, "fixed by {}{tag}{stable}{note}: {words}", fix.commit);
    }
    if in_tree {
        out.push_str("A fix is in the tree and the cell still fails, so the signature is wrong or there is a second problem.\n");
    }
    Ok(out)
}

/// The class and fixes of a failed cell, for `matrix.json` (spec 08.6): `None` for a cell that did not fail.
#[must_use]
pub fn published(
    repo: &Repo,
    catalog: &[Compiled<'_>],
    dir: &Path,
    r: &CellRecord,
) -> Option<(String, Vec<String>)> {
    let f = failure(dir, r)?;
    let v = judge(repo, catalog, r, &f);
    let Some(class) = v.class else {
        return Some(("unclassified".into(), Vec::new()));
    };
    let fixes = repo.signatures.get(&class).map_or_else(Vec::new, |s| {
        s.fixed_by
            .iter()
            .map(|f| {
                if f.first_tag.is_empty() {
                    f.commit.clone()
                } else {
                    format!("{} ({})", f.commit, f.first_tag)
                }
            })
            .collect()
    });
    Some((class, fixes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gk_model::signatures::Signatures;

    fn record(kernel: &str, gcc: &str, platform: &str) -> CellRecord {
        let json = format!(
            r#"{{"cell":"sha256:00","coordinates":{{"kernel":{{"name":"linux-{kernel}","digest":""}},"gcc":{{"name":"gcc-{gcc}","digest":""}},"binutils":{{"name":"binutils-2.44","digest":""}},"platform":"{platform}","config":{{"name":"defconfig+gk","digest":""}},"host":{{"name":"bookworm","digest":""}},"qemu":"","initramfs":""}},"rung":"L0","verdict":"fails","steps":[],"probe":{{"result":"refused","step":"init/main.i","seconds":1.0}},"era":"M12","started":0,"seconds":1.0,"machine":"x","gk":"0","graded":true}}"#
        );
        serde_json::from_str(&json).unwrap()
    }

    fn repo() -> Repo {
        let signatures: Signatures = toml::from_str(
            r#"
[[signature]]
class = "no-compiler-header"
rung = "L1"
match = { first-error = 'linux/compiler-gcc(\d+)\.h: No such file or directory' }
kind = "refusal"
summary = "One header per major"
gcc = ">=5"
kernel = ">=2.6.29, <=4.1"

[[signature]]
class = "gcc10-start-secondary"
rung = "L5"
match = { first-error = 'start_secondary' }
kind = "miscompile-by-assumption"
summary = "Tail call"
gcc = ">=10"
platform = ["x86_64"]
"#,
        )
        .unwrap();
        Repo {
            signatures,
            ..Repo::default()
        }
    }

    #[test]
    fn a_failure_in_range_is_classified_and_one_outside_is_a_finding() {
        let repo = repo();
        let catalog = compile(&repo);
        let f = Failure {
            rung: "L1".into(),
            first_error: "include/linux/compiler.h:54:28: fatal error: linux/compiler-gcc7.h: No such file or directory".into(),
            ..Failure::default()
        };
        let v = judge(&repo, &catalog, &record("3.18.140", "7.5.0", "x86_64"), &f);
        assert_eq!(v.class.as_deref(), Some("no-compiler-header"));
        assert!(v.findings.is_empty());
        let v = judge(&repo, &catalog, &record("4.4.302", "7.5.0", "x86_64"), &f);
        assert_eq!(v.class, None);
        assert_eq!(
            v.findings,
            ["no-compiler-header: kernel 4.4.302 is not >=2.6.29, <=4.1"]
        );
        let boot = Failure {
            rung: "L3".into(),
            ..f
        };
        assert_eq!(
            judge(
                &repo,
                &catalog,
                &record("3.18.140", "7.5.0", "x86_64"),
                &boot
            ),
            Verdict::default()
        );
    }

    #[test]
    fn rungs_and_platforms_limit_a_signature() {
        let repo = repo();
        let catalog = compile(&repo);
        let f = Failure {
            rung: "L5".into(),
            first_error: "Kernel panic - not syncing: stack-protector: Kernel stack is corrupted in: start_secondary+0x1a0/0x1b0".into(),
            ..Failure::default()
        };
        let v = judge(&repo, &catalog, &record("5.4.1", "10.1.0", "x86_64"), &f);
        assert_eq!(v.class.as_deref(), Some("gcc10-start-secondary"));
        let v = judge(&repo, &catalog, &record("5.4.1", "10.1.0", "arm64"), &f);
        assert_eq!(
            v.findings,
            ["gcc10-start-secondary: platform arm64 is not x86_64"]
        );
    }

    #[test]
    fn a_link_error_beats_stop_lines_that_say_nothing() {
        let make = "  LD      drivers/gpu/drm/i915/i915.o\ni915_irq.c:(.text+0x20f0): multiple definition of `intel_gmbus_is_forced_bit'\nmake[4]: *** [i915.o] Error 1\n";
        let quiet = "make -f /src/scripts/Makefile.build obj=net/sunrpc/auth_gss";
        assert_eq!(
            no_unit(Some(quiet.to_owned()), make),
            "i915_irq.c:(.text+0x20f0): multiple definition of `intel_gmbus_is_forced_bit'"
        );
        assert_eq!(no_unit(Some(quiet.to_owned()), "nothing\n"), quiet);
        let loud = "ld: final link failed: error: bad value";
        assert_eq!(no_unit(Some(loud.to_owned()), make), loud);
    }

    #[test]
    fn the_line_naming_the_target_make_gave_up_on_wins() {
        let stopped = "kernel/tsacct.o: warning: objtool: missing symbol for section .text";
        let new = "   ./tools/objtool/objtool orc generate  --no-fp --retpoline --uaccess arch/x86/entry/thunk_64.o\narch/x86/entry/thunk_64.o: warning: objtool: missing symbol table\nmake[3]: *** [/src/scripts/Makefile.build:369: arch/x86/entry/thunk_64.o] Error 1\n";
        let old = "arch/x86/entry/thunk_64.o: warning: objtool: missing symbol table\n/src/scripts/Makefile.build:405: recipe for target 'arch/x86/entry/thunk_64.o' failed\nmake[3]: *** [arch/x86/entry/thunk_64.o] Error 1\n";
        for make in [new, old] {
            assert_eq!(
                no_unit(Some(stopped.to_owned()), make),
                "arch/x86/entry/thunk_64.o: warning: objtool: missing symbol table"
            );
        }
    }

    #[test]
    fn a_museum_tree_whose_make_dep_failed_is_read_from_dep_log() {
        let dir = std::env::temp_dir().join(format!("gk-dep-log-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("dep.log"),
            "gcc: unrecognized option `-M'\nmake: *** [dep] Error 2\n",
        )
        .unwrap();
        assert!(build_log(&dir).contains("unrecognized option"));
        std::fs::write(
            dir.join("dep.log"),
            "make[1]: Leaving directory `/out/fs'\n",
        )
        .unwrap();
        std::fs::write(dir.join("make.log"), "fs/a.c:1: error: x\n").unwrap();
        assert_eq!(build_log(&dir), "fs/a.c:1: error: x\n");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn clusters_ignore_names_and_numbers() {
        assert_eq!(
            cluster_key("fs/a.c:12:3: error: 'foo' undeclared (first use in this function)"),
            cluster_key("fs/b.c:99:1: error: 'bar' undeclared (first use in this function)")
        );
        assert_eq!(
            cluster_key("timed out after: [ 12.5] x"),
            "timed out after: [ N.N] x"
        );
    }

    #[test]
    fn the_seed_catalog_reads_and_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let repo = Repo::load(&root).unwrap();
        assert!(repo.signatures.signatures.len() >= 47);
        assert_eq!(repo.signatures.check(), Vec::<String>::new());
        assert_eq!(compile(&repo).len(), repo.signatures.signatures.len());
    }
}
