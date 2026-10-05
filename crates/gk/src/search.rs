//! `gk search`: run the cells of one matrix row, a kernel on a platform across the GCC columns.
//!
//! The dense search runs every upstream column that targets the platform in version order, except that the kernel's era GCC runs first because the others grade their KUnit runs against it. The frontier search of spec 09.3 starts at the era GCC too, gallops out to each edge of the working set over the last-point columns, binary searches the edge, samples two interior columns and narrows each edge with the point columns. The edge failures are run again with `make -k` (spec 09.5). A row whose era cell does not work, or whose interior sample finds a hole, is run dense.
//!
//! The last-point columns are ordered by version, not by release date as 09.3 has it: the series overlap in time, and 13.5 came out after 16.2, so date order would put it at the newer edge.
//!
//! A cell already in the store with the same identity is not run again unless `--rerun` is given, so a search that dies halfway picks up where it stopped. The one exception is a cell whose build went over its budget, which is run again, since on a loaded machine a timeout says more about the machine than the cell.

use crate::cell::{self, CellRecord, Setup};
use crate::{build, store};
use gk_model::Version;
use gk_model::repo::Repo;
use gk_model::toolchains::Gcc;

/// How to search. Each switch is a command line flag, which is why there are so many bools.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct Options {
    /// Run every column instead of searching for the edges.
    pub dense: bool,
    /// The seed that picks the interior sample. The search prints the one it used, so a run can be repeated.
    pub seed: Option<u64>,
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

/// A GCC's release series: the major version from GCC 5 on, major and minor before.
fn series(v: &Version) -> Vec<u32> {
    v.series(if v.series(1)[0] >= 5 { 1 } else { 2 })
}

/// The column that stands for the era GCC `id` on `triple`: the column itself when it is pinned for the target, or else the newest pinned column of the same release series, as `gcc-10.5.0` stands for `debian-bullseye-gcc-10`.
pub(crate) fn era_column(repo: &Repo, id: &str, triple: &str) -> Option<String> {
    let usable = |g: &&Gcc| g.targets.iter().any(|t| t == triple);
    if let Some(g) = repo.gccs.get(id).filter(usable) {
        return Some(g.id.clone());
    }
    let want: Version = id.rsplit_once("gcc-")?.1.parse().ok()?;
    repo.gccs
        .gccs
        .iter()
        .filter(usable)
        .filter(|g| g.flavor == "upstream" && series(&g.version) == series(&want))
        .max_by(|a, b| a.version.cmp(&b.version))
        .map(|g| g.id.clone())
}

