//! `reports/eras.md`, `reports/question-10.md` and `reports/holes.md`, which `gk publish` writes from the matrix (spec 11.2 and 08.5), and `matrix/frontiers.json` and `matrix/ranges.json`, the same rows read the other two ways (spec 02.6 and 10.2).
//!
//! Both read the newest cell at every crossing of a kernel, a GCC column, a platform and a configuration, so a rerun replaces the cell it reran. A row is every cell of one kernel on one platform and configuration, in GCC version order, and its working set is the columns whose cell works.

use crate::publish::{Entry, Matrix, letter};
use crate::search::era_column;
use gk_model::Version;
use gk_model::repo::Repo;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Platform, configuration and kernel.
type RowKey<'a> = (&'a str, &'a str, Version);

/// A GCC column by version, then its name, platform and configuration.
type ColumnKey<'a> = (Version, &'a str, &'a str, &'a str);

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
        "# Eras\n\nWhether the era GCC of each era works for every kernel in it (spec 11.2), for rucc-kernel's `personas.toml`. Each row is the newest cell of every GCC column run on one kernel, platform and configuration, and the working set runs from the oldest column that works to the newest. Where the era GCC does not work, the nearest working column is the fallback persona, and a person decides, because the persona also has to match what the era's distributions shipped. The kernel plan's open question 10, which 3.x and 4.x branches a later GCC can build, has its own report in [question-10.md](question-10.md). Written by `gk publish` from `matrix/matrix.json`.\n",
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

/// `matrix/frontiers.json`.
#[derive(Debug, Serialize)]
pub struct Frontiers {
    /// The schema version, the same as `matrix.json`'s.
    pub schema: u32,
    /// One per kernel, platform and configuration that has an upstream cell.
    pub rows: Vec<Frontier>,
}

/// The working set W(K, P) of one row over the upstream columns (spec 02.6).
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Frontier {
    /// The kernel.
    pub kernel: String,
    /// The platform.
    pub platform: String,
    /// The configuration.
    pub config: String,
    /// The era column e(K) on the platform, when the kernel has one.
    pub era: Option<String>,
    /// Whether the era column's cell works, or `None` when it has not run.
    pub era_works: Option<bool>,
    /// min(K, P), the oldest column that works, or `None` when none does.
    pub min: Option<String>,
    /// max(K, P), the newest column that works.
    pub max: Option<String>,
    /// The newest column older than min that ran and does not work, which is the other side of the older edge.
    pub below: Option<String>,
    /// The oldest column newer than max that ran and does not work.
    pub above: Option<String>,
    /// The columns strictly between min and max that ran and do not work.
    pub holes: Vec<String>,
    /// How many upstream columns have a cell in the row.
    pub columns: usize,
}

/// `matrix/ranges.json`.
#[derive(Debug, Serialize)]
pub struct Ranges {
    /// The schema version, the same as `matrix.json`'s.
    pub schema: u32,
    /// One per upstream column, platform and configuration that has a cell.
    pub columns: Vec<KernelRange>,
}

/// The kernel range of one GCC column on one platform and configuration (spec 02.6), the view rucc reads.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct KernelRange {
    /// The GCC column.
    pub gcc: String,
    /// The platform.
    pub platform: String,
    /// The configuration.
    pub config: String,
    /// The oldest kernel the column works on, or `None` when it works on none.
    pub oldest: Option<String>,
    /// The newest kernel it works on.
    pub newest: Option<String>,
    /// The kernels strictly between the two that ran and do not work.
    pub gaps: Vec<String>,
    /// How many kernels it works on.
    pub works: usize,
    /// How many kernels it ran on.
    pub ran: usize,
}

