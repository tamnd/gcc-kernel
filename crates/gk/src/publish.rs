//! `gk publish`: `matrix/matrix.json` from the result store (spec 10.5 and 10.7).
//!
//! Each record is a cell's `cell.json` with the heavy fields removed, plus the first error and the count of failing units from `errors.jsonl`. Ungraded cells are left out, because they ran from a checkout with uncommitted changes.
//!
//! The status section of `README.md` is rewritten too, between its markers (see [`crate::status`]).
//!
//! The heat maps go to `reports/matrix-<platform>.md`: a table per configuration, a row per kernel that has a cell, a column per GCC that targets the platform, and one colored square per cell. The per-GCC and per-kernel pages wait for `gk explain`.

use crate::cell::CellRecord;
use crate::store;
use gk_model::Version;
use gk_model::repo::Repo;
use gk_model::toolchains::add_days;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::path::Path;

/// The version of the `matrix.json` schema. It changes only when a field changes meaning or goes away.
pub const SCHEMA: u32 = 1;

/// `matrix.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matrix {
    /// The schema version.
    pub schema: u32,
    /// Every graded cell, by platform, kernel and GCC.
    pub cells: Vec<Entry>,
}

/// One cell of the matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Entry {
    /// The identity.
    pub cell: String,
    /// The kernel version, as `7.2.8`.
    pub kernel: String,
    /// The GCC column.
    pub gcc: String,
    /// The binutils.
    pub binutils: String,
    /// The platform.
    pub platform: String,
    /// The configuration.
    pub config: String,
    /// The host environment.
    pub host: String,
    /// The highest rung passed.
    pub rung: String,
    /// The verdict.
    pub verdict: String,
    /// How many units failed to compile.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub failing_units: usize,
    /// The first unit that failed and its first error.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub first_error: String,
    /// How many warnings its units got, from `warnings.jsonl`, or `None` for a cell run before the census.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warnings: Option<usize>,
    /// How many times the cell booted, or 1 for a cell that never got to boot.
    pub runs: u32,
    /// Whether its boots disagreed.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub flaky: bool,
    /// The gk version and commit that ran it.
    pub gk: String,
    /// The day it ran, as `YYYY-MM-DD`.
    pub date: String,
    /// When it started, in seconds since the epoch, which tells two runs of the same coordinates apart.
    #[serde(default)]
    pub started: u64,
    /// Wall seconds for the whole cell.
    pub seconds: f64,
    /// Wall seconds of the build step alone, L3, which is the number document 09 estimates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_seconds: Option<f64>,
    /// The machine.
    pub machine: String,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// The first error and the count of failing units, from a cell's `errors.jsonl`.
fn errors(dir: &Path) -> (usize, String) {
    let text = std::fs::read_to_string(dir.join("errors.jsonl")).unwrap_or_default();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let first = lines
        .first()
        .and_then(|l| serde_json::from_str::<crate::build::FailedUnit>(l).ok())
        .map(|u| format!("{}: {}", u.unit, u.error))
        .unwrap_or_default();
    (lines.len(), first)
}

/// The number of warnings in a cell's `warnings.jsonl`, or `None` when the cell has none.
fn warnings(dir: &Path) -> Option<usize> {
    let text = std::fs::read_to_string(dir.join("warnings.jsonl")).ok()?;
    Some(
        text.lines()
            .filter_map(|l| serde_json::from_str::<crate::build::Warnings>(l).ok())
            .map(|w| w.count)
            .sum(),
    )
}

/// The matrix record of one cell.
pub fn entry(dir: &Path, r: &CellRecord) -> Entry {
    let c = &r.coordinates;
    let (failing_units, first_error) = errors(dir);
    let days = i64::try_from(r.started / 86_400).unwrap_or(0);
    Entry {
        cell: r.cell.clone(),
        kernel: c.kernel.name.trim_start_matches("linux-").to_owned(),
        gcc: c.gcc.name.clone(),
        binutils: c.binutils.name.clone(),
        platform: c.platform.clone(),
        config: c.config.name.clone(),
        host: c.host.name.clone(),
        rung: r.rung.clone(),
        verdict: r.verdict.clone(),
        failing_units,
        first_error,
        warnings: warnings(dir),
        runs: u32::try_from(r.boots.len().max(1)).unwrap_or(u32::MAX),
        flaky: r.flaky,
        gk: r.gk.clone(),
        date: add_days("1970-01-01", days).unwrap_or_default(),
        started: r.started,
        seconds: r.seconds,
        build_seconds: r.steps.iter().find(|s| s.rung == "L3").map(|s| s.seconds),
        machine: r.machine.clone(),
    }
}

