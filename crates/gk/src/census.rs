//! The Current set's warning census (spec 11.5) and configuration differential (spec 11.3), which `gk publish` writes as `reports/warning-census.md` and `reports/config-differential.md`.
//!
//! Both read the newest cell at every crossing of a Current kernel with an upstream GCC column, so a rerun replaces the cell it reran, and a crossing that has not run yet is left out rather than counted as clean.

use crate::build::Warnings;
use crate::differential::config_of;
use crate::kconfig::{self, Config};
use crate::report::Cell;
use gk_model::Version;
use gk_model::repo::Repo;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// Platform, configuration, kernel and GCC version.
type Key = (String, String, Version, Version);

/// A symbol that changed on one kernel: the kernel, and the values before and after.
type Change = (Version, Option<String>, Option<String>);

/// The cells of one platform and configuration, per kernel, with their GCC versions.
type Kernels<'a> = BTreeMap<&'a Version, Vec<(&'a Version, &'a Cell)>>;

/// Symbols that change at every step, which the tables leave out so the probes stand out.
const EVERY_STEP: [&str; 2] = ["CC_VERSION_TEXT", "GCC_VERSION"];

/// The newest cell at every crossing of a Current kernel with an upstream GCC column.
fn latest<'a>(repo: &Repo, cells: &'a [Cell]) -> BTreeMap<Key, &'a Cell> {
    let current: Vec<Version> = repo
        .kernels
        .in_set("current")
        .iter()
        .map(|k| k.version.clone())
        .collect();
    let mut out: BTreeMap<Key, &Cell> = BTreeMap::new();
    for c in cells {
        let e = &c.entry;
        let Some(g) = repo.gccs.get(&e.gcc).filter(|g| g.flavor == "upstream") else {
            continue;
        };
        let Some(k) = e
            .kernel
            .parse::<Version>()
            .ok()
            .filter(|k| current.contains(k))
        else {
            continue;
        };
        let key = (e.platform.clone(), e.config.clone(), k, g.version.clone());
        if out
            .get(&key)
            .is_none_or(|old| old.entry.started < e.started)
        {
            out.insert(key, c);
        }
    }
    out
}

/// The configurations in the cells, in name order.
fn configs(latest: &BTreeMap<Key, &Cell>) -> BTreeSet<String> {
    latest.keys().map(|k| k.1.clone()).collect()
}