/// `reports/question-10.md`: the kernel plan's open question 10, which 3.x and 4.x stable branches a later GCC can build (spec 11.2). A branch is answered by the row of its last point, and a later GCC is the era GCC of any era after the branch's own.
#[must_use]
pub fn question_10(repo: &Repo, m: &Matrix) -> String {
    let rows = rows(repo, m);
    let last: Vec<&Version> = repo
        .kernels
        .in_set("last-points")
        .into_iter()
        .map(|k| &k.version)
        .filter(|v| matches!(v.series(1)[0], 3 | 4))
        .collect();
    let mut table = String::from(
        "| Branch | Last point | Platform | Configuration | Era | Era GCC | Newest working column | Later era GCCs that work | Later era GCCs that do not |\n|---|---|---|---|---|---|---|---|---|\n",
    );
    let (mut answered, mut later, mut own) = (BTreeMap::new(), 0, 0);
    for ((platform, config, kernel), cells) in &rows {
        if !last.contains(&kernel) {
            continue;
        }
        let Some(n) = repo.eras.eras.iter().rposition(|e| e.from <= *kernel) else {
            continue;
        };
        let era = &repo.eras.eras[n];
        let triple = repo
            .platforms
            .get(platform)
            .map(|p| p.triple.clone())
            .unwrap_or_default();
        let column = era_column(repo, &era.gcc, &triple).unwrap_or_default();
        let (mut good, mut bad): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
        for e in &repo.eras.eras[n + 1..] {
            let Some(c) = era_column(repo, &e.gcc, &triple) else {
                continue;
            };
            if c == column || good.contains(&c) || bad.contains(&c) {
                continue;
            }
            match cells.iter().find(|(_, x)| x.gcc == c) {
                Some((_, x)) if x.verdict == "works" => good.push(c),
                Some(_) => bad.push(c),
                None => {}
            }
        }
        if !good.is_empty() {
            later += 1;
        } else if !bad.is_empty() {
            own += 1;
        }
        let newest = range(cells).map_or_else(|| "none".to_owned(), |(_, b)| b.gcc.clone());
        let series = kernel.series(2);
        *answered.entry(series.clone()).or_insert(0) += 1;
        let _ = writeln!(
            table,
            "| {}.{} | {kernel} | {platform} | {config} | {} | {column} | {newest} | {} | {} |",
            series[0],
            series[1],
            era.name,
            good.join(", "),
            bad.join(", ")
        );
    }
    let mut out = String::from(
        "# Open question 10\n\nThe kernel plan's open question 10 asks which 3.x and 4.x stable branches a later GCC can build. This report answers it from the matrix (spec 11.2). Each branch is read at its last point, and a later GCC is the era GCC of any era after the branch's own, as the column that stands for it on the platform. A row counts the newest cell of every GCC column run on the last point, so the answer grows as the frontier searches of G2 reach more branches. Written by `gk publish` from `matrix/matrix.json`.\n\n",
    );
    let _ = writeln!(
        out,
        "Rows for a last point so far: {}, covering {} of the {} branches from 3.0 to 4.20. A later era's GCC builds the last point in {later} of them. In {own} every later era GCC that ran fails, so the branch stays with its own era's GCC or an older one. The rest have no later era GCC cell yet.\n",
        answered.values().sum::<usize>(),
        answered.len(),
        last.len()
    );
    if answered.is_empty() {
        out.push_str("No last point of a 3.x or 4.x branch has a row yet.\n");
    } else {
        out.push_str(&table);
    }
    out
}

/// The upstream cells of each row, which are the only ones W(K, P) counts.
fn upstream<'a>(repo: &Repo, m: &'a Matrix) -> BTreeMap<RowKey<'a>, Vec<(Version, &'a Entry)>> {
    let mut out = rows(repo, m);
    for cells in out.values_mut() {
        cells.retain(|(_, e)| {
            repo.gccs
                .get(&e.gcc)
                .is_some_and(|g| g.flavor == "upstream")
        });
    }
    out.retain(|_, cells| !cells.is_empty());
    out
}

/// The frontier of every row.
#[must_use]
pub fn frontiers(repo: &Repo, m: &Matrix) -> Frontiers {
    let full = rows(repo, m);
    let mut out = Vec::new();
    for (key, cells) in upstream(repo, m) {
        let (platform, config, kernel) = &key;
        let triple = repo
            .platforms
            .get(platform)
            .map(|p| p.triple.clone())
            .unwrap_or_default();
        let era = repo
            .eras
            .of(kernel)
            .and_then(|e| era_column(repo, &e.gcc, &triple));
        let era_works = era.as_ref().and_then(|c| {
            full.get(&key)?
                .iter()
                .find(|(_, e)| e.gcc == *c)
                .map(|(_, e)| e.verdict == "works")
        });
        let works = |e: &Entry| e.verdict == "works";
        let first = cells.iter().position(|(_, e)| works(e));
        let last = cells.iter().rposition(|(_, e)| works(e));
        let name = |i: usize| cells[i].1.gcc.clone();
        let (below, above, holes) = match (first, last) {
            (Some(a), Some(b)) => (
                a.checked_sub(1).map(name),
                (b + 1 < cells.len()).then(|| name(b + 1)),
                cells[a..=b]
                    .iter()
                    .filter(|(_, e)| !works(e))
                    .map(|(_, e)| e.gcc.clone())
                    .collect(),
            ),
            _ => (None, None, Vec::new()),
        };
        out.push(Frontier {
            kernel: kernel.to_string(),
            platform: (*platform).to_owned(),
            config: (*config).to_owned(),
            era,
            era_works,
            min: first.map(name),
            max: last.map(name),
            below,
            above,
            holes,
            columns: cells.len(),
        });
    }
    Frontiers {
        schema: crate::publish::SCHEMA,
        rows: out,
    }
}

