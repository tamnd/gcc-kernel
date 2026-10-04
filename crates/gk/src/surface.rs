//! What the kernel notices about a GCC, release by release (spec 11.3): `gk gates K`, `gk flags-diff K G1 G2` and `gk report persona-surface`.
//!
//! A persona only works if the kernel sees the same compiler in rucc as in the GCC it stands for, and the kernel looks in three places. Its sources test the version outright, in `#if` lines, Makefiles and Kconfig, and those gates are read from the tree. Kconfig probes the compiler and writes the answers into `.config`, which every cell keeps. kbuild probes it again with `cc-option` and passes a flag or leaves it out, and the cells that kept their `compile.jsonl` have every command line. The report puts the three side by side for each GCC column over the Current set.

use crate::build::is_unit;
use crate::census::{EVERY_STEP, escape, latest, list};
use crate::differential::config_of;
use crate::kconfig;
use crate::report::Cell;
use crate::store;
use gk_cc::record::read_log;
use gk_model::Version;
use gk_model::repo::Repo;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// One place a tree tests the compiler's version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    /// The file, relative to the tree.
    pub file: String,
    /// The line number, from 1.
    pub line: usize,
    /// The first GCC release that gets the other answer.
    pub flips: Version,
    /// The line itself, trimmed.
    pub text: String,
}

/// A version from its three numbers.
fn version(major: u32, minor: u32, patch: u32) -> Option<Version> {
    format!("{major}.{minor}.{patch}").parse().ok()
}

/// Where a test against `at` flips: at `at` itself for `>=` and `<`, one step later for `>` and `<=`. An equality test is listed at the version it names.
fn after(op: &str, at: (u32, u32, u32), step: (u32, u32, u32)) -> Option<Version> {
    let (a, b, c) = at;
    if matches!(op, ">" | "<=" | "-gt" | "-le") {
        version(a + step.0, b + step.1, c + step.2)
    } else {
        version(a, b, c)
    }
}

/// The patterns of a version test, built once per scan.
struct Patterns {
    gcc_version: Regex,
    major: Regex,
    minor: Regex,
    ifversion: Regex,
    min_version: Regex,
}

impl Patterns {
    fn new() -> Self {
        let re = |s: &str| Regex::new(s).expect("a valid pattern");
        Self {
            gcc_version: re(r"\bGCC_VERSION\s*(>=|<=|==|!=|>|<)\s*(\d{5,6})\b"),
            major: re(r"\b__GNUC__\s*(>=|<=|==|!=|>|<)\s*(\d+)\b"),
            minor: re(r"\b__GNUC_MINOR__\s*(>=|<=|==|!=|>|<)\s*(\d+)\b"),
            ifversion: re(r"cc-ifversion,\s*-(ge|gt|lt|le|eq|ne),\s*(\d{4})\b"),
            min_version: re(r"gcc-min-version,\s*(\d{5,6})\b"),
        }
    }

    /// The versions at which one line's tests flip.
    fn flips(&self, line: &str) -> Vec<Version> {
        let mut out = Vec::new();
        let num = |s: &str| s.parse::<u32>().unwrap_or(0);
        for c in self.gcc_version.captures_iter(line) {
            let n = num(&c[2]);
            out.extend(after(&c[1], (n / 10000, n / 100 % 100, n % 100), (0, 0, 1)));
        }
        for c in self.min_version.captures_iter(line) {
            let n = num(&c[1]);
            out.extend(version(n / 10000, n / 100 % 100, n % 100));
        }
        for c in self.ifversion.captures_iter(line) {
            let n = num(&c[2]);
            out.extend(after(&c[1], (n / 100, n % 100, 0), (0, 1, 0)));
        }
        // `__GNUC__ == 4 && __GNUC_MINOR__ >= 6` is a test of 4.6. A line that tests the minor number is read that way, and its plain tests of the major number only bound it.
        let majors: Vec<(String, u32)> = self
            .major
            .captures_iter(line)
            .map(|c| (c[1].to_owned(), num(&c[2])))
            .collect();
        let minors: Vec<(String, u32)> = self
            .minor
            .captures_iter(line)
            .map(|c| (c[1].to_owned(), num(&c[2])))
            .collect();
        if let (Some((_, m)), false) = (majors.iter().find(|(op, _)| op == "=="), minors.is_empty())
        {
            for (op, n) in &minors {
                out.extend(after(op, (*m, *n, 0), (0, 1, 0)));
            }
        } else {
            for (op, m) in &majors {
                out.extend(after(op, (*m, 0, 0), (1, 0, 0)));
            }
        }
        out.sort();
        out.dedup();
        out
    }
}

