//! The status section of `README.md`, written by `gk publish` from `matrix/matrix.json`.
//!
//! The section sits between two marker comments and is replaced whole each time, so the rest of the README stays hand written. It has progress bars for the stripes G1 runs, a chart of verdicts and one of build times per GCC column, the newest cells, and a heat map per platform and configuration with a row for every Current kernel, so the empty squares show what is still to run.

use crate::publish::{Entry, LEGEND, Matrix, square, table};
use gk_model::Version;
use gk_model::repo::Repo;
use std::fmt::Write as _;

/// The line before the section.
pub const BEGIN: &str = "<!-- gk:status:begin -->";
/// The line after it.
pub const END: &str = "<!-- gk:status:end -->";

/// The configurations of the current stripe (spec 09.6).
const STRIPE_CONFIGS: &[&str] = &["defconfig+gk", "tinyconfig+gk"];

/// One crossing to run: kernel, GCC, platform and configuration.
type Crossing = (Version, String, String, &'static str);

/// Crossings grouped under a label, one progress line each.
type Rows = Vec<(String, Vec<Crossing>)>;

/// The blocks for one to seven eighths of a character.
const EIGHTHS: [char; 7] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉'];

/// `eighths` eighths of a character drawn in blocks, padded with `pad` to `width` characters.
fn blocks(eighths: usize, width: usize, pad: char) -> String {
    let eighths = eighths.min(width * 8);
    let mut out = "█".repeat(eighths / 8);
    if !eighths.is_multiple_of(8) {
        out.push(EIGHTHS[eighths % 8 - 1]);
    }
    let used = eighths.div_ceil(8);
    out.extend(std::iter::repeat_n(pad, width - used));
    out
}

/// A progress bar `width` characters long, solid for the done part and shaded for the rest. Anything done shows at least an eighth.
fn bar(done: usize, total: usize, width: usize) -> String {
    let eighths = if total == 0 {
        0
    } else {
        (done * width * 8).div_ceil(total)
    };
    blocks(eighths, width, '░')
}

/// `1 cell`, `2 cells`.
fn count(n: usize, what: &str) -> String {
    if n == 1 {
        format!("1 {what}")
    } else {
        format!("{n} {what}s")
    }
}

fn percent(done: usize, total: usize) -> usize {
    (done * 100).checked_div(total).unwrap_or(0)
}

/// Whether a crossing has a cell. A Current kernel that kernel.org has moved to a new point is counted by the cells of the point before until the new one has run, as in the census, so a new pin does not empty the progress bars.
fn ran(cells: &[Entry], kernel: &Version, gcc: &str, platform: &str, config: &str) -> bool {
    let line = crate::census::line(kernel);
    cells.iter().any(|e| {
        e.gcc == gcc
            && e.platform == platform
            && e.config == config
            && e
                .kernel
                .parse::<Version>()
                .is_ok_and(|k| k <= *kernel && crate::census::line(&k) == line)
    })
}

/// The tier 1 platforms, in the order of `platforms.toml`.
fn tier1(repo: &Repo) -> Vec<&gk_model::platforms::Platform> {
    repo.platforms
        .platforms
        .iter()
        .filter(|p| p.tier == 1)
        .collect()
}

/// Progress over a set of crossings: one line per (platform, config) and a total.
fn progress(out: &mut String, m: &Matrix, rows: &[(String, Vec<Crossing>)]) {
    let (mut done, mut total) = (0, 0);
    let mut lines = String::new();
    for (label, crossings) in rows {
        let d = crossings
            .iter()
            .filter(|(k, g, p, c)| ran(&m.cells, k, g, p, c))
            .count();
        let t = crossings.len();
        done += d;
        total += t;
        let _ = writeln!(
            lines,
            "{label:<24} {} {d:>4}/{t:<4} {:>3}%",
            bar(d, t, 30),
            percent(d, t)
        );
    }
    out.push_str("```text\n");
    out.push_str(&lines);
    let _ = writeln!(
        out,
        "{:<24} {} {done:>4}/{total:<4} {:>3}%",
        "total",
        bar(done, total, 30),
        percent(done, total)
    );
    out.push_str("```\n");
}

/// The crossings of the current stripe, grouped by platform and configuration. Its columns are the upstream ones from 8.1 (spec 09.6); the older columns pinned for the history are searched against these rows, not run dense on them.
fn current_stripe(repo: &Repo) -> Rows {
    let kernels = repo.kernels.in_set("current");
    let mut rows = Vec::new();
    for p in tier1(repo) {
        for &config in STRIPE_CONFIGS {
            let mut crossings = Vec::new();
            for k in &kernels {
                if config == "tinyconfig+gk" && k.version.series(2) < [3, 17].to_vec() {
                    continue;
                }
                for g in crate::search::columns(repo, &p.triple) {
                    if g.version.series(2) >= [8, 1].to_vec() && p.applies(&k.version, &g.version) {
                        crossings.push((k.version.clone(), g.id.clone(), p.name.clone(), config));
                    }
                }
            }
            rows.push((format!("{} {config}", p.name), crossings));
        }
    }
    rows
}

/// The crossings of the GCC 16 column over releases 5.0 on, grouped by platform.
fn gcc16_column(repo: &Repo) -> Rows {
    let Some(g) = repo
        .gccs
        .gccs
        .iter()
        .filter(|g| g.flavor == "upstream" && g.version.parts().first() == Some(&16))
        .max_by(|a, b| a.version.cmp(&b.version))
    else {
        return Vec::new();
    };
    let mut kernels: Vec<Version> = repo
        .kernels
        .kernels
        .iter()
        .filter(|k| k.sets.iter().any(|s| s == "releases" || s == "current"))
        .map(|k| k.version.clone())
        .collect();
    kernels.sort();
    tier1(repo)
        .into_iter()
        .filter(|p| g.targets.contains(&p.triple))
        .map(|p| {
            let crossings = kernels
                .iter()
                .filter(|k| p.applies(k, &g.version))
                .map(|k| (k.clone(), g.id.clone(), p.name.clone(), crate::cell::CONFIG))
                .collect();
            (format!("{} {}", p.name, g.id), crossings)
        })
        .collect()
}

/// A horizontal bar chart of labelled counts, scaled to the largest.
pub(crate) fn chart(out: &mut String, rows: &[(String, f64, String)]) {
    let max = rows.iter().map(|r| r.1).fold(0.0_f64, f64::max);
    out.push_str("```text\n");
    for (label, value, note) in rows {
        // The chart is 40 characters wide at most, drawn in eighths; a non-zero value always gets one.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = if max > 0.0 {
            ((value / max) * 320.0)
                .round()
                .max(if *value > 0.0 { 1.0 } else { 0.0 }) as usize
        } else {
            0
        };
        let _ = writeln!(out, "{label:<12} {} {note}", blocks(n, 40, ' '));
    }
    out.push_str("```\n");
}