/// Text that is safe inside a markdown table cell.
fn escape(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

/// What one GCC column's cells add up to.
#[derive(Default)]
struct Column {
    cells: usize,
    warnings: usize,
    units: BTreeSet<String>,
    options: BTreeSet<String>,
    werror_units: BTreeSet<String>,
    fatal: usize,
}

/// What one warning option adds up to over the columns.
#[derive(Default)]
struct Seen {
    first: Option<Version>,
    last: Option<Version>,
    cells: usize,
    warnings: usize,
}

/// The warning census, as `reports/warning-census.md`.
#[must_use]
pub fn warnings(repo: &Repo, cells: &[Cell]) -> String {
    let latest = latest(repo, cells);
    let mut out = String::from(
        "# Warning census\n\nThe compiler's warnings on the Current set, per GCC column (spec 11.5). Each cell's `warnings.jsonl` has them by unit and option, and this page adds them up over the newest cell at every crossing of a Current kernel with an upstream column, on every platform. A unit built with `-Werror` that warns is a broken build waiting for a compiler that warns more, so those units are listed on their own. Written by `gk publish`.\n",
    );
    if latest.is_empty() {
        out.push_str("\nNo Current cell has run yet.\n");
        return out;
    }
    for config in configs(&latest) {
        let mine: Vec<(&Key, &&Cell)> = latest.iter().filter(|(k, _)| k.1 == config).collect();
        let mut columns: BTreeMap<&Version, Column> = BTreeMap::new();
        let mut options: BTreeMap<String, Seen> = BTreeMap::new();
        let mut werror: Vec<(&Key, &Warnings)> = Vec::new();
        for (key, c) in &mine {
            let col = columns.entry(&key.3).or_default();
            col.cells += 1;
            let mut seen = BTreeSet::new();
            for w in &c.warnings {
                let name = if w.option.is_empty() {
                    "(none named)".to_owned()
                } else {
                    w.option.clone()
                };
                col.warnings += w.count;
                col.fatal += w.fatal;
                col.units.insert(format!("{} {}", key.0, w.unit));
                col.options.insert(name.clone());
                let o = options.entry(name.clone()).or_default();
                o.warnings += w.count;
                if o.first.as_ref().is_none_or(|v| key.3 < *v) {
                    o.first = Some(key.3.clone());
                }
                if o.last.as_ref().is_none_or(|v| key.3 > *v) {
                    o.last = Some(key.3.clone());
                }
                if seen.insert(name) {
                    o.cells += 1;
                }
                if w.werror {
                    col.werror_units.insert(format!("{} {}", key.0, w.unit));
                    werror.push((key, w));
                }
            }
        }
        let _ = write!(
            out,
            "\n## {config}\n\n### By GCC column\n\n| GCC | Cells | Warnings | Per cell | Units | Options | Units under -Werror | Fatal |\n|---|--:|--:|--:|--:|--:|--:|--:|\n"
        );
        let mut chart = Vec::new();
        for (v, c) in &columns {
            #[allow(clippy::cast_precision_loss)]
            let per = c.warnings as f64 / c.cells as f64;
            let _ = writeln!(
                out,
                "| {v} | {} | {} | {per:.1} | {} | {} | {} | {} |",
                c.cells,
                c.warnings,
                c.units.len(),
                c.options.len(),
                c.werror_units.len(),
                c.fatal
            );
            chart.push((v.to_string(), per, format!("{per:.1}")));
        }
        if columns.values().all(|c| c.warnings == 0) {
            let _ = writeln!(out, "\nNo column warns on any of the {} cells.", mine.len());
            continue;
        }
        out.push_str("\nWarnings per cell:\n\n");
        crate::status::chart(&mut out, &chart);

        let mut ranked: Vec<_> = options.into_iter().collect();
        ranked.sort_by(|a, b| b.1.warnings.cmp(&a.1.warnings).then(a.0.cmp(&b.0)));
        out.push_str("\n### By option\n\nThe first column is the oldest one that gives the warning on any Current cell, which is the release that introduced it for these kernels.\n\n| Option | First column | Last column | Cells | Warnings |\n|---|---|---|--:|--:|\n");
        let show = |v: &Option<Version>| v.as_ref().map(ToString::to_string).unwrap_or_default();
        for (name, o) in &ranked {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                escape(name),
                show(&o.first),
                show(&o.last),
                o.cells,
                o.warnings
            );
        }

        under_werror(&mut out, &mut werror);
    }
    out
}

/// The warnings on units built with `-Werror`, the first hundred of them.
fn under_werror(out: &mut String, werror: &mut [(&Key, &Warnings)]) {
    out.push_str("\n### Under -Werror\n\n");
    if werror.is_empty() {
        out.push_str("No unit built with `-Werror` warns.\n");
    } else {
        werror.sort_by(|a, b| {
            (&a.0.3, &a.0.2, &a.0.0, &a.1.unit).cmp(&(&b.0.3, &b.0.2, &b.0.0, &b.1.unit))
        });
        let _ = write!(
            out,
            "{} warning lines on units built with `-Werror`.\n\n| GCC | Kernel | Platform | Unit | Option | Count | Fatal | First |\n|---|---|---|---|---|--:|--:|---|\n",
            werror.len()
        );
        for (key, w) in werror.iter().take(100) {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                key.3,
                key.2,
                key.0,
                escape(&w.unit),
                escape(&w.option),
                w.count,
                w.fatal,
                crate::report::cell_text(&w.first)
            );
        }
        if werror.len() > 100 {
            let _ = writeln!(
                out,
                "\nThe first 100 are shown, and `warnings.jsonl` in each cell has the rest."
            );
        }
    }
}

