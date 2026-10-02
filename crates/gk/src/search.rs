//! `gk search`: run the cells of one matrix row, a kernel on a platform across the GCC columns.
//!
//! Only the dense search exists so far, every upstream column that targets the platform in version order, except that the kernel's era GCC runs first because the others grade their KUnit runs against it. The frontier search of spec 09.3 arrives with G2. A cell already in the store with the same identity is not run again unless `--rerun` is given, so a search that dies halfway picks up where it stopped.

use crate::cell::{self, CellRecord, Setup};
use crate::store;
use gk_model::Version;
use gk_model::repo::Repo;
use gk_model::toolchains::Gcc;

/// How to search. Each switch is a command line flag, which is why there are so many bools.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct Options {
    /// Run every column, which is the only search there is so far.
    pub dense: bool,
    /// Parallel jobs per build.
    pub jobs: usize,
    /// Run cells again that are already in the store.
    pub rerun: bool,
    /// Keep the scratch directories.
    pub keep: bool,
    /// Boot the cells that can boot.
    pub boot: bool,
    /// The configuration, a name from [`cell::CONFIGS`].
    pub config: &'static str,
}

/// One column of the row.
#[derive(Debug)]
pub enum Column {
    /// The cell, run now or found in the store.
    Ran {
        /// Whether it came from the store.
        cached: bool,
        /// The record.
        record: Box<CellRecord>,
    },
    /// The platform does not exist for this kernel or GCC.
    NotApplicable(String),
    /// The cell could not be set up or run, which is a harness problem, not a verdict.
    Broken(String),
}

/// The GCC columns of a row: every upstream column with a bundle target for the platform, oldest first.
#[must_use]
pub fn columns<'a>(repo: &'a Repo, triple: &str) -> Vec<&'a Gcc> {
    let mut gccs: Vec<_> = repo
        .gccs
        .gccs
        .iter()
        .filter(|g| g.flavor == "upstream" && g.targets.iter().any(|t| t == triple))
        .collect();
    gccs.sort_by(|a, b| a.version.cmp(&b.version));
    gccs
}

/// Search the row of `kernel` on `platform`, printing each column as it finishes.
pub fn run(
    repo: &Repo,
    kernel: &str,
    platform: &str,
    opts: Options,
) -> Result<Vec<(String, Column)>, String> {
    if !opts.dense {
        return Err("only the dense search exists until G2; pass --dense".into());
    }
    let version: Version = kernel
        .trim_start_matches("linux-")
        .parse()
        .map_err(|e| format!("kernel {kernel}: {e:?}"))?;
    let p = repo
        .platforms
        .get(platform)
        .ok_or_else(|| format!("platform {platform} is not in platforms.toml"))?;
    let cols = columns(repo, &p.triple);
    if cols.is_empty() {
        return Err(format!("no GCC column targets {}", p.triple));
    }
    let mut row = Vec::new();
    for g in era_first(cols, repo.eras.of(&version).map(|e| e.gcc.as_str())) {
        let column = if p.applies(&version, &g.version) {
            one(repo, &version, &g.id, platform, opts)
        } else {
            Column::NotApplicable(format!(
                "{platform} starts at kernel {} and GCC {}",
                p.first_kernel, p.first_gcc
            ))
        };
        print_column(&g.id, &column);
        row.push((g.id.clone(), column));
    }
    let version = |id: &str| repo.gccs.get(id).map(|g| g.version.clone());
    row.sort_by_key(|(id, _)| version(id));
    Ok(row)
}

/// The columns with the era GCC's moved to the front. Every other cell of the row grades its KUnit run against the era cell (spec 02.4), so that cell has to be in the store before they reach L7.
fn era_first<'a>(mut cols: Vec<&'a Gcc>, era: Option<&str>) -> Vec<&'a Gcc> {
    if let Some(at) = cols.iter().position(|g| Some(g.id.as_str()) == era) {
        let g = cols.remove(at);
        cols.insert(0, g);
    }
    cols
}

fn one(repo: &Repo, version: &Version, gcc: &str, platform: &str, opts: Options) -> Column {
    let setup = match Setup::new(repo, version.as_str(), gcc, platform, opts.config) {
        Ok(s) if opts.boot => match s.booting(repo) {
            Ok(s) => s,
            Err(e) => return Column::Broken(e),
        },
        Ok(s) => s,
        Err(e) => return Column::Broken(e),
    };
    let identity = setup.coordinates.identity();
    if !opts.rerun
        && let Ok(text) = std::fs::read_to_string(store::cell_dir(&identity).join("cell.json"))
        && let Ok(record) = serde_json::from_str::<CellRecord>(&text)
        && record.graded
    {
        return Column::Ran {
            cached: true,
            record: Box::new(record),
        };
    }
    match cell::run(repo, &setup, opts.jobs, opts.keep) {
        Ok((_, record)) => Column::Ran {
            cached: false,
            record: Box::new(record),
        },
        Err(e) => Column::Broken(e),
    }
}

fn print_column(gcc: &str, column: &Column) {
    match column {
        Column::Ran { cached, record } => println!(
            "{gcc:<12} {:<3} {:<7} {:>6.0}s{}",
            record.rung,
            record.verdict,
            record.seconds,
            if *cached { "  (in the store)" } else { "" }
        ),
        Column::NotApplicable(why) => println!("{gcc:<12} n/a: {why}"),
        Column::Broken(why) => println!("{gcc:<12} not run: {why}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn the_x86_64_row_has_every_g0_column_in_order() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let ids: Vec<&str> = columns(&repo, "x86_64-linux-gnu")
            .iter()
            .map(|g| g.id.as_str())
            .collect();
        assert_eq!(ids.first(), Some(&"gcc-8.5.0"));
        assert_eq!(ids.last(), Some(&"gcc-16.2.0"));
        assert_eq!(ids.len(), 12);
    }

    #[test]
    fn the_era_column_runs_first() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let cols = columns(&repo, "x86_64-linux-gnu");
        let order: Vec<&str> = era_first(cols.clone(), Some("gcc-14.2.0"))
            .iter()
            .map(|g| g.id.as_str())
            .collect();
        assert_eq!(order[0], "gcc-14.2.0");
        assert_eq!(order[1], "gcc-8.5.0");
        assert_eq!(order.len(), cols.len());
        assert_eq!(era_first(cols.clone(), None).len(), cols.len());
    }
}