/// A table of the newest cells per GCC column: how many of each verdict, and a bar for the share that works.
fn by_gcc(out: &mut String, gccs: &[(&str, Version)], latest: &[&Entry]) {
    let rows: Vec<(&Version, Vec<&&Entry>)> = gccs
        .iter()
        .map(|(id, v)| {
            (
                v,
                latest.iter().filter(|e| e.gcc == *id).collect::<Vec<_>>(),
            )
        })
        .filter(|(_, cells)| !cells.is_empty())
        .collect();
    if rows.is_empty() {
        return;
    }
    out.push_str("\n### By GCC column\n\nThe newest cell at each crossing again, over every kernel, platform and configuration.\n\n| GCC | Cells | 🟩 | 🟨 | 🟧 | 🟥 | ⚠️ | Works |\n|---|--:|--:|--:|--:|--:|--:|---|\n");
    for (v, cells) in rows {
        let n = |verdict: &str| cells.iter().filter(|e| e.verdict == verdict).count();
        let works = n("works");
        let _ = writeln!(
            out,
            "| {v} | {} | {works} | {} | {} | {} | {} | `{}` {}% |",
            cells.len(),
            n("runs"),
            n("builds"),
            n("fails"),
            cells.iter().filter(|e| e.flaky).count(),
            bar(works, cells.len(), 10),
            percent(works, cells.len())
        );
    }
}

