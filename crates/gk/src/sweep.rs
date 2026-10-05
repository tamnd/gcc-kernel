//! `gk binutils-sweep` and `gk report binutils`: the binutils axis of spec 04.6, with GCC fixed at the kernel's era GCC.
//!
//! The columns are the last point of each binutils release series, oldest first, except that the series of the binutils the era GCC's bundle was built with is stood for by that release itself, so the sweep starts from the matrix cell. From there the frontier search of spec 09.3 gallops out to each edge and binary searches it, as `gk search` does over GCC. A binutils that has no bundle yet is forged on the way, in the forge of the newest GCC released before it, and one that cannot be built for the platform at all is a column that does not work.
//!
//! A cell of the sweep carries the binutils bundle's digest in its binutils coordinate, where a matrix cell carries the tarball's, which is how the matrix and the reports about the GCC axis leave the sweep's cells out.

use crate::forge;
use crate::publish::{self, Entry};
use crate::search::{self, Column, Options};
use crate::store;
use gk_model::Version;
use gk_model::cell::Coordinates;
use gk_model::repo::Repo;
use gk_model::toolchains::Binutils;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Whether a cell is one of the sweep's: its binutils is a release of `binutils.toml` whose digest is not the tarball's.
#[must_use]
pub fn swept(repo: &Repo, c: &Coordinates) -> bool {
    repo.binutils
        .get(&c.binutils.name)
        .is_some_and(|b| c.binutils.digest != format!("sha256:{}", b.sha256))
}

/// The sweep's columns, oldest first: the last point of each series, with the series of `paired` stood for by `paired`.
#[must_use]
pub fn columns<'a>(repo: &'a Repo, paired: &str) -> Vec<&'a Binutils> {
    let mut last: BTreeMap<Vec<u32>, &Binutils> = BTreeMap::new();
    for b in &repo.binutils.releases {
        let at = last.entry(b.version.series(2)).or_insert(b);
        if b.version > at.version {
            *at = b;
        }
    }
    if let Some(p) = repo
        .binutils
        .releases
        .iter()
        .find(|b| b.version.as_str() == paired)
    {
        last.insert(p.version.series(2), p);
    }
    last.into_values().collect()
}

/// Sweep the binutils of `kernel` on `platform`, printing each column as it finishes. The cells come back oldest binutils first.
pub fn run(
    repo: &Repo,
    kernel: &str,
    platform: &str,
    opts: Options,
) -> Result<Vec<(String, Column)>, String> {
    let version: Version = kernel
        .trim_start_matches("linux-")
        .parse()
        .map_err(|e| format!("kernel {kernel}: {e:?}"))?;
    let p = repo
        .platforms
        .get(platform)
        .ok_or_else(|| format!("platform {platform} is not in platforms.toml"))?;
    let gcc = repo
        .eras
        .of(&version)
        .and_then(|e| search::era_column(repo, &e.gcc, &p.triple))
        .ok_or_else(|| format!("{version} has no era column on {platform}"))?;
    let paired = forge::manifest(&gcc, &p.triple)?.binutils;
    let cols = columns(repo, &paired);
    let at = cols
        .iter()
        .position(|b| b.version.as_str() == paired)
        .ok_or_else(|| format!("binutils {paired} of {gcc} is not in binutils.toml"))?;
    println!("binutils sweep of {version} on {platform} with {gcc}, from binutils {paired}");
    let mut ran: BTreeMap<usize, Column> = BTreeMap::new();
    let mut works = |i: usize| {
        let b = cols[i];
        let c = if b.version.as_str() != paired
            && forge::manifest(&b.id, &p.triple).is_err()
            && let Err(e) = forge::run(repo, &b.id, std::slice::from_ref(&p.triple), None)
        {
            Column::Broken(format!("no bundle: {e}"))
        } else {
            search::one_with(repo, &version, &gcc, platform, opts, false, Some(&b.id))
        };
        search::print_column(&b.id, &c);
        let ok = search::works(&c);
        ran.insert(i, c);
        ok
    };
    if search::frontier(cols.len(), at, &mut works).is_none() {
        println!("the cell with the paired binutils does not work, so there is nothing to sweep");
    }
    let row: Vec<(String, Column)> = ran
        .into_iter()
        .map(|(i, c)| (cols[i].id.clone(), c))
        .collect();
    search::print_edges(&row);
    Ok(row)
}

/// A cell of the sweep, or the matrix cell it starts from, as the report needs it.
struct Run {
    binutils: Version,
    entry: Entry,
}

/// The rows the sweep has run, by platform, configuration, kernel and GCC, each with its runs oldest binutils first.
type Rows = BTreeMap<(String, String, Version, String), Vec<Run>>;

/// Every row of the store with more cells than the matrix one, where the GCC is the kernel's era GCC.
fn rows(repo: &Repo, ungraded: bool) -> Result<Rows, String> {
    let catalog = crate::classify::compile(repo);
    let mut rows: Rows = BTreeMap::new();
    for (dir, record) in store::cells()? {
        if !(ungraded || record.graded) || record.coordinates.kernel.digest.starts_with("git:") {
            continue;
        }
        let coords = &record.coordinates;
        let (Some(binutils), Ok(kernel)) = (
            repo.binutils.get(&coords.binutils.name),
            coords
                .kernel
                .name
                .trim_start_matches("linux-")
                .parse::<Version>(),
        ) else {
            continue;
        };
        let Some(platform) = repo.platforms.get(&coords.platform) else {
            continue;
        };
        let era = repo
            .eras
            .of(&kernel)
            .and_then(|e| search::era_column(repo, &e.gcc, &platform.triple));
        if era.as_deref() != Some(coords.gcc.name.as_str()) {
            continue;
        }
        let key = (
            coords.platform.clone(),
            coords.config.name.clone(),
            kernel,
            coords.gcc.name.clone(),
        );
        let runs = rows.entry(key).or_default();
        let entry = publish::entry(repo, &catalog, &dir, &record);
        // The newest run of each binutils stands.
        match runs.iter_mut().find(|run| run.binutils == binutils.version) {
            Some(run) if run.entry.started < entry.started => run.entry = entry,
            Some(_) => {}
            None => runs.push(Run {
                binutils: binutils.version.clone(),
                entry,
            }),
        }
    }
    rows.retain(|_, runs| runs.len() > 1);
    for runs in rows.values_mut() {
        runs.sort_by(|a, b| a.binutils.cmp(&b.binutils));
    }
    Ok(rows)
}

