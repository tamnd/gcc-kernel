//! `gk report new-gcc G`: the release report for a newly added GCC (spec 10.8).
//!
//! The report reads the result store and says four things about the Current set: which kernels build and run with the new GCC on each platform, which ones the previous release of the same series got further with, which warnings are new, and how the kernels' configurations differ from the previous column's. A crossing with no cell is reported as not run rather than guessed at, so the report can be written while a sweep is still going and written again when it is done.

use crate::build::Warnings;
use crate::differential;
use crate::publish::{self, Entry};
use crate::store;
use gk_model::Version;
use gk_model::repo::Repo;
use gk_model::toolchains::Gcc;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

/// A cell as the report needs it.
pub struct Cell {
    /// Its matrix record.
    pub entry: Entry,
    /// Its directory in the store.
    pub dir: PathBuf,
    /// Its warning census.
    pub warnings: Vec<Warnings>,
}

/// How far a verdict got, for comparing two of them.
fn rank(verdict: &str) -> u8 {
    match verdict {
        "works" => 3,
        "runs" => 2,
        "builds" => 1,
        _ => 0,
    }
}

/// The name the report goes under: `16.2` for 16.2.0, the full version otherwise.
#[must_use]
pub fn short(version: &Version) -> String {
    let parts = version.parts();
    if parts.len() == 3 && parts[2] == 0 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        version.to_string()
    }
}

/// The upstream GCC just before `gcc`, and the newest before it in the same major series.
fn previous<'a>(repo: &'a Repo, gcc: &Gcc) -> (Option<&'a Gcc>, Option<&'a Gcc>) {
    let older = || {
        repo.gccs
            .gccs
            .iter()
            .filter(|g| g.flavor == "upstream" && g.version < gcc.version)
    };
    let column = older().max_by(|a, b| a.version.cmp(&b.version));
    let series = older()
        .filter(|g| g.version.parts().first() == gcc.version.parts().first())
        .max_by(|a, b| a.version.cmp(&b.version));
    (column, series)
}

/// Every graded cell in the store with its warnings, or every cell with `ungraded`.
pub fn cells(ungraded: bool) -> Result<Vec<Cell>, String> {
    Ok(store::cells()?
        .into_iter()
        .filter(|(_, r)| ungraded || r.graded)
        .map(|(dir, r)| {
            let warnings = std::fs::read_to_string(dir.join("warnings.jsonl"))
                .unwrap_or_default()
                .lines()
                .filter_map(|l| serde_json::from_str(l).ok())
                .collect();
            Cell {
                entry: publish::entry(&dir, &r),
                dir,
                warnings,
            }
        })
        .collect())
}