/// What changes between two neighbouring columns: the kernels compared, and per symbol the kernel and both values.
#[derive(Default)]
struct Step {
    kernels: Vec<Version>,
    symbols: BTreeMap<String, Vec<Change>>,
}

/// The configuration differential, as `reports/config-differential.md`.
#[must_use]
pub fn config_differential(repo: &Repo, cells: &[Cell]) -> String {
    let latest = latest(repo, cells);
    let mut out = format!(
        "# Configuration differential\n\nWhat the kernel's configuration notices about each GCC on the Current set (spec 11.3). For every kernel, the `.config` of each column is compared with the one of the next column that has a cell, and the symbols that change are gathered per step. `{}` and `{}` change at every step and are left out. A persona's `config-divergences.toml` in rucc-kernel must be a subset of these lines. Written by `gk publish`.\n",
        EVERY_STEP[0], EVERY_STEP[1]
    );
    let mut groups: BTreeMap<(&str, &str), Kernels> = BTreeMap::new();
    for (key, c) in &latest {
        if config_of(&c.dir).is_none() {
            continue;
        }
        groups
            .entry((&key.0, &key.1))
            .or_default()
            .entry(&key.2)
            .or_default()
            .push((&key.3, c));
    }
    if groups.is_empty() {
        out.push_str("\nNo Current cell with a configuration has run yet.\n");
        return out;
    }
    let load = |c: &Cell| -> Option<Config> { kconfig::load(&config_of(&c.dir)?).ok() };
    for ((platform, config), kernels) in groups {
        let mut steps: BTreeMap<(Version, Version), Step> = BTreeMap::new();
        for (kernel, columns) in kernels {
            // The columns are in version order already, from the key.
            for pair in columns.windows(2) {
                let ((v1, c1), (v2, c2)) = (pair[0], pair[1]);
                let (Some(a), Some(b)) = (load(c1), load(c2)) else {
                    continue;
                };
                let step = steps.entry((v1.clone(), v2.clone())).or_default();
                step.kernels.push(kernel.clone());
                for d in kconfig::diff(&a, &b) {
                    if EVERY_STEP.contains(&d.symbol.as_str()) {
                        continue;
                    }
                    step.symbols
                        .entry(d.symbol)
                        .or_default()
                        .push((kernel.clone(), d.from, d.to));
                }
            }
        }
        let _ = write!(out, "\n## {platform} {config}\n\n");
        if steps.is_empty() {
            out.push_str("No kernel has two columns to compare yet.\n");
            continue;
        }
        out.push_str("| Step | Kernels | Symbols that change |\n|---|--:|--:|\n");
        for ((v1, v2), s) in &steps {
            let _ = writeln!(
                out,
                "| {v1} to {v2} | {} | {} |",
                s.kernels.len(),
                s.symbols.len()
            );
        }
        for ((v1, v2), s) in &steps {
            if s.symbols.is_empty() {
                continue;
            }
            let _ = write!(
                out,
                "\n### {v1} to {v2}\n\nOver {}.\n\n| Symbol | Kernels | {v1} | {v2} |\n|---|---|---|---|\n",
                list(&s.kernels)
            );
            for (symbol, seen) in &s.symbols {
                let kernels: Vec<Version> = seen.iter().map(|x| x.0.clone()).collect();
                let which = if kernels.len() == s.kernels.len() {
                    "all".to_owned()
                } else {
                    list(&kernels)
                };
                let value = |pick: fn(&Change) -> &Option<String>| {
                    let first = pick(&seen[0]);
                    if seen.iter().all(|x| pick(x) == first) {
                        escape(first.as_deref().unwrap_or("(absent)"))
                    } else {
                        "varies".to_owned()
                    }
                };
                let _ = writeln!(
                    out,
                    "| {symbol} | {which} | {} | {} |",
                    value(|x| &x.1),
                    value(|x| &x.2)
                );
            }
        }
    }
    out
}

