//! `reports/eras.md` and `reports/holes.md`, which `gk publish` writes from the matrix (spec 11.2 and 08.5).
//!
//! Both read the newest cell at every crossing of a kernel, a GCC column, a platform and a configuration, so a rerun replaces the cell it reran. A row is every cell of one kernel on one platform and configuration, in GCC version order, and its working set is the columns whose cell works.

use crate::publish::{Entry, Matrix, letter};
use crate::search::era_column;
use gk_model::Version;
use gk_model::repo::Repo;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Platform, configuration and kernel.
type RowKey<'a> = (&'a str, &'a str, Version);

/// The rows of the matrix: the newest cell per GCC column, in GCC version order.
fn rows<'a>(repo: &Repo, m: &'a Matrix) -> BTreeMap<RowKey<'a>, Vec<(Version, &'a Entry)>> {
    let mut newest: BTreeMap<(RowKey<'a>, &'a str), &'a Entry> = BTreeMap::new();
    for e in &m.cells {
        let Ok(k) = e.kernel.parse::<Version>() else {
            continue;
        };
        let slot = newest
            .entry(((e.platform.as_str(), e.config.as_str(), k), e.gcc.as_str()))
            .or_insert(e);
        if e.started > slot.started {
            *slot = e;
        }
    }
    let mut out: BTreeMap<RowKey<'a>, Vec<(Version, &'a Entry)>> = BTreeMap::new();
    for ((row, gcc), e) in newest {
        let Some(g) = repo.gccs.get(gcc) else {
            continue;
        };
        out.entry(row).or_default().push((g.version.clone(), e));
    }
    for cells in out.values_mut() {
        cells.sort_by(|a, b| a.0.cmp(&b.0));
    }
    out
}

/// The oldest and newest column of a row that work.
fn range<'a>(cells: &[(Version, &'a Entry)]) -> Option<(&'a Entry, &'a Entry)> {
    let mut works = cells.iter().filter(|(_, e)| e.verdict == "works");
    let first = works.next()?.1;
    Some((first, works.next_back().map_or(first, |c| c.1)))
}

/// The working column nearest to `want` by version, the fallback persona of spec 11.2.
fn nearest<'a>(cells: &[(Version, &'a Entry)], want: &Version) -> Option<&'a Entry> {
    let distance = |v: &Version| {
        let (a, b) = (v.series(1)[0], want.series(1)[0]);
        (a.abs_diff(b), v.clone())
    };
    cells
        .iter()
        .filter(|(_, e)| e.verdict == "works")
        .min_by_key(|(v, _)| distance(v))
        .map(|c| c.1)
}

/// `reports/eras.md`: whether each era GCC works for every kernel of its era that has a row.
#[must_use]
pub fn eras(repo: &Repo, m: &Matrix) -> String {
    let rows = rows(repo, m);
    let mut out = String::from(
        "# Eras\n\nWhether the era GCC of each era works for every kernel in it (spec 11.2), for rucc-kernel's `personas.toml`. Each row is the newest cell of every GCC column run on one kernel, platform and configuration, and the working set runs from the oldest column that works to the newest. Where the era GCC does not work, the nearest working column is the fallback persona, and a person decides, because the persona also has to match what the era's distributions shipped. Written by `gk publish` from `matrix/matrix.json`.\n",
    );
    let mut proposals = Vec::new();
    let mut summary = String::from(
        "\n| Era | Plan eras | From | Era GCC | Rows | Era GCC works | Era GCC does not | Era GCC not run |\n|---|---|---|---|--:|--:|--:|--:|\n",
    );
    let mut sections = String::new();
    for (n, era) in repo.eras.eras.iter().enumerate() {
        let until = repo.eras.eras.get(n + 1).map(|e| &e.from);
        let inside = |k: &Version| *k >= era.from && until.is_none_or(|u| k < u);
        let mine: Vec<_> = rows.iter().filter(|((_, _, k), _)| inside(k)).collect();
        if mine.is_empty() {
            continue;
        }
        let (mut good, mut bad, mut missing) = (0, 0, 0);
        let _ = write!(
            sections,
            "\n## {}, from {}, era GCC {}\n\n| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |\n|---|---|---|---|:-:|---|---|\n",
            era.name, era.from, era.gcc
        );
        for ((platform, config, kernel), cells) in &mine {
            let triple = repo
                .platforms
                .get(platform)
                .map(|p| p.triple.clone())
                .unwrap_or_default();
            let column = era_column(repo, &era.gcc, &triple).unwrap_or_default();
            let cell = cells.iter().find(|(_, e)| e.gcc == column).map(|c| c.1);
            let set = range(cells).map_or_else(
                || "empty".to_owned(),
                |(a, b)| {
                    if a.gcc == b.gcc {
                        a.gcc.clone()
                    } else {
                        format!("{} to {}", a.gcc, b.gcc)
                    }
                },
            );
            let fallback = match cell {
                Some(e) if e.verdict == "works" => {
                    good += 1;
                    String::new()
                }
                Some(e) => {
                    bad += 1;
                    let near = repo
                        .gccs
                        .get(&column)
                        .and_then(|g| nearest(cells, &g.version))
                        .map(|e| e.gcc.clone());
                    proposals.push(format!(
                        "- {} on {platform} ({config}): {column} {} at {}{}.",
                        kernel,
                        e.verdict,
                        e.rung,
                        near.as_ref().map_or_else(
                            || ", and no column works".to_owned(),
                            |g| format!(", and the nearest working column is {g}")
                        )
                    ));
                    near.unwrap_or_default()
                }
                None => {
                    missing += 1;
                    String::new()
                }
            };
            let _ = writeln!(
                sections,
                "| {kernel} | {platform} | {config} | {column} | {} | {set} | {fallback} |",
                cell.map_or("", |e| letter(&e.verdict))
            );
        }
        let _ = writeln!(
            summary,
            "| {} | {} | {} | {} | {} | {good} | {bad} | {missing} |",
            era.name,
            era.plan_eras.join(", "),
            era.from,
            era.gcc,
            mine.len()
        );
    }
    out.push_str(&summary);
    out.push_str("\n## Proposed changes to personas.toml\n\n");
    if proposals.is_empty() {
        out.push_str(
            "None. Every era GCC that has run works on every kernel of its era it ran on.\n",
        );
    } else {
        out.push_str("Each line is a row where the era GCC does not work. One such row is evidence that the era should split or change its persona (spec 03.3), and the cell says why.\n\n");
        for p in proposals {
            out.push_str(&p);
            out.push('\n');
        }
    }
    out.push_str(&sections);
    out
}