/// The report, from the cells given.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render(repo: &Repo, gcc: &Gcc, config: &str, cells: &[Cell]) -> String {
    let kernels = repo.kernels.in_set("current");
    let platforms: Vec<_> = repo
        .platforms
        .platforms
        .iter()
        .filter(|p| gcc.targets.contains(&p.triple))
        .collect();
    let (column, series) = previous(repo, gcc);
    // A rerun is a new cell with the same names, and the newest one counts.
    let find = |kernel: &Version, id: &str, platform: &str| {
        cells
            .iter()
            .filter(|c| {
                let e = &c.entry;
                e.gcc == id
                    && e.platform == platform
                    && e.config == config
                    && e.kernel.parse::<Version>().ok().as_ref() == Some(kernel)
            })
            .max_by_key(|c| c.entry.started)
    };
    let name = short(&gcc.version);
    let mut out = format!(
        "# GCC {name}\n\nThe release report for {} (spec 10.8), over the {} kernels of the Current set with {config}. It is written by `gk report new-gcc {}` from the result store, and a crossing that has no cell yet is shown as not run.\n",
        gcc.id,
        kernels.len(),
        gcc.id
    );

    out.push_str("\n## What builds and runs\n");
    for p in &platforms {
        let found: Vec<_> = kernels
            .iter()
            .map(|k| (k, find(&k.version, &gcc.id, &p.name)))
            .collect();
        let ran = found.iter().filter(|(_, c)| c.is_some()).count();
        let works = found
            .iter()
            .filter(|(_, c)| c.is_some_and(|c| rank(&c.entry.verdict) >= 2))
            .count();
        let _ = write!(
            out,
            "\n### {}\n\nKernels with a cell: {ran} of {}. Of those, {works} run.\n\n| Kernel | Line | Verdict | Rung | Warnings | First error |\n|---|---|---|---|--:|---|\n",
            p.name,
            kernels.len()
        );
        for (k, c) in found {
            let line = if k.moniker.is_empty() {
                "-"
            } else {
                &k.moniker
            };
            match c {
                Some(c) => {
                    let e = &c.entry;
                    let _ = writeln!(
                        out,
                        "| {} | {line} | {} | {} | {} | {} |",
                        k.version,
                        e.verdict,
                        e.rung,
                        e.warnings.map_or("-".into(), |n| n.to_string()),
                        cell_text(&e.first_error)
                    );
                }
                None => {
                    let _ = writeln!(out, "| {} | {line} | not run | | | |", k.version);
                }
            }
        }
    }

    out.push_str("\n## Regressions against the same series\n\n");
    match series {
        None => {
            let _ = writeln!(
                out,
                "{} is the first release of its series in gccs.toml, so there is nothing to compare with.",
                gcc.id
            );
        }
        Some(s) => {
            let mut rows = String::new();
            for p in &platforms {
                for k in &kernels {
                    let (Some(old), Some(new)) = (
                        find(&k.version, &s.id, &p.name),
                        find(&k.version, &gcc.id, &p.name),
                    ) else {
                        continue;
                    };
                    if rank(&old.entry.verdict) > rank(&new.entry.verdict) {
                        let _ = writeln!(
                            rows,
                            "| {} | {} | {} | {} | {} |",
                            p.name,
                            k.version,
                            old.entry.verdict,
                            new.entry.verdict,
                            cell_text(&new.entry.first_error)
                        );
                    }
                }
            }
            if rows.is_empty() {
                let _ = writeln!(
                    out,
                    "No kernel that {} got to a verdict with does worse with {}.",
                    s.id, gcc.id
                );
            } else {
                let _ = write!(
                    out,
                    "Kernels that {} got further with. The signatures arrive with G3, so the first error is shown as the compiler wrote it.\n\n| Platform | Kernel | {} | {} | First error |\n|---|---|---|---|---|\n{rows}",
                    s.id, s.id, gcc.id
                );
            }
        }
    }

    out.push_str("\n## New warnings\n\n");
    match column {
        None => {
            let _ = writeln!(
                out,
                "{} is the oldest column, so no warning is new.",
                gcc.id
            );
        }
        Some(prev) => {
            // option -> (count before, count now, units now, first now)
            let mut by_option: BTreeMap<String, (usize, usize, usize, String)> = BTreeMap::new();
            let mut pairs = 0;
            for p in &platforms {
                for k in &kernels {
                    let (Some(old), Some(new)) = (
                        find(&k.version, &prev.id, &p.name),
                        find(&k.version, &gcc.id, &p.name),
                    ) else {
                        continue;
                    };
                    pairs += 1;
                    for w in &old.warnings {
                        by_option.entry(option(w)).or_default().0 += w.count;
                    }
                    for w in &new.warnings {
                        let o = by_option.entry(option(w)).or_default();
                        o.1 += w.count;
                        o.2 += 1;
                        if o.3.is_empty() {
                            o.3.clone_from(&w.first);
                        }
                    }
                }
            }
            let mut grew: Vec<_> = by_option.into_iter().filter(|(_, o)| o.1 > o.0).collect();
            grew.sort_by(|a, b| (b.1.1 - b.1.0).cmp(&(a.1.1 - a.1.0)).then(a.0.cmp(&b.0)));
            if pairs == 0 {
                let _ = writeln!(
                    out,
                    "No kernel has a cell with both {} and {} yet.",
                    prev.id, gcc.id
                );
            } else if grew.is_empty() {
                let _ = writeln!(
                    out,
                    "Over the {pairs} cells that have a {} cell to compare with, no warning option fires more often than it did.",
                    prev.id
                );
            } else {
                let _ = write!(
                    out,
                    "Warning options that fire more often than with {}, over the {pairs} cells that have one to compare with, counted by option (spec 11.5).\n\n| Option | {} | {} | Units | First |\n|---|--:|--:|--:|---|\n",
                    prev.id, prev.id, gcc.id
                );
                for (o, (before, now, units, first)) in grew {
                    let _ = writeln!(
                        out,
                        "| {o} | {before} | {now} | {units} | {} |",
                        cell_text(&first)
                    );
                }
            }
        }
    }

    out.push_str("\n## Configuration differences\n");
    match column {
        None => {
            let _ = writeln!(
                out,
                "\n{} is the oldest column, so there is nothing to compare with.",
                gcc.id
            );
        }
        Some(prev) => {
            let mut any = false;
            for p in &platforms {
                for k in &kernels {
                    let (Some(old), Some(new)) = (
                        find(&k.version, &prev.id, &p.name),
                        find(&k.version, &gcc.id, &p.name),
                    ) else {
                        continue;
                    };
                    let from = (prev.id.clone(), old.dir.clone());
                    let to = (gcc.id.clone(), new.dir.clone());
                    let body = differential::differences(&from, &to)
                        .unwrap_or_else(|e| format!("Not compared: {e}.\n"));
                    let _ = write!(out, "\n### {} on {}\n\n{body}", k.version, p.name);
                    any = true;
                }
            }
            if !any {
                let _ = writeln!(
                    out,
                    "\nNo kernel has a cell with both {} and {} yet.",
                    prev.id, gcc.id
                );
            }
        }
    }
    out
}