/// The status section, markers included.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn section(repo: &Repo, m: &Matrix, date: &str) -> String {
    let mut out = format!(
        "{BEGIN}\n## Status\n\nThis section is written by `gk publish` from [matrix/matrix.json](matrix/matrix.json) and is replaced every time the results are published. Last published {date}, with {} in the matrix.\n",
        count(m.cells.len(), "cell")
    );

    out.push_str("\n### Progress\n\nThe current stripe of G1 is every Current kernel with every upstream GCC column from 8.1 on the tier 1 platforms, with `defconfig+gk` and `tinyconfig+gk`. The GCC 16 column is every release from 5.0 on with the newest GCC 16. A Current kernel that has just moved to a new point release counts the cells of the point before until the new one has run.\n\n");
    progress(&mut out, m, &current_stripe(repo));
    out.push('\n');
    progress(&mut out, m, &gcc16_column(repo));

    out.push_str("\n### Verdicts\n\nThe newest cell at each crossing, over every platform and configuration.\n\n");
    let mut latest: Vec<&Entry> = Vec::new();
    for e in &m.cells {
        let same = |x: &&Entry| {
            x.kernel == e.kernel
                && x.gcc == e.gcc
                && x.platform == e.platform
                && x.config == e.config
        };
        match latest.iter().position(same) {
            Some(i) if latest[i].started >= e.started => {}
            Some(i) => latest[i] = e,
            None => latest.push(e),
        }
    }
    let verdicts: Vec<(String, f64, String)> = ["works", "runs", "builds", "fails"]
        .iter()
        .map(|v| {
            let n = latest.iter().filter(|e| e.verdict == *v).count();
            #[allow(clippy::cast_precision_loss)]
            (
                format!("{} {v}", crate::publish::letter(v)),
                n as f64,
                n.to_string(),
            )
        })
        .collect();
    chart(&mut out, &verdicts);
    out.push_str("\nThe same cells by the rung they reached (spec 02.2), L8 being clean all the way through KUnit and the splat check.\n\n");
    let rungs: Vec<(String, f64, String)> = (0..=8)
        .map(|r| {
            let rung = format!("L{r}");
            let n = latest.iter().filter(|e| e.rung == rung).count();
            #[allow(clippy::cast_precision_loss)]
            (rung, n as f64, n.to_string())
        })
        .collect();
    chart(&mut out, &rungs);

    let mut gccs: Vec<(&str, Version)> = repo
        .gccs
        .gccs
        .iter()
        .filter(|g| g.flavor == "upstream")
        .map(|g| (g.id.as_str(), g.version.clone()))
        .collect();
    gccs.sort_by(|a, b| a.1.cmp(&b.1));
    let times: Vec<(String, f64, String)> = gccs
        .iter()
        .filter_map(|(id, v)| {
            let mut s: Vec<f64> = latest
                .iter()
                .filter(|e| e.gcc == *id && e.config == crate::cell::CONFIG)
                .filter_map(|e| e.build_seconds)
                .collect();
            if s.is_empty() {
                return None;
            }
            s.sort_by(f64::total_cmp);
            let median = s[s.len() / 2] / 60.0;
            Some((
                v.to_string(),
                median,
                format!("{median:.0} min, {}", count(s.len(), "cell")),
            ))
        })
        .collect();
    if !times.is_empty() {
        out.push_str("\nThe median build time of `defconfig+gk` cells per GCC column, in minutes. The machines are shared, so these move with their load.\n\n");
        chart(&mut out, &times);
    }

    by_gcc(&mut out, &gccs, &latest);

    let mut recent = latest.clone();
    recent.sort_by_key(|e| std::cmp::Reverse(e.started));
    if !recent.is_empty() {
        out.push_str("\n### Latest cells\n\n| Date | Kernel | GCC | Platform | Config | Verdict | Rung | Minutes |\n|---|---|---|---|---|---|---|--:|\n");
        for e in recent.iter().take(10) {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} {} | {} | {:.0} |",
                e.date,
                e.kernel,
                e.gcc.trim_start_matches("gcc-"),
                e.platform,
                e.config,
                square(e),
                e.verdict,
                e.rung,
                e.seconds / 60.0
            );
        }
    }

    let _ = write!(
        out,
        "\n### Matrices\n\nOne square per cell: {LEGEND} A table appears once its first cell has run, and then every Current kernel has a row in it. The full heat maps, with warning counts, are in [reports](reports), next to the [warning census](reports/warning-census.md) and the [configuration differential](reports/config-differential.md) of the Current set.\n"
    );
    let current: Vec<Version> = repo
        .kernels
        .in_set("current")
        .iter()
        .map(|k| k.version.clone())
        .collect();
    for p in tier1(repo) {
        let cells: Vec<&Entry> = m.cells.iter().filter(|e| e.platform == p.name).collect();
        let mut cols: Vec<(&str, Version)> = gccs
            .iter()
            .filter(|(id, _)| {
                repo.gccs
                    .get(id)
                    .is_some_and(|g| g.targets.contains(&p.triple))
            })
            .cloned()
            .collect();
        cols.sort_by(|a, b| a.1.cmp(&b.1));
        let mut configs: Vec<&str> = STRIPE_CONFIGS.to_vec();
        for e in &cells {
            if !configs.contains(&e.config.as_str()) {
                configs.push(&e.config);
            }
        }
        for config in configs {
            let mut kernels: Vec<Version> = cells
                .iter()
                .filter(|e| e.config == config)
                .filter_map(|e| e.kernel.parse().ok())
                .collect();
            if STRIPE_CONFIGS.contains(&config) {
                kernels.extend(current.iter().cloned());
            }
            kernels.sort();
            kernels.dedup();
            if kernels.is_empty() {
                continue;
            }
            let mut ran: Vec<(&str, &str)> = cells
                .iter()
                .filter(|e| e.config == config)
                .map(|e| (e.kernel.as_str(), e.gcc.as_str()))
                .collect();
            ran.sort_unstable();
            ran.dedup();
            if ran.is_empty() {
                continue;
            }
            let _ = write!(
                out,
                "\n#### {} {config}, {}\n\n",
                p.name,
                count(ran.len(), "cell")
            );
            table(&mut out, &cells, config, &kernels, &cols, square);
        }
    }
    out.push_str(END);
    out.push('\n');
    out
}