/// `reports/holes.md`: every cell inside a row's working set that does not work (spec 08.5).
#[must_use]
pub fn holes(repo: &Repo, m: &Matrix) -> String {
    let mut out = String::from(
        "# Holes\n\nA hole is a cell that does not work between two columns of the same row that do (spec 08.5). It is where one GCC release broke a kernel that the releases around it built and ran, or where the rig was noisy, which is what the flaky mark and the three runs are for. Written by `gk publish` from `matrix/matrix.json`.\n\n",
    );
    let mut list = String::new();
    let mut count = 0;
    for ((platform, config, kernel), cells) in &rows(repo, m) {
        let Some((oldest, newest)) = range(cells) else {
            continue;
        };
        let between = cells
            .iter()
            .skip_while(|(_, e)| e.gcc != oldest.gcc)
            .take_while(|(_, e)| e.gcc != newest.gcc);
        for (_, e) in between.filter(|(_, e)| e.verdict != "works") {
            count += 1;
            let class = if e.class.is_empty() {
                "unclassified"
            } else {
                &e.class
            };
            let mut why = e.first_error.replace('|', "\\|");
            if why.is_empty() && e.flaky {
                why = "the boots disagreed".into();
            }
            let _ = writeln!(
                list,
                "| {kernel} | {platform} | {config} | {} | {}{} | {} | {class} | {why} |",
                e.gcc,
                letter(&e.verdict),
                if e.flaky { "⚠️" } else { "" },
                e.rung
            );
        }
    }
    if count == 0 {
        out.push_str("No holes yet.\n");
    } else {
        let _ = write!(
            out,
            "{count} holes.\n\n| Kernel | Platform | Configuration | GCC | Cell | Rung | Class | First error |\n|---|---|---|---|:-:|---|---|---|\n{list}"
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn cell(kernel: &str, gcc: &str, verdict: &str) -> Entry {
        serde_json::from_value(serde_json::json!({
            "cell": "", "kernel": kernel, "gcc": gcc, "binutils": "", "platform": "x86_64",
            "config": "defconfig+gk", "host": "", "rung": "L3", "verdict": verdict, "runs": 1,
            "gk": "", "date": "", "seconds": 0.0, "machine": ""
        }))
        .unwrap()
    }

    #[test]
    fn a_failing_column_between_two_working_ones_is_a_hole() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let m = Matrix {
            schema: 1,
            cells: vec![
                cell("7.2.8", "gcc-8.5.0", "works"),
                cell("7.2.8", "gcc-12.5.0", "fails"),
                cell("7.2.8", "gcc-14.2.0", "works"),
                cell("7.2.8", "gcc-16.2.0", "fails"),
            ],
        };
        let text = holes(&repo, &m);
        assert!(text.contains("1 holes."));
        assert!(text.contains("| 7.2.8 | x86_64 | defconfig+gk | gcc-12.5.0 |"));
        assert!(!text.contains("gcc-16.2.0"));
    }

    #[test]
    fn an_era_gcc_that_fails_is_proposed_with_the_nearest_working_column() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let era = repo.eras.of(&"7.2.8".parse().unwrap()).unwrap().clone();
        let column = era_column(&repo, &era.gcc, "x86_64-linux-gnu").unwrap();
        let m = Matrix {
            schema: 1,
            cells: vec![
                cell("7.2.8", &column, "fails"),
                cell("7.2.8", "gcc-16.2.0", "works"),
            ],
        };
        let text = eras(&repo, &m);
        assert!(text.contains(&format!(
            "{column} fails at L3, and the nearest working column is gcc-16.2.0"
        )));
    }
}