fn option(w: &Warnings) -> String {
    if w.option.is_empty() {
        "(none named)".into()
    } else {
        w.option.clone()
    }
}

/// Text that is safe inside a markdown table cell, cut to a readable length.
pub(crate) fn cell_text(s: &str) -> String {
    let s: String = s.replace('|', "\\|").replace('\n', " ");
    if s.chars().count() > 120 {
        s.chars().take(117).collect::<String>() + "..."
    } else {
        s
    }
}

/// `gk report new-gcc G [--config C] [--ungraded] [--stdout]`. Writes the report and returns its path, or returns the report itself with `--stdout`.
pub fn command(repo: &Repo, args: &[String]) -> Result<String, String> {
    let usage = "usage: gk report new-gcc G [--config C] [--ungraded] [--stdout]";
    let (mut words, mut config) = (Vec::new(), crate::cell::CONFIG.to_owned());
    let (mut ungraded, mut stdout) = (false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--config" => config.clone_from(it.next().ok_or("--config needs a value")?),
            "--ungraded" => ungraded = true,
            "--stdout" => stdout = true,
            other if !other.starts_with('-') => words.push(other.to_owned()),
            other => return Err(format!("gk report: unexpected {other:?}")),
        }
    }
    let ["new-gcc", g] = words.iter().map(String::as_str).collect::<Vec<_>>()[..] else {
        return Err(usage.into());
    };
    let gcc = repo
        .gccs
        .gccs
        .iter()
        .find(|x| x.id == g || x.id.strip_prefix("gcc-") == Some(g))
        .ok_or_else(|| format!("{g} is not in gccs.toml"))?;
    let text = render(repo, gcc, &config, &cells(ungraded)?);
    if stdout {
        return Ok(text);
    }
    let name = format!("reports/new-gcc-{}.md", short(&gcc.version));
    let dir = repo.root.join("reports");
    std::fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    std::fs::write(repo.root.join(&name), text).map_err(|e| format!("writing {name}: {e}"))?;
    Ok(name + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn cell(kernel: &str, gcc: &str, verdict: &str, warnings: &[(&str, usize)]) -> Cell {
        Cell {
            entry: Entry {
                cell: String::new(),
                kernel: kernel.into(),
                gcc: gcc.into(),
                binutils: String::new(),
                platform: "x86_64".into(),
                config: "defconfig+gk".into(),
                host: String::new(),
                rung: String::new(),
                verdict: verdict.into(),
                failing_units: 0,
                first_error: if verdict == "fails" {
                    "fs/x.c: error: oops".into()
                } else {
                    String::new()
                },
                warnings: Some(warnings.iter().map(|w| w.1).sum()),
                runs: 3,
                flaky: false,
                gk: String::new(),
                date: String::new(),
                started: 0,
                seconds: 0.0,
                build_seconds: None,
                machine: String::new(),
            },
            dir: PathBuf::from("/nonexistent"),
            warnings: warnings
                .iter()
                .map(|(o, n)| Warnings {
                    unit: "fs/x.c".into(),
                    option: (*o).into(),
                    count: *n,
                    fatal: 0,
                    werror: false,
                    first: format!("fs/x.c:1:1: warning: {o}"),
                })
                .collect(),
        }
    }

    #[test]
    fn the_report_has_its_four_parts() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let current = repo.kernels.in_set("current");
        let k = current.last().unwrap().version.to_string();
        let gcc = repo.gccs.get("gcc-16.2.0").unwrap();
        let cells = [
            cell(&k, "gcc-16.1.0", "works", &[("-Wunused", 2)]),
            cell(&k, "gcc-16.2.0", "fails", &[("-Wunused", 2), ("-Wnew", 5)]),
        ];
        let text = render(&repo, gcc, "defconfig+gk", &cells);
        assert!(text.starts_with("# GCC 16.2\n"));
        assert!(text.contains(&format!(
            "| x86_64 | {k} | works | fails | fs/x.c: error: oops |"
        )));
        assert!(text.contains("| -Wnew | 0 | 5 | 1 |"));
        assert!(!text.contains("| -Wunused |"));
        assert!(text.contains(&format!("### {k} on x86_64\n\nNot compared:")));
        assert!(text.contains("| not run |"));
    }

    #[test]
    fn report_names_drop_a_zero_patch_level() {
        assert_eq!(short(&"16.2.0".parse().unwrap()), "16.2");
        assert_eq!(short(&"4.9.4".parse().unwrap()), "4.9.4");
    }
}