/// Whether a file is one kbuild reads: C, headers, assembler, Makefiles and Kconfig.
fn scanned(name: &str) -> bool {
    [".c", ".h", ".S"].iter().any(|e| name.ends_with(e))
        || ["Makefile", "Kbuild", "Kconfig"]
            .iter()
            .any(|p| name.starts_with(p))
}

/// Every version gate in a tree, by the version it flips at. `Documentation` and `tools` are left out, since the kernel build reads neither.
#[must_use]
pub fn gates(tree: &Path) -> Vec<Gate> {
    let patterns = Patterns::new();
    let mut out = Vec::new();
    let mut dirs = vec![tree.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                let top = dir == tree;
                if !(top && (name == "Documentation" || name == "tools")) && !name.starts_with('.')
                {
                    dirs.push(path);
                }
                continue;
            }
            if !kind.is_file() || !scanned(&name) {
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let text = String::from_utf8_lossy(&bytes);
            if !["GCC_VERSION", "__GNUC", "cc-ifversion", "gcc-min-version"]
                .iter()
                .any(|w| text.contains(w))
            {
                continue;
            }
            let file = path
                .strip_prefix(tree)
                .map_or_else(|_| path.display().to_string(), |p| p.display().to_string());
            for (i, line) in text.lines().enumerate() {
                for flips in patterns.flips(line) {
                    out.push(Gate {
                        file: file.clone(),
                        line: i + 1,
                        flips,
                        text: line.trim().to_owned(),
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| (&a.flips, &a.file, a.line).cmp(&(&b.flips, &b.file, b.line)));
    out
}

/// The upstream GCC columns, oldest first, one per version.
fn columns(repo: &Repo) -> Vec<(String, Version)> {
    let mut out: Vec<(String, Version)> = repo
        .gccs
        .gccs
        .iter()
        .filter(|g| g.flavor == "upstream")
        .map(|g| (g.id.clone(), g.version.clone()))
        .collect();
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out.dedup_by(|a, b| a.1 == b.1);
    out
}

/// The first column that gets the other answer from a gate, or `None` when no pinned GCC is that new yet.
fn first_column<'a>(
    columns: &'a [(String, Version)],
    flips: &Version,
) -> Option<&'a (String, Version)> {
    columns.iter().find(|(_, v)| v >= flips)
}

/// `gk gates K`: every version gate in the kernel's tree, with the first column that sees it flip.
pub fn gates_command(repo: &Repo, args: &[String]) -> Result<String, String> {
    let [kernel] = args else {
        return Err("usage: gk gates K".into());
    };
    let v: Version = kernel
        .trim_start_matches("linux-")
        .parse()
        .map_err(|e| format!("kernel {kernel}: {e}"))?;
    let k = repo
        .kernels
        .get(&v)
        .ok_or_else(|| format!("{v} is not in kernels.toml"))?;
    let tree = crate::cell::fetched_tree(repo, &v, k.file_name())?;
    let cols = columns(repo);
    let found = gates(&tree);
    let mut out = format!(
        "# Version gates in {v}\n\n{} places in the tree test the GCC version. Each flips at the release given, and the column is the first pinned GCC that gets the other answer.\n\n| Flips at | Column | Where | Test |\n|---|---|---|---|\n",
        found.len()
    );
    for g in &found {
        let column = first_column(&cols, &g.flips).map_or("none yet", |c| c.0.as_str());
        let _ = writeln!(
            out,
            "| {} | {column} | {}:{} | `{}` |",
            g.flips,
            g.file,
            g.line,
            escape(&g.text).replace('`', "'")
        );
    }
    Ok(out)
}

/// How many units of a build were given each flag. Only the options that change code generation or diagnostics are counted: `-f`, `-m`, `-W`, `-O`, `-g`, `-std=` and `--param`. Paths and defines are left out, since they name the tree and not the compiler.
#[must_use]
pub fn unit_flags(path: &Path) -> Option<BTreeMap<String, usize>> {
    let (records, _) = read_log(path).ok()?;
    let mut out: BTreeMap<String, usize> = BTreeMap::new();
    for r in records.iter().filter(|r| is_unit(r)) {
        let mut seen = BTreeSet::new();
        let mut words = r.argv.iter().skip(1);
        while let Some(w) = words.next() {
            let flag = if w == "--param" {
                format!("--param={}", words.next().map_or("", String::as_str))
            } else if w.starts_with("--param=")
                || w.starts_with("-std=")
                || (["-f", "-m", "-W", "-O", "-g"]
                    .iter()
                    .any(|p| w.starts_with(p))
                    && !w.starts_with("-Wp,"))
            {
                w.clone()
            } else {
                continue;
            };
            seen.insert(flag);
        }
        for f in seen {
            *out.entry(f).or_default() += 1;
        }
    }
    Some(out)
}

/// The flags one build passes and the other does not: the flag, and how many units had it in each.
#[must_use]
pub fn flag_changes(
    from: &BTreeMap<String, usize>,
    to: &BTreeMap<String, usize>,
) -> Vec<(String, usize, usize)> {
    let flags: BTreeSet<&String> = from.keys().chain(to.keys()).collect();
    flags
        .into_iter()
        .map(|f| {
            (
                f.clone(),
                from.get(f).copied().unwrap_or(0),
                to.get(f).copied().unwrap_or(0),
            )
        })
        .filter(|(_, a, b)| (*a == 0) != (*b == 0))
        .collect()
}

/// The newest cell directory in the store for one crossing.
fn cell_dir(
    kernel: &Version,
    gcc: &str,
    platform: &str,
    config: &str,
) -> Result<Option<PathBuf>, String> {
    let mut found: Option<(PathBuf, u64)> = None;
    for (dir, r) in store::cells()? {
        let c = &r.coordinates;
        let same_gcc = c.gcc.name == gcc || c.gcc.name.strip_prefix("gcc-") == Some(gcc);
        let k = c
            .kernel
            .name
            .trim_start_matches("linux-")
            .parse::<Version>();
        if same_gcc
            && c.platform == platform
            && c.config.name == config
            && k.as_ref() == Ok(kernel)
            && found.as_ref().is_none_or(|f| f.1 < r.started)
        {
            found = Some((dir, r.started));
        }
    }
    Ok(found.map(|f| f.0))
}

/// `gk flags-diff K G1 G2 --platform P [--config C]`: the flags kbuild passes with one GCC and not the other.
pub fn flags_command(args: &[String]) -> Result<String, String> {
    let usage = "usage: gk flags-diff K G1 G2 --platform P [--config C]";
    let (mut words, mut platform, mut config) = (Vec::new(), None, crate::cell::CONFIG.to_owned());
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--platform" => platform = Some(it.next().ok_or("--platform needs a value")?.clone()),
            "--config" => config.clone_from(it.next().ok_or("--config needs a value")?),
            other if !other.starts_with('-') => words.push(other.to_owned()),
            other => return Err(format!("gk flags-diff: unexpected {other:?}")),
        }
    }
    let (Some(platform), [kernel, g1, g2]) = (platform, words.as_slice()) else {
        return Err(usage.into());
    };
    let v: Version = kernel
        .trim_start_matches("linux-")
        .parse()
        .map_err(|e| format!("kernel {kernel}: {e}"))?;
    let load = |g: &str| -> Result<BTreeMap<String, usize>, String> {
        let dir = cell_dir(&v, g, &platform, &config)?
            .ok_or_else(|| format!("no cell for {v} with {g} on {platform} in the store"))?;
        unit_flags(&dir.join("compile.jsonl"))
            .ok_or_else(|| format!("the cell of {v} with {g} on {platform} kept no compile.jsonl"))
    };
    let (a, b) = (load(g1)?, load(g2)?);
    let changes = flag_changes(&a, &b);
    let mut out = format!("# Flags of {v} on {platform}, {config}, {g1} to {g2}\n\n");
    if changes.is_empty() {
        out.push_str("Every flag is passed with both or with neither.\n");
        return Ok(out);
    }
    let _ = write!(
        out,
        "| Flag | Units with {g1} | Units with {g2} |\n|---|--:|--:|\n"
    );
    for (f, x, y) in changes {
        let _ = writeln!(out, "| `{f}` | {x} | {y} |");
    }
    Ok(out)
}

