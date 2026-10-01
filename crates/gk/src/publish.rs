//! `gk publish`: `matrix/matrix.json` from the result store (spec 10.5 and 10.7).
//!
//! Each record is a cell's `cell.json` with the heavy fields removed, plus the first error and the count of failing units from `errors.jsonl`. Ungraded cells are left out, because they ran from a checkout with uncommitted changes. The heat maps and the per-GCC and per-kernel pages arrive with G1.

use crate::cell::CellRecord;
use crate::store;
use gk_model::Version;
use gk_model::toolchains::add_days;
use serde::{Deserialize, Serialize};
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
    /// How many times the cell ran. Always 1 until the three-run rule of G1.
    pub runs: u32,
    /// The gk version and commit that ran it.
    pub gk: String,
    /// The day it ran, as `YYYY-MM-DD`.
    pub date: String,
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

fn entry(dir: &Path, r: &CellRecord) -> Entry {
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
        runs: 1,
        gk: r.gk.clone(),
        date: add_days("1970-01-01", days).unwrap_or_default(),
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

/// Write `matrix/matrix.json` under `root`. Returns how many cells it holds.
pub fn write(root: &Path, ungraded: bool) -> Result<usize, String> {
    let m = matrix(ungraded)?;
    let dir = root.join("matrix");
    std::fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let text = serde_json::to_string_pretty(&m).map_err(|e| e.to_string())? + "\n";
    std::fs::write(dir.join("matrix.json"), text)
        .map_err(|e| format!("writing matrix/matrix.json: {e}"))?;
    Ok(m.cells.len())
}