/// Search the row of `kernel` on `platform`, printing each column as it finishes.
pub fn run(
    repo: &Repo,
    kernel: &str,
    platform: &str,
    opts: Options,
) -> Result<Vec<(String, Column)>, String> {
    if !opts.dense {
        return frontier_row(repo, kernel, platform, opts);
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
    let era = repo
        .eras
        .of(&version)
        .and_then(|e| era_column(repo, &e.gcc, &p.triple));
    for g in era_first(cols, era.as_deref()) {
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

/// The columns with the era GCC's, or the one standing for it, moved to the front. Every other cell of the row grades its KUnit run against the era cell (spec 02.4), so that cell has to be in the store before they reach L7.
fn era_first<'a>(mut cols: Vec<&'a Gcc>, era: Option<&str>) -> Vec<&'a Gcc> {
    if let Some(at) = cols.iter().position(|g| Some(g.id.as_str()) == era) {
        let g = cols.remove(at);
        cols.insert(0, g);
    }
    cols
}

fn one(repo: &Repo, version: &Version, gcc: &str, platform: &str, opts: Options) -> Column {
    one_with(repo, version, gcc, platform, opts, false, None)
}

/// One cell, with `keep_going` asking for the `make -k` run when its build fails, and `binutils` laid over the bundle's own when it is given. A stored cell without the `make -k` run is run again when it is asked for.
pub(crate) fn one_with(
    repo: &Repo,
    version: &Version,
    gcc: &str,
    platform: &str,
    opts: Options,
    keep_going: bool,
    binutils: Option<&str>,
) -> Column {
    let setup =
        Setup::new(repo, version.as_str(), gcc, platform, opts.config).and_then(
            |s| match binutils {
                Some(b) => s.with_binutils(repo, b),
                None => Ok(s),
            },
        );
    let setup = match setup {
        Ok(s) if opts.boot => match s.booting(repo) {
            Ok(s) => s,
            Err(e) => return Column::Broken(e),
        },
        Ok(s) => s,
        Err(e) => return Column::Broken(e),
    };
    let mut setup = setup;
    setup.keep_going = keep_going;
    let identity = setup.coordinates.identity();
    if !opts.rerun
        && let Ok(text) = std::fs::read_to_string(store::cell_dir(&identity).join("cell.json"))
        && let Ok(record) = serde_json::from_str::<CellRecord>(&text)
        && record.graded
        && !(keep_going && record.rung == "L3" && record.units_failed.is_none())
        && !build::over_budget(&store::cell_dir(&identity))
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

/// The two edges of a working set found by [`frontier`], as indices into the columns it searched.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frontier {
    /// The oldest column that works and the newest older one that does not, if any does not.
    pub older: (usize, Option<usize>),
    /// The newest column that works and the oldest newer one that does not, if any does not.
    pub newer: (usize, Option<usize>),
}

/// The frontier search of spec 09.3 over `n` ordered columns, starting at `era`. `works` runs the cell of a column and says whether it works. Returns `None` when the era column itself does not work, which sends the row dense.
pub fn frontier(n: usize, era: usize, works: &mut impl FnMut(usize) -> bool) -> Option<Frontier> {
    if era >= n || !works(era) {
        return None;
    }
    Some(Frontier {
        older: side(era, 0, works),
        newer: side(era, n - 1, works),
    })
}

/// Gallop from `from` toward `end`, one column, then two, four and so on, until a column fails or `end` works. Then binary search between the last that worked and the first that failed.
fn side(from: usize, end: usize, works: &mut impl FnMut(usize) -> bool) -> (usize, Option<usize>) {
    let toward = |at: usize, step: usize| {
        if end < from {
            at.saturating_sub(step).max(end)
        } else {
            (at + step).min(end)
        }
    };
    let (mut good, mut step) = (from, 1);
    let bad = loop {
        if good == end {
            return (good, None);
        }
        let next = toward(from, step);
        if works(next) {
            good = next;
            step *= 2;
        } else {
            break next;
        }
    };
    let (mut good, mut bad) = (good, bad);
    while good.abs_diff(bad) > 1 {
        let mid = usize::midpoint(good, bad);
        if works(mid) {
            good = mid;
        } else {
            bad = mid;
        }
    }
    (good, Some(bad))
}

/// Two interior columns, strictly between `lo` and `hi` and not already run, picked by `seed`.
#[must_use]
pub fn interior(lo: usize, hi: usize, ran: &[usize], seed: u64) -> Vec<usize> {
    let mut left: Vec<usize> = (lo + 1..hi).filter(|i| !ran.contains(i)).collect();
    let mut state = seed | 1;
    let mut out = Vec::new();
    while out.len() < 2 && !left.is_empty() {
        // xorshift64, which is enough to spread a sample and needs no crate.
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let len = u64::try_from(left.len()).unwrap_or(u64::MAX);
        let at = usize::try_from(state % len).unwrap_or(0);
        out.push(left.remove(at));
    }
    out.sort_unstable();
    out
}

/// The frontier search of one row. The cells it runs come back in version order.
#[allow(clippy::too_many_lines)]
fn frontier_row(
    repo: &Repo,
    kernel: &str,
    platform: &str,
    opts: Options,
) -> Result<Vec<(String, Column)>, String> {
    let version: Version = kernel
        .trim_start_matches("linux-")
        .parse()
        .map_err(|e| format!("kernel {kernel}: {e:?}"))?;
    let plat = repo
        .platforms
        .get(platform)
        .ok_or_else(|| format!("platform {platform} is not in platforms.toml"))?;
    let usable: Vec<&Gcc> = columns(repo, &plat.triple)
        .into_iter()
        .filter(|g| plat.applies(&version, &g.version))
        .collect();
    let era = repo
        .eras
        .of(&version)
        .and_then(|e| era_column(repo, &e.gcc, &plat.triple))
        .ok_or_else(|| format!("{version} has no era column on {platform}"))?;
    let last: Vec<&Gcc> = usable
        .iter()
        .copied()
        .filter(|g| g.id == era || g.columns.iter().any(|c| c == "last-point"))
        .collect();
    let at = last
        .iter()
        .position(|g| g.id == era)
        .ok_or_else(|| format!("the era column {era} does not apply on {platform}"))?;
    let seed = opts.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_secs())
    });
    println!("frontier search of {version} on {platform} from {era}, seed {seed}");

    let mut ran: std::collections::BTreeMap<String, Column> = std::collections::BTreeMap::new();
    let run_one =
        |g: &Gcc, keep_going: bool, ran: &mut std::collections::BTreeMap<String, Column>| -> bool {
            if !keep_going && let Some(c) = ran.get(&g.id) {
                return works(c);
            }
            let c = one_with(repo, &version, &g.id, platform, opts, keep_going, None);
            print_column(&g.id, &c);
            let ok = works(&c);
            ran.insert(g.id.clone(), c);
            ok
        };
    let found = frontier(last.len(), at, &mut |i| run_one(last[i], false, &mut ran));
    let Some(found) = found else {
        println!("the era cell does not work, so the row is run dense");
        return run(
            repo,
            kernel,
            platform,
            Options {
                dense: true,
                ..opts
            },
        );
    };
    let visited: Vec<usize> = (0..last.len())
        .filter(|i| ran.contains_key(&last[*i].id))
        .collect();
    for i in interior(found.older.0, found.newer.0, &visited, seed) {
        if !run_one(last[i], false, &mut ran) {
            println!(
                "{} inside the working set does not work: a hole, so the row is run dense",
                last[i].id
            );
            return run(
                repo,
                kernel,
                platform,
                Options {
                    dense: true,
                    ..opts
                },
            );
        }
    }

    // Narrow each edge with the point columns between its two last points, by the same binary search. The cell that fails at the edge is the one that explains it, so it gets the make -k run.
    for (good, bad) in [found.older, found.newer] {
        let Some(bad) = bad else { continue };
        let (from, to) = (&last[good].version, &last[bad].version);
        let (lo, hi) = if from < to { (from, to) } else { (to, from) };
        let mut line: Vec<&Gcc> = usable
            .iter()
            .copied()
            .filter(|g| g.version > *lo && g.version < *hi)
            .collect();
        if bad < good {
            line.reverse();
        }
        line.insert(0, last[good]);
        line.push(last[bad]);
        let (mut pass, mut fail) = (0, line.len() - 1);
        while fail - pass > 1 {
            let mid = usize::midpoint(pass, fail);
            if run_one(line[mid], false, &mut ran) {
                pass = mid;
            } else {
                fail = mid;
            }
        }
        run_one(line[fail], true, &mut ran);
    }

    let mut row: Vec<(String, Column)> = ran.into_iter().collect();
    let version_of = |id: &str| repo.gccs.get(id).map(|g| g.version.clone());
    row.sort_by_key(|(id, _)| version_of(id));
    print_edges(&row);
    Ok(row)
}