/// What changes at one column, from the column before it that has a cell.
#[derive(Default)]
struct Step {
    /// Gates by file and text: the version they flip at and the kernels that have them.
    gates: BTreeMap<(String, String), (Version, BTreeSet<Version>)>,
    /// Kconfig symbols by name and the column compared with: platforms, kernels, and the values seen before and after.
    symbols: BTreeMap<(String, Version), Seen>,
    /// Flags by name, whether they start, and the column compared with: platforms and kernels.
    flags: BTreeMap<(String, bool, Version), Seen>,
}

/// Where one change was seen.
#[derive(Default)]
struct Seen {
    platforms: BTreeSet<String>,
    kernels: BTreeSet<Version>,
    before: BTreeSet<String>,
    after: BTreeSet<String>,
}

/// One value, or `varies` when the kernels disagree.
fn one(values: &BTreeSet<String>) -> String {
    match values.len() {
        1 => escape(values.iter().next().map_or("", String::as_str)),
        _ => "varies".into(),
    }
}

/// The kernels of a change, or `all` when every kernel of the set has it.
fn which(kernels: &BTreeSet<Version>, all: usize) -> String {
    if kernels.len() == all {
        "all".into()
    } else {
        list(&kernels.iter().cloned().collect::<Vec<_>>())
    }
}