/// `5.10.270, 6.1.188 and 7.2.8`.
fn list(kernels: &[Version]) -> String {
    let names: Vec<String> = kernels.iter().map(ToString::to_string).collect();
    match names.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::publish::Entry;
    use std::path::{Path, PathBuf};

    fn cell(kernel: &Version, gcc: &str, dir: PathBuf, warnings: &[(&str, usize, bool)]) -> Cell {
        Cell {
            entry: Entry {
                cell: String::new(),
                kernel: kernel.to_string(),
                gcc: gcc.into(),
                binutils: String::new(),
                platform: "x86_64".into(),
                config: "defconfig+gk".into(),
                host: String::new(),
                rung: String::new(),
                verdict: "works".into(),
                failing_units: 0,
                first_error: String::new(),
                warnings: Some(warnings.iter().map(|w| w.1).sum()),
                runs: 3,
                flaky: false,
                gk: String::new(),
                date: String::new(),
                started: 1,
                seconds: 0.0,
                build_seconds: None,
                machine: String::new(),
            },
            dir,
            warnings: warnings
                .iter()
                .map(|(o, n, werror)| Warnings {
                    unit: "fs/x.c".into(),
                    option: (*o).into(),
                    count: *n,
                    fatal: 0,
                    werror: *werror,
                    first: format!("fs/x.c:1:1: warning: {o}"),
                })
                .collect(),
        }
    }

    #[test]
    fn the_census_counts_per_column_and_dates_each_option() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let k = repo.kernels.in_set("current")[0].version.clone();
        let none = PathBuf::from("/nonexistent");
        let cells = [
            cell(&k, "gcc-14.2.0", none.clone(), &[("-Wold", 2, false)]),
            cell(
                &k,
                "gcc-16.2.0",
                none.clone(),
                &[("-Wold", 1, false), ("-Wnew", 4, true)],
            ),
            cell(
                &"2.6.39".parse().unwrap(),
                "gcc-16.2.0",
                none,
                &[("-Wgone", 9, false)],
            ),
        ];
        let text = warnings(&repo, &cells);
        assert!(text.contains("| 14.2.0 | 1 | 2 | 2.0 | 1 | 1 | 0 | 0 |"));
        assert!(text.contains("| 16.2.0 | 1 | 5 | 5.0 | 1 | 2 | 1 | 0 |"));
        assert!(text.contains("| -Wnew | 16.2.0 | 16.2.0 | 1 | 4 |"));
        assert!(text.contains("| -Wold | 14.2.0 | 16.2.0 | 2 | 3 |"));
        assert!(!text.contains("-Wgone"));
        assert!(text.contains(&format!(
            "| 16.2.0 | {k} | x86_64 | fs/x.c | -Wnew | 4 | 0 |"
        )));
    }

    #[test]
    fn the_differential_gathers_symbols_per_step() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let current = repo.kernels.in_set("current");
        let (k1, k2) = (current[0].version.clone(), current[1].version.clone());
        let root = std::env::temp_dir().join(format!("gk-census-{}", std::process::id()));
        let write = |name: &str, text: &str| {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(".config"), text).unwrap();
            dir
        };
        let old = "CONFIG_GCC_VERSION=140200\nCONFIG_CC_HAS_X=y\n";
        let new = "CONFIG_GCC_VERSION=160200\nCONFIG_CC_HAS_X=y\nCONFIG_CC_HAS_Y=y\n";
        let cells = [
            cell(&k1, "gcc-14.2.0", write("a", old), &[]),
            cell(&k1, "gcc-16.2.0", write("b", new), &[]),
            cell(&k2, "gcc-14.2.0", write("c", old), &[]),
            cell(
                &k2,
                "gcc-16.2.0",
                write("d", old.replace("1402", "1602").as_str()),
                &[],
            ),
        ];
        let text = config_differential(&repo, &cells);
        std::fs::remove_dir_all(&root).unwrap();
        assert!(text.contains("| 14.2.0 to 16.2.0 | 2 | 1 |"));
        assert!(text.contains(&format!("| CC_HAS_Y | {k1} | (absent) | y |")));
        assert!(!text.contains("| GCC_VERSION |"));
        assert_eq!(list(&[k1.clone(), k2.clone()]), format!("{k1} and {k2}"));
    }
}