/// The kernel range of every column.
#[must_use]
pub fn ranges(repo: &Repo, m: &Matrix) -> Ranges {
    let mut by: BTreeMap<ColumnKey<'_>, Vec<(&Version, bool)>> = BTreeMap::new();
    let rows = upstream(repo, m);
    for ((platform, config, kernel), cells) in &rows {
        for (v, e) in cells {
            by.entry((v.clone(), e.gcc.as_str(), *platform, *config))
                .or_default()
                .push((kernel, e.verdict == "works"));
        }
    }
    let columns = by
        .into_iter()
        .map(|((_, gcc, platform, config), kernels)| {
            let first = kernels.iter().position(|k| k.1);
            let last = kernels.iter().rposition(|k| k.1);
            let gaps = match (first, last) {
                (Some(a), Some(b)) => kernels[a..=b]
                    .iter()
                    .filter(|k| !k.1)
                    .map(|k| k.0.to_string())
                    .collect(),
                _ => Vec::new(),
            };
            KernelRange {
                gcc: gcc.to_owned(),
                platform: platform.to_owned(),
                config: config.to_owned(),
                oldest: first.map(|i| kernels[i].0.to_string()),
                newest: last.map(|i| kernels[i].0.to_string()),
                gaps,
                works: kernels.iter().filter(|k| k.1).count(),
                ran: kernels.len(),
            }
        })
        .collect();
    Ranges {
        schema: crate::publish::SCHEMA,
        columns,
    }
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

    #[test]
    fn a_frontier_has_both_edges_and_its_holes() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let m = Matrix {
            schema: 1,
            cells: vec![
                cell("7.2.8", "gcc-5.5.0", "fails"),
                cell("7.2.8", "gcc-8.5.0", "works"),
                cell("7.2.8", "gcc-12.5.0", "fails"),
                cell("7.2.8", "gcc-14.2.0", "works"),
                cell("7.2.8", "gcc-16.2.0", "builds"),
            ],
        };
        let f = frontiers(&repo, &m);
        assert_eq!(f.rows.len(), 1);
        let row = &f.rows[0];
        assert_eq!(row.min.as_deref(), Some("gcc-8.5.0"));
        assert_eq!(row.max.as_deref(), Some("gcc-14.2.0"));
        assert_eq!(row.below.as_deref(), Some("gcc-5.5.0"));
        assert_eq!(row.above.as_deref(), Some("gcc-16.2.0"));
        assert_eq!(row.holes, ["gcc-12.5.0"]);
        assert_eq!(row.columns, 5);
        let r = ranges(&repo, &m);
        let twelve = r.columns.iter().find(|c| c.gcc == "gcc-12.5.0").unwrap();
        assert_eq!(
            (twelve.oldest.as_ref(), twelve.works, twelve.ran),
            (None, 0, 1)
        );
    }

    #[test]
    fn a_last_point_built_by_a_later_era_gcc_answers_question_10() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let k: Version = "3.0.101".parse().unwrap();
        let n = repo.eras.eras.iter().rposition(|e| e.from <= k).unwrap();
        let own = era_column(&repo, &repo.eras.eras[n].gcc, "x86_64-linux-gnu").unwrap();
        let next = era_column(&repo, &repo.eras.eras[n + 1].gcc, "x86_64-linux-gnu").unwrap();
        let m = Matrix {
            schema: 1,
            cells: vec![
                cell("3.0.101", &own, "works"),
                cell("3.0.101", &next, "works"),
                cell("3.0.101", "gcc-16.2.0", "fails"),
                cell("3.1", &next, "works"),
            ],
        };
        let text = question_10(&repo, &m);
        assert!(text.contains("Rows for a last point so far: 1, covering 1 of the 41 branches from 3.0 to 4.20. A later era's GCC builds the last point in 1 of them. In 0 "));
        assert!(text.contains(&format!(
            "| 3.0 | 3.0.101 | x86_64 | defconfig+gk | {} | {own} | {next} | {next}",
            repo.eras.eras[n].name
        )));
        assert!(!text.contains("| 3.1 |"));
    }

    #[test]
    fn a_kernel_range_has_its_gaps() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let m = Matrix {
            schema: 1,
            cells: vec![
                cell("6.1.188", "gcc-14.2.0", "works"),
                cell("6.6.157", "gcc-14.2.0", "fails"),
                cell("7.2.8", "gcc-14.2.0", "works"),
            ],
        };
        let r = ranges(&repo, &m);
        assert_eq!(r.columns.len(), 1);
        let c = &r.columns[0];
        assert_eq!(c.oldest.as_deref(), Some("6.1.188"));
        assert_eq!(c.newest.as_deref(), Some("7.2.8"));
        assert_eq!(c.gaps, ["6.6.157"]);
        assert_eq!((c.works, c.ran), (2, 3));
    }
}