/// The README with its status section replaced, or `None` when the markers are missing.
#[must_use]
pub fn splice(readme: &str, section: &str) -> Option<String> {
    let begin = readme.find(BEGIN)?;
    let end = readme[begin..].find(END)? + begin + END.len();
    let rest = readme[end..].strip_prefix('\n').unwrap_or(&readme[end..]);
    Some(format!("{}{section}{rest}", &readme[..begin]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn bars_fill_in_eighths() {
        assert_eq!(bar(0, 10, 10), "░░░░░░░░░░");
        assert_eq!(bar(1, 10, 10), "█░░░░░░░░░");
        assert_eq!(bar(1, 504, 30), "▏░░░░░░░░░░░░░░░░░░░░░░░░░░░░░");
        assert_eq!(bar(3, 16, 4), "▊░░░");
        assert_eq!(bar(1, 2, 3), "█▌░");
        assert_eq!(bar(10, 10, 10), "██████████");
        assert_eq!(bar(0, 0, 4), "░░░░");
        assert_eq!(blocks(9, 3, ' '), "█▏ ");
    }

    #[test]
    fn the_section_replaces_only_what_is_between_the_markers() {
        let readme = format!("# x\n\nintro\n\n{BEGIN}\nold\n{END}\n\n## Next\n");
        let new = splice(&readme, &format!("{BEGIN}\nnew\n{END}\n")).unwrap();
        assert_eq!(
            new,
            format!("# x\n\nintro\n\n{BEGIN}\nnew\n{END}\n\n## Next\n")
        );
        assert!(splice("no markers", "x").is_none());
    }

    #[test]
    fn a_moved_pin_counts_the_cells_of_the_point_before() {
        let m: Matrix =
            serde_json::from_str(&std::fs::read_to_string("../../matrix/matrix.json").unwrap())
                .unwrap();
        let mut e = m.cells[0].clone();
        e.kernel = "6.12.40".into();
        let cells = vec![e.clone()];
        let at = |k: &str| ran(&cells, &k.parse().unwrap(), &e.gcc, &e.platform, &e.config);
        assert!(at("6.12.40"));
        assert!(at("6.12.48"));
        assert!(!at("6.12.30"));
        assert!(!at("6.13.2"));
    }

    #[test]
    fn the_section_has_progress_charts_and_a_row_per_current_kernel() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let m: Matrix =
            serde_json::from_str(&std::fs::read_to_string("../../matrix/matrix.json").unwrap())
                .unwrap();
        let s = section(&repo, &m, "2026-10-02");
        assert!(s.starts_with(BEGIN) && s.trim_end().ends_with(END));
        assert!(s.contains("x86_64 defconfig+gk"));
        assert!(s.contains("🟩 works"));
        assert!(s.contains("| GCC | Cells |"));
        let current = repo.kernels.in_set("current");
        for k in current {
            assert!(s.contains(&format!("| {} |", k.version)), "{}", k.version);
        }
        assert!(!s.contains('\u{2014}') && !s.contains('\u{2013}'));
    }
}