/// The matrix of every graded cell in the store, or every cell with `ungraded`.
pub fn matrix(ungraded: bool) -> Result<Matrix, String> {
    let mut cells: Vec<Entry> = store::cells()?
        .iter()
        .filter(|(_, r)| ungraded || r.graded)
        .map(|(dir, r)| entry(dir, r))
        .collect();
    let gcc_version = |id: &str| {
        id.rsplit_once("gcc-")
            .and_then(|(_, v)| v.parse::<Version>().ok())
    };
    cells.sort_by(|a, b| {
        (
            &a.platform,
            a.kernel.parse::<Version>().ok(),
            gcc_version(&a.gcc),
        )
            .cmp(&(
                &b.platform,
                b.kernel.parse::<Version>().ok(),
                gcc_version(&b.gcc),
            ))
    });
    Ok(Matrix {
        schema: SCHEMA,
        cells,
    })
}

/// The heat map square of a verdict: green works, yellow runs, orange builds, red fails, or `·` for n/a.
pub(crate) fn letter(verdict: &str) -> &'static str {
    match verdict {
        "works" => "🟩",
        "runs" => "🟨",
        "builds" => "🟧",
        "fails" => "🟥",
        _ => "·",
    }
}

/// The square of a cell in a heat map, with ⚠️ after it when its boots disagreed.
pub(crate) fn square(e: &Entry) -> String {
    format!("{}{}", letter(&e.verdict), if e.flaky { "⚠️" } else { "" })
}

/// The legend under every heat map.
pub(crate) const LEGEND: &str = "🟩 works, 🟨 runs, 🟧 builds, 🟥 fails, · n/a, and blank where the cell has not run yet. ⚠️ marks a flaky cell, whose boots disagreed and which keeps the lowest rung.";

/// The heat map of one platform, or `None` when it has no cells.
#[must_use]
pub fn heat_map(repo: &Repo, m: &Matrix, platform: &str) -> Option<String> {
    let p = repo.platforms.get(platform)?;
    let cells: Vec<&Entry> = m.cells.iter().filter(|e| e.platform == platform).collect();
    if cells.is_empty() {
        return None;
    }
    let mut gccs: Vec<(&str, Version)> = repo
        .gccs
        .gccs
        .iter()
        .filter(|g| g.targets.contains(&p.triple))
        .map(|g| (g.id.as_str(), g.version.clone()))
        .collect();
    gccs.sort_by(|a, b| a.1.cmp(&b.1));
    let mut configs: Vec<&str> = cells.iter().map(|e| e.config.as_str()).collect();
    configs.sort_unstable();
    configs.dedup();

    let mut out = format!(
        "# {platform}\n\nOne square per cell: {LEGEND} Written by `gk publish` from `matrix/matrix.json`.\n"
    );
    for config in configs {
        let mut kernels: Vec<Version> = cells
            .iter()
            .filter(|e| e.config == config)
            .filter_map(|e| e.kernel.parse().ok())
            .collect();
        kernels.sort();
        kernels.dedup();
        let _ = write!(out, "\n## {config}\n\n");
        table(&mut out, &cells, config, &kernels, &gccs, square);
        if cells
            .iter()
            .any(|e| e.config == config && e.warnings.is_some_and(|n| n > 0))
        {
            let _ = write!(
                out,
                "\nWarnings from the compiler on {config}, counted over every unit (spec 11.5), with `warnings.jsonl` in each cell giving them by unit and option.\n\n"
            );
            table(&mut out, &cells, config, &kernels, &gccs, |e| {
                e.warnings.map(|n| n.to_string()).unwrap_or_default()
            });
        }
    }
    Some(out)
}