/// The gates of each tree, under the first column that gets the other answer from them.
fn gate_steps(cols: &[(String, Version)], trees: &[(Version, PathBuf)], steps: &mut BTreeMap<Version, Step>) {
    for (kernel, tree) in trees {
        for g in gates(tree) {
            let Some((_, at)) = first_column(cols, &g.flips) else {
                continue;
            };
            let entry = steps
                .entry(at.clone())
                .or_default()
                .gates
                .entry((g.file, g.text))
                .or_insert_with(|| (g.flips.clone(), BTreeSet::new()));
            entry.1.insert(kernel.clone());
        }
    }
}

/// The newest cells of one platform, configuration and kernel, with their GCC versions, oldest GCC first.
type Rows<'a> = BTreeMap<(&'a str, &'a str, &'a Version), Vec<(&'a Version, &'a Cell)>>;

/// The Kconfig symbols and flags that change between neighbouring columns with cells, under the newer column. Returns how many kernels had two configurations to compare.
fn cell_steps(repo: &Repo, cells: &[Cell], steps: &mut BTreeMap<Version, Step>) -> usize {
    let newest = latest(repo, cells);
    let mut by_row: Rows = BTreeMap::new();
    for ((platform, config, kernel, gcc), c) in &newest {
        by_row
            .entry((platform, config, kernel))
            .or_default()
            .push((gcc, c));
    }
    let mut compared = BTreeSet::new();
    for ((platform, _, kernel), columns) in &by_row {
        for pair in columns.windows(2) {
            let ((v1, c1), (v2, c2)) = (pair[0], pair[1]);
            let step = steps.entry(v2.clone()).or_default();
            let load = |c: &Cell| config_of(&c.dir).and_then(|p| kconfig::load(&p).ok());
            if let (Some(a), Some(b)) = (load(c1), load(c2)) {
                compared.insert((*kernel).clone());
                for d in kconfig::diff(&a, &b) {
                    if EVERY_STEP.contains(&d.symbol.as_str()) {
                        continue;
                    }
                    let seen = step.symbols.entry((d.symbol, v1.clone())).or_default();
                    seen.platforms.insert((*platform).to_owned());
                    seen.kernels.insert((*kernel).clone());
                    seen.before
                        .insert(d.from.unwrap_or_else(|| "(absent)".into()));
                    seen.after.insert(d.to.unwrap_or_else(|| "(absent)".into()));
                }
            }
            let flags = |c: &Cell| unit_flags(&c.dir.join("compile.jsonl"));
            if let (Some(a), Some(b)) = (flags(c1), flags(c2)) {
                for (f, x, y) in flag_changes(&a, &b) {
                    let seen = step.flags.entry((f, x == 0, v1.clone())).or_default();
                    seen.platforms.insert((*platform).to_owned());
                    seen.kernels.insert((*kernel).clone());
                    seen.before.insert(x.to_string());
                    seen.after.insert(y.to_string());
                }
            }
        }
    }
    compared.len()
}