/// One edge of a row in words: the run just beyond the oldest or the newest that works, and why it does not.
fn edge(runs: &[Run], older: bool) -> String {
    let ok: Vec<&Run> = runs.iter().filter(|r| r.entry.verdict == "works").collect();
    let Some(bound) = (if older { ok.first() } else { ok.last() }) else {
        return String::new();
    };
    let beyond = if older {
        runs.iter().rev().find(|r| r.binutils < bound.binutils)
    } else {
        runs.iter().find(|r| r.binutils > bound.binutils)
    };
    let Some(r) = beyond else {
        return "none run".into();
    };
    let why = if r.entry.class.is_empty() {
        r.entry.first_error.clone()
    } else {
        r.entry.class.clone()
    };
    let why = why.replace('|', "\\|");
    format!(
        "{} {} at {}{}",
        r.binutils,
        r.entry.verdict,
        r.entry.rung,
        if why.is_empty() {
            String::new()
        } else {
            format!(": `{}`", why.chars().take(80).collect::<String>())
        }
    )
}

/// The report, `reports/binutils.md`: for each kernel the binutils range that works with its era GCC, the edges and why they fail, and a strip of every column.
pub fn report(repo: &Repo, ungraded: bool) -> Result<String, String> {
    let rows = rows(repo, ungraded)?;
    let all = columns(repo, "");
    let mut out = String::new();
    out.push_str("# The binutils sweep\n\n");
    out.push_str("Which binutils builds and boots each kernel when GCC is held at the kernel's era GCC (spec 04.6). Each row starts from the matrix cell, whose binutils is the one the era GCC's bundle was built with, and the frontier search runs out from there over the last point of each binutils series until it finds the oldest and the newest release that work. A release between two that work is taken to work too, as the GCC search takes it, and a column nobody ran is left blank.\n\n");
    out.push_str("Written by `gk report binutils` from the result store. `gk binutils-sweep K --platform P` runs a row.\n\n");
    if rows.is_empty() {
        out.push_str("No row has been swept yet.\n");
        return Ok(out);
    }
    let mut last_section = (String::new(), String::new());
    for ((platform, config, kernel, gcc), runs) in &rows {
        if last_section != (platform.clone(), config.clone()) {
            last_section = (platform.clone(), config.clone());
            let _ = write!(
                out,
                "## {platform}, {config}\n\n| kernel | GCC | works from | works to | older edge | newer edge |\n|---|---|---|---|---|---|\n"
            );
        }
        let ok: Vec<&Run> = runs.iter().filter(|r| r.entry.verdict == "works").collect();
        let (from, to) = match (ok.first(), ok.last()) {
            (Some(a), Some(b)) => (a.binutils.to_string(), b.binutils.to_string()),
            _ => ("none".into(), "none".into()),
        };
        let _ = writeln!(
            out,
            "| {kernel} | {gcc} | {from} | {to} | {} | {} |",
            edge(runs, true),
            edge(runs, false)
        );
    }
    out.push_str("\n## Every column\n\nOne character for each binutils series, oldest first: `#` works, `+` builds or runs but does not pass every rung, `x` fails, and a dot is not run.\n\n```\n");
    let series: Vec<String> = all
        .iter()
        .map(|b| b.version.series(2).iter().map(u32::to_string).collect::<Vec<_>>().join("."))
        .collect();
    let width = rows
        .keys()
        .map(|(p, _, k, _)| format!("{p} {k}").len())
        .max()
        .unwrap_or(0);
    let _ = writeln!(
        out,
        "{:width$}  {} to {}",
        "",
        series.first().cloned().unwrap_or_default(),
        series.last().cloned().unwrap_or_default()
    );
    for ((platform, config, kernel, _), runs) in &rows {
        let strip: String = all
            .iter()
            .map(|b| {
                let mine = runs
                    .iter()
                    .filter(|r| r.binutils.series(2) == b.version.series(2))
                    .max_by(|x, y| x.binutils.cmp(&y.binutils));
                match mine.map(|r| r.entry.verdict.as_str()) {
                    None => '.',
                    Some("works") => '#',
                    Some("builds" | "runs") => '+',
                    Some(_) => 'x',
                }
            })
            .collect();
        let _ = writeln!(out, "{:width$}  {strip}  {config}", format!("{platform} {kernel}"));
    }
    out.push_str("```\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_columns_are_one_per_series_with_the_paired_release() {
        let repo = Repo::find().unwrap();
        let cols = columns(&repo, "2.26");
        let series: Vec<Vec<u32>> = cols.iter().map(|b| b.version.series(2)).collect();
        let mut sorted = series.clone();
        sorted.dedup();
        assert_eq!(series, sorted);
        assert!(cols.iter().any(|b| b.version.as_str() == "2.26"));
        assert!(!cols.iter().any(|b| b.version.as_str() == "2.26.1"));
        assert!(cols.windows(2).all(|w| w[0].version < w[1].version));
    }
}