/// One table of a heat map: a row per kernel, a column per GCC, and `square` for the newest cell at each crossing.
pub(crate) fn table(
    out: &mut String,
    cells: &[&Entry],
    config: &str,
    kernels: &[Version],
    gccs: &[(&str, Version)],
    square: impl Fn(&Entry) -> String,
) {
    out.push_str("| Kernel |");
    for (_, v) in gccs {
        let _ = write!(out, " {v} |");
    }
    out.push_str("\n|---|");
    out.push_str(&":-:|".repeat(gccs.len()));
    out.push('\n');
    for k in kernels {
        let _ = write!(out, "| {k} |");
        for (id, _) in gccs {
            // A rerun with a new rig or toolchain is a new cell with the same names, and the newest one is the square.
            let cell = cells
                .iter()
                .filter(|e| {
                    e.config == config
                        && e.gcc == *id
                        && e.kernel.parse::<Version>().ok().as_ref() == Some(k)
                })
                .max_by_key(|e| e.started);
            let _ = write!(out, " {} |", cell.map(|e| square(e)).unwrap_or_default());
        }
        out.push('\n');
    }
}

/// Write `matrix/matrix.json` and the heat maps under the repository. Returns how many cells the matrix holds and which reports were written.
pub fn write(repo: &Repo, ungraded: bool) -> Result<(usize, Vec<String>), String> {
    let m = matrix(ungraded)?;
    let dir = repo.root.join("matrix");
    std::fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let text = serde_json::to_string_pretty(&m).map_err(|e| e.to_string())? + "\n";
    std::fs::write(dir.join("matrix.json"), text)
        .map_err(|e| format!("writing matrix/matrix.json: {e}"))?;
    let reports = repo.root.join("reports");
    let mut written = Vec::new();
    for p in &repo.platforms.platforms {
        let Some(text) = heat_map(repo, &m, &p.name) else {
            continue;
        };
        std::fs::create_dir_all(&reports)
            .map_err(|e| format!("creating {}: {e}", reports.display()))?;
        let name = format!("reports/matrix-{}.md", p.name);
        std::fs::write(repo.root.join(&name), text).map_err(|e| format!("writing {name}: {e}"))?;
        written.push(name);
    }
    let readme = repo.root.join("README.md");
    if let Ok(text) = std::fs::read_to_string(&readme) {
        let days = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() / 86_400);
        let date = add_days("1970-01-01", i64::try_from(days).unwrap_or(0)).unwrap_or_default();
        if let Some(new) = crate::status::splice(&text, &crate::status::section(repo, &m, &date)) {
            std::fs::write(&readme, new).map_err(|e| format!("writing README.md: {e}"))?;
            written.push("README.md".into());
        }
    }
    Ok((m.cells.len(), written))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(kernel: &str, gcc: &str, verdict: &str, flaky: bool) -> Entry {
        Entry {
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
            first_error: String::new(),
            warnings: None,
            runs: 3,
            flaky,
            gk: String::new(),
            date: String::new(),
            started: 0,
            seconds: 0.0,
            build_seconds: None,
            machine: String::new(),
        }
    }

    #[test]
    fn a_heat_map_has_a_row_per_kernel_and_a_square_per_cell() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let mut newer = cell("7.2.8", "gcc-16.2.0", "runs", false);
        newer.started = 5;
        newer.warnings = Some(12);
        let m = Matrix {
            schema: SCHEMA,
            cells: vec![
                cell("7.2.8", "gcc-16.2.0", "builds", false),
                newer,
                cell("7.2.8", "gcc-8.5.0", "fails", false),
                cell("6.18", "gcc-16.2.0", "works", true),
            ],
        };
        let text = heat_map(&repo, &m, "x86_64").unwrap();
        let rows: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("| 6") || l.starts_with("| 7"))
            .collect();
        assert_eq!(rows.len(), 4);
        assert!(rows[0].starts_with("| 6.18 |"));
        assert!(rows[0].ends_with(" 🟩⚠️ |"));
        assert!(rows[1].starts_with("| 7.2.8 | 🟥 |"));
        assert!(rows[1].ends_with(" 🟨 |"));
        assert!(rows[3].starts_with("| 7.2.8 |  |"));
        assert!(rows[3].ends_with(" 12 |"));
        assert!(heat_map(&repo, &m, "arm64").is_none());
    }
}