/// One column's section of the report.
fn section(out: &mut String, title: &str, s: &Step, name: &dyn Fn(&Version) -> String, all: usize, compared: usize) {
    if s.gates.is_empty() && s.symbols.is_empty() && s.flags.is_empty() {
        return;
    }
    let _ = write!(out, "\n## {title}\n");
    if !s.gates.is_empty() {
        out.push_str(
            "\n### Version gates\n\n| Flips at | Where | Test | Kernels |\n|---|---|---|---|\n",
        );
        let mut rows: Vec<_> = s.gates.iter().collect();
        rows.sort_by(|a, b| (&a.1.0, &a.0).cmp(&(&b.1.0, &b.0)));
        for ((file, text), (flips, ks)) in rows {
            let _ = writeln!(
                out,
                "| {flips} | {file} | `{}` | {} |",
                escape(text).replace('`', "'"),
                which(ks, all)
            );
        }
    }
    if !s.symbols.is_empty() {
        out.push_str("\n### Kconfig symbols\n\n| Symbol | From | Platforms | Kernels | Before | After |\n|---|---|---|---|---|---|\n");
        for ((symbol, from), seen) in &s.symbols {
            let _ = writeln!(
                out,
                "| {symbol} | {} | {} | {} | {} | {} |",
                name(from),
                seen.platforms
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
                which(&seen.kernels, compared),
                one(&seen.before),
                one(&seen.after)
            );
        }
    }
    if !s.flags.is_empty() {
        out.push_str(
            "\n### Flags\n\n| Flag | Change | From | Platforms | Kernels |\n|---|---|---|---|---|\n",
        );
        for ((flag, starts, from), seen) in &s.flags {
            let _ = writeln!(
                out,
                "| `{flag}` | {} | {} | {} | {} |",
                if *starts { "starts" } else { "stops" },
                name(from),
                seen.platforms
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
                list(&seen.kernels.iter().cloned().collect::<Vec<_>>())
            );
        }
    }
}

/// The persona surface, as `reports/persona-surface.md`, from the Current set's newest cells and the trees of its kernels.
#[must_use]
pub fn persona_surface(repo: &Repo, cells: &[Cell], trees: &[(Version, PathBuf)]) -> String {
    let cols = columns(repo);
    let mut steps: BTreeMap<Version, Step> = BTreeMap::new();
    gate_steps(&cols, trees, &mut steps);
    let compared = cell_steps(repo, cells, &mut steps);
    let kernels: Vec<Version> = trees.iter().map(|(k, _)| k.clone()).collect();
    let mut out = format!(
        "# Persona surface\n\nWhat the kernel notices about each GCC release (spec 11.3), over the Current set: {}. A persona that stands for a GCC has to give the same answers to all of it. Written by `gk report persona-surface`.\n\nThree things are listed per GCC column. The version gates are the places the sources test the version outright, in `#if` lines, Makefiles and Kconfig, read from the trees and placed at the first pinned column that gets the other answer. The Kconfig symbols are the `.config` lines that differ from the column before that has a cell, from the newest cells, with `{}` and `{}` left out since they change at every step. The flags are the ones kbuild starts or stops passing to the units, from the cells that kept their `compile.jsonl`. A column with nothing in it is left out of the sections.\n",
        list(&kernels),
        EVERY_STEP[0],
        EVERY_STEP[1]
    );
    if steps.is_empty() {
        out.push_str(
            "\nNothing to list yet: no Current tree was read and no Current cell has run.\n",
        );
        return out;
    }
    out.push_str("\n| Column | Gates | Kconfig symbols | Flags |\n|---|--:|--:|--:|\n");
    let name = |v: &Version| {
        cols.iter()
            .find(|(_, x)| x == v)
            .map_or_else(|| v.to_string(), |(id, _)| id.clone())
    };
    for (v, s) in &steps {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            name(v),
            s.gates.len(),
            s.symbols.len(),
            s.flags.len()
        );
    }
    let all = kernels.len();
    for (v, s) in &steps {
        section(&mut out, &name(v), s, &name, all, compared);
    }
    out
}