/// Whether a column's cell works.
pub(crate) fn works(c: &Column) -> bool {
    matches!(c, Column::Ran { record, .. } if record.verdict == "works")
}

/// The edges of a searched row, in words.
pub(crate) fn print_edges(row: &[(String, Column)]) {
    let ok: Vec<usize> = (0..row.len()).filter(|i| works(&row[*i].1)).collect();
    let (Some(&first), Some(&last)) = (ok.first(), ok.last()) else {
        println!("no column works");
        return;
    };
    if first > 0 {
        println!(
            "older edge: {} works, {} does not",
            row[first].0,
            row[first - 1].0
        );
    } else {
        println!("older edge: none, the oldest column run works");
    }
    if last + 1 < row.len() {
        println!(
            "newer edge: {} works, {} does not",
            row[last].0,
            row[last + 1].0
        );
    } else {
        println!("newer edge: none, the newest column run works");
    }
}

pub(crate) fn print_column(gcc: &str, column: &Column) {
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
    fn the_x86_64_row_has_every_column_in_order() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let ids: Vec<&str> = columns(&repo, "x86_64-linux-gnu")
            .iter()
            .map(|g| g.id.as_str())
            .collect();
        assert_eq!(ids.first(), Some(&"gcc-3.2.3"));
        assert_eq!(ids.last(), Some(&"gcc-16.2.0"));
        assert_eq!(ids.len(), 33);
        assert_eq!(columns(&repo, "aarch64-linux-gnu").len(), 20);
        let i686 = columns(&repo, "i686-linux-gnu");
        assert_eq!(i686.first().map(|g| g.id.as_str()), Some("gcc-2.5.8"));
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
        assert_eq!(order[1], "gcc-3.2.3");
        assert_eq!(order.len(), cols.len());
        assert_eq!(era_first(cols.clone(), None).len(), cols.len());
    }

    #[test]
    fn a_distribution_era_gcc_is_stood_for_by_its_series() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let t = "x86_64-linux-gnu";
        assert_eq!(
            era_column(&repo, "debian-bullseye-gcc-10", t).as_deref(),
            Some("gcc-10.5.0")
        );
        assert_eq!(
            era_column(&repo, "gcc-14.2.0", t).as_deref(),
            Some("gcc-14.2.0")
        );
        assert_eq!(
            era_column(&repo, "gcc-3.4.6", t).as_deref(),
            Some("gcc-3.4.6")
        );
        assert_eq!(era_column(&repo, "gcc-2.95.3", t), None);
        assert_eq!(
            era_column(&repo, "gcc-2.95.3", "i686-linux-gnu").as_deref(),
            Some("gcc-2.95.3")
        );
    }

    /// Run the frontier search over a row whose working set is `ok`, and return the edges and the columns it ran.
    fn search(n: usize, era: usize, ok: std::ops::Range<usize>) -> (Option<Frontier>, Vec<usize>) {
        let mut ran = Vec::new();
        let found = frontier(n, era, &mut |i| {
            ran.push(i);
            ok.contains(&i)
        });
        ran.sort_unstable();
        ran.dedup();
        (found, ran)
    }

    #[test]
    fn the_frontier_finds_both_edges() {
        let (found, ran) = search(36, 20, 9..30);
        assert_eq!(
            found,
            Some(Frontier {
                older: (9, Some(8)),
                newer: (29, Some(30)),
            })
        );
        // Each side gallops five times and bisects three, against 36 for the dense row.
        assert_eq!(ran.len(), 17, "ran {ran:?}");
    }

    #[test]
    fn a_row_that_works_to_the_end_has_no_edge_there() {
        let (found, _) = search(12, 3, 0..12);
        assert_eq!(
            found,
            Some(Frontier {
                older: (0, None),
                newer: (11, None),
            })
        );
        let (found, _) = search(12, 11, 5..12);
        assert_eq!(found.unwrap().newer, (11, None));
    }

    #[test]
    fn an_era_cell_that_fails_sends_the_row_dense() {
        assert_eq!(search(12, 3, 5..12).0, None);
    }

    #[test]
    fn the_interior_sample_is_two_unrun_columns_and_repeats_with_its_seed() {
        let a = interior(2, 20, &[3, 5, 11], 42);
        assert_eq!(a.len(), 2);
        assert!(
            a.iter()
                .all(|i| (3..20).contains(i) && ![5, 11].contains(i))
        );
        assert!(!a.contains(&3));
        assert_eq!(a, interior(2, 20, &[3, 5, 11], 42));
        assert_eq!(interior(4, 6, &[], 7), [5]);
        assert!(interior(4, 5, &[], 7).is_empty());
    }
}