/// `gk report persona-surface`: the trees of the Current set, fetched where they are missing, and the newest cells.
pub fn report(repo: &Repo, cells: &[Cell]) -> Result<String, String> {
    let mut trees = Vec::new();
    for k in repo.kernels.in_set("current") {
        trees.push((
            k.version.clone(),
            crate::cell::fetched_tree(repo, &k.version, k.file_name())?,
        ));
    }
    Ok(persona_surface(repo, cells, &trees))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(line: &str) -> Vec<String> {
        Patterns::new()
            .flips(line)
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn each_kind_of_gate_is_read_at_the_release_it_flips() {
        assert_eq!(at("#if GCC_VERSION >= 110100"), ["11.1.0"]);
        assert_eq!(
            at("\tdepends on GCC_VERSION > 80000 || CC_IS_CLANG"),
            ["8.0.1"]
        );
        assert_eq!(at("#if __GNUC__ < 5"), ["5.0.0"]);
        assert_eq!(
            at("#if __GNUC__ > 4 || (__GNUC__ == 4 && __GNUC_MINOR__ >= 6)"),
            ["4.6.0"]
        );
        assert_eq!(
            at("KBUILD_CFLAGS += $(call cc-ifversion, -lt, 0409, -fno-x)"),
            ["4.9.0"]
        );
        assert_eq!(at("ifeq ($(call gcc-min-version, 120100),y)"), ["12.1.0"]);
        assert!(at("#define GCC_VERSION (__GNUC__ * 10000)").is_empty());
    }

    #[test]
    fn a_tree_is_scanned_for_gates_outside_documentation_and_tools() {
        let tree = std::env::temp_dir().join(format!("gk-gates-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tree);
        for (file, text) in [
            (
                "include/linux/compiler-gcc.h",
                "#if GCC_VERSION < 50100\n#error old\n#endif\n",
            ),
            (
                "arch/x86/Makefile",
                "x := $(call cc-ifversion, -ge, 0800, y)\n",
            ),
            ("Documentation/x.h", "#if GCC_VERSION >= 90000\n"),
            ("tools/x.c", "#if __GNUC__ >= 9\n"),
            ("init/README", "GCC_VERSION >= 90000\n"),
        ] {
            let p = tree.join(file);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
        let found = gates(&tree);
        let _ = std::fs::remove_dir_all(&tree);
        let seen: Vec<(String, usize, String)> = found
            .iter()
            .map(|g| (g.file.clone(), g.line, g.flips.to_string()))
            .collect();
        assert_eq!(
            seen,
            [
                (
                    "include/linux/compiler-gcc.h".to_owned(),
                    1,
                    "5.1.0".to_owned()
                ),
                ("arch/x86/Makefile".to_owned(), 1, "8.0.0".to_owned()),
            ]
        );
    }

    #[test]
    fn only_flags_one_side_lacks_are_changes() {
        let a: BTreeMap<String, usize> = [("-fa".into(), 10), ("-fb".into(), 3)].into();
        let b: BTreeMap<String, usize> = [("-fa".into(), 9), ("-fc".into(), 4)].into();
        assert_eq!(
            flag_changes(&a, &b),
            [("-fb".to_owned(), 3, 0), ("-fc".to_owned(), 0, 4)]
        );
    }

    #[test]
    fn unit_flags_count_units_and_leave_probes_and_paths_out() {
        let dir = std::env::temp_dir().join(format!("gk-flags-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let log = dir.join("compile.jsonl");
        std::fs::write(
            &log,
            concat!(
                r#"{"started":1,"argv":["gk-cc","-O2","-fno-pie","-I","include","-DX","-Wp,-MMD,a.d","-c","a.c","-o","a.o"],"compiler":"/g","cwd":"/out","inputs":[{"path":"a.c","sha256":""}],"wall-seconds":0.1,"exit":0}"#,
                "\n",
                r#"{"started":2,"argv":["gk-cc","-O2","--param","x=1","-c","b.c","-o","b.o"],"compiler":"/g","cwd":"/out","inputs":[{"path":"b.S","sha256":""}],"wall-seconds":0.1,"exit":0}"#,
                "\n",
                r#"{"started":3,"argv":["gk-cc","-fprobe","-c","-x","c","/dev/null","-o","/dev/null"],"compiler":"/g","cwd":"/out","wall-seconds":0.1,"exit":0,"probe":true}"#,
                "\n"
            ),
        )
        .unwrap();
        let flags = unit_flags(&log).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        let want: BTreeMap<String, usize> = [
            ("--param=x=1".into(), 1),
            ("-O2".into(), 2),
            ("-fno-pie".into(), 1),
        ]
        .into();
        assert_eq!(flags, want);
    }
}
