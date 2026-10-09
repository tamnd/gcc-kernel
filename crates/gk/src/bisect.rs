//! `gk history` and `gk bisect-kernel`: the full history clone and the bisection of an edge over it (spec 03.5).
//!
//! The clone is a bare repository at `GK_HISTORY`, or `history/linux.git` in the cache. `gk history update` makes it from Linus's tree and adds the tags of the stable tree, which is enough to bisect anything from 2.6.12 on, mainline or stable. The history from before git joins it at G3.
//!
//! A bisection runs cells of the form (commit, G, P) with the row's configuration and host, judged by whether they reach a rung. It uses `git bisect --no-checkout` on the bare clone, so no work tree is kept: each commit's tree is exported with `git archive`, built, and removed. Commit cells go to the store like any other, with `git:` and the commit as the kernel digest, and `gk publish` leaves them out of the matrix. A commit whose cell misses keeps the first line of its first error in the record, so a bisection that lands on a merge can be read. A bisection at L1 is of a refusal, and since L1 is the accept probe, each commit and each end runs the probe alone and leaves no cell. A bisection of a build failure can name the object that fails with `--unit`, and then each commit, and each end, builds that object alone and leaves no cell either.

use crate::cell::{self, CellRecord, Setup};
use crate::{build, classify, fetch, store};
use gk_model::Version;
use gk_model::repo::Repo;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Linus's tree.
const MAINLINE: &str = "https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git";

/// The stable tree, whose tags are the point releases.
const STABLE: &str = "https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git";

/// The most commits a bisection tests. 2.6.12 to 7.2 is about 1.4 million commits, which is 21 steps.
const MAX_STEPS: usize = 40;

/// Stretches of history that cannot build out of tree for a reason of their own, each as the commit that broke it and the one that fixed it. gk always builds with `O=`, so a bisection would skip such commits one at a time and could run out of steps among them. Every commit that has the first and not the second is skipped before the first step instead.
const UNBUILDABLE: &[(&str, &str)] = &[
    // "kbuild: create .kernelrelease at *config step" makes .kernelrelease in the source tree, where O= has no .config, until "kbuild: fix build with O=..". That is 1353 of the 5734 commits from 2.6.15 to 2.6.16.
    ("2244cbd8a918", "8c7f75d3257f"),
];

/// The clone: `GK_HISTORY`, or `history/linux.git` in the cache.
#[must_use]
pub fn history_dir() -> PathBuf {
    std::env::var_os("GK_HISTORY").map_or_else(
        || fetch::cache_dir().join("history").join("linux.git"),
        PathBuf::from,
    )
}

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("running git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Make the clone, or bring it up to date. Run weekly.
pub fn update() -> Result<(), String> {
    let dir = history_dir();
    if !dir.join("HEAD").is_file() {
        let parent = dir
            .parent()
            .ok_or("the history clone has no parent directory")?;
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("creating {}: {e}", parent.display()))?;
        println!("cloning {MAINLINE} into {}", dir.display());
        let status = Command::new("git")
            .args(["clone", "--bare", "--quiet", MAINLINE])
            .arg(&dir)
            .status()
            .map_err(|e| format!("running git: {e}"))?;
        if !status.success() {
            return Err("cloning the mainline tree failed".into());
        }
    }
    println!("fetching mainline and the stable tags");
    git(
        &dir,
        &[
            "fetch",
            "--quiet",
            MAINLINE,
            "+refs/heads/master:refs/heads/master",
            "+refs/tags/*:refs/tags/*",
        ],
    )?;
    git(
        &dir,
        &["fetch", "--quiet", STABLE, "+refs/tags/*:refs/tags/*"],
    )?;
    let tags = git(&dir, &["tag", "--list", "v*"])?.lines().count();
    println!("{}: {tags} tags", dir.display());
    Ok(())
}

/// The tag of a release, as `v3.0` or `v4.9.337`.
fn tag(kernel: &str) -> String {
    format!(
        "v{}",
        kernel.trim_start_matches("linux-").trim_start_matches('v')
    )
}

/// The version a tree says it is, from the top of its `Makefile`, as `3.17` or `3.16.7`.
fn makefile_version(makefile: &str) -> Option<Version> {
    let field = |name: &str| {
        makefile.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == name).then(|| v.trim().to_owned())
        })
    };
    let (major, minor, sub) = (field("VERSION")?, field("PATCHLEVEL")?, field("SUBLEVEL")?);
    let text = if sub.is_empty() || sub == "0" {
        format!("{major}.{minor}")
    } else {
        format!("{major}.{minor}.{sub}")
    };
    text.parse().ok()
}

/// What a bisection asks of each cell.
#[derive(Debug, Clone)]
pub struct Options {
    /// The configuration.
    pub config: &'static str,
    /// The rung a cell must reach to count as passing, or `None` for the higher rung of the two ends.
    pub rung: Option<String>,
    /// Parallel jobs for each build.
    pub jobs: usize,
    /// The one object to build instead of the whole kernel, for a failure at L3.
    pub unit: Option<String>,
    /// The binutils laid over the GCC bundle's own, for a signature that comes from binutils, or `None` for the bundle's.
    pub binutils: Option<String>,
}

/// One tested commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tested {
    commit: String,
    version: String,
    rung: String,
    term: String,
    /// Why a commit whose cell was built missed the rung: the first line of its first error. A bisection that ends on a merge is read with these, since a branch can miss for another reason than the one bisected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// The record a bisection leaves in the store under `bisections/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bisection {
    from: String,
    to: String,
    gcc: String,
    platform: String,
    config: String,
    rung: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    binutils: Option<String>,
    first: String,
    subject: String,
    contained_in: String,
    #[serde(default)]
    tested: Vec<Tested>,
}

/// The rung of a release cell, run or from the store.
fn release_rung(
    repo: &Repo,
    kernel: &str,
    gcc: &str,
    platform: &str,
    opts: &Options,
    boot: bool,
) -> Result<String, String> {
    let setup = Setup::new(repo, kernel, gcc, platform, opts.config)?;
    let setup = with_binutils(repo, setup, opts)?;
    let setup = if boot { setup.booting(repo)? } else { setup };
    if let Some(object) = &opts.unit {
        let (rung, error) = unit_rung(&setup, object, opts.jobs)?;
        println!("{kernel:<10} {rung:<3} {object} {}", unit_term(&rung));
        if let Some(e) = error {
            println!("           {e}");
        }
        return Ok(rung);
    }
    // An end with no cell yet would only be built to show it gets past the probe, so it is probed, like the commits between.
    let stored = store::cell_dir(&setup.coordinates.identity()).join("cell.json");
    if opts.rung.as_deref() == Some("L1") && !stored.is_file() {
        let rung = probe_rung(&setup, opts.jobs)?;
        println!(
            "{kernel:<10} {rung:<3} {}",
            if rung == "L1" { "accepted" } else { "refused" }
        );
        return Ok(rung);
    }
    let record = run_or_load(repo, &setup, opts.jobs)?;
    println!("{kernel:<10} {:<3} {}", record.rung, record.verdict);
    Ok(record.rung)
}

/// The rung of a tree from one object, `L3` when it builds and `L2` when it does not, and when it does not, the first error its logs give.
fn unit_rung(setup: &Setup, object: &str, jobs: usize) -> Result<(String, Option<String>), String> {
    let dir = fetch::cache_dir()
        .join("scratch")
        .join(format!("unit-{}", setup.coordinates.short_id()));
    let built = cell::unit(setup, &dir, object, jobs);
    let error = matches!(built, Ok(false)).then(|| unit_error(&dir)).flatten();
    let _ = std::fs::remove_dir_all(&dir);
    Ok((if built? { "L3" } else { "L2" }.into(), error))
}

/// The first line of a failed unit's log that names an error, or else its last line that says why kbuild stopped. The object can fail for another reason than the one being bisected, and the bisection would follow that one without a word, so its output shows the line.
fn unit_error(dir: &Path) -> Option<String> {
    let log = ["unit.log", "prepare.log"]
        .iter()
        .filter_map(|l| std::fs::read_to_string(dir.join(l)).ok())
        .find(|t| !t.trim().is_empty())?;
    let line = log
        .lines()
        .filter(|l| !l.starts_with("make"))
        .find(|l| l.contains("error:") || l.contains("Error:"))
        .map(str::to_owned)
        .or_else(|| build::failure_lines(&log, 1).pop())?;
    Some(line.trim().chars().take(300).collect())
}

/// The cell with the bisection's binutils, when it names one.
fn with_binutils(repo: &Repo, setup: Setup, opts: &Options) -> Result<Setup, String> {
    match &opts.binutils {
        Some(b) => setup.with_binutils(repo, b),
        None => Ok(setup),
    }
}

fn unit_term(rung: &str) -> &'static str {
    if rung == "L3" { "builds" } else { "fails" }
}

fn run_or_load(repo: &Repo, setup: &Setup, jobs: usize) -> Result<CellRecord, String> {
    let dir = store::cell_dir(&setup.coordinates.identity());
    if let Ok(text) = std::fs::read_to_string(dir.join("cell.json"))
        && let Ok(record) = serde_json::from_str::<CellRecord>(&text)
    {
        if !build::rig_failed(&dir) {
            return Ok(record);
        }
        println!("the stored cell failed for the machine's reasons, building it again");
    }
    let record = cell::run(repo, setup, jobs, false).map(|(_, r)| r)?;
    // A miss the machine caused would send the bisection down the wrong half, so the step is skipped instead.
    if build::rig_failed(&dir) {
        return Err("the build failed for the machine's reasons and not the commit's".into());
    }
    Ok(record)
}

/// The first line of a failed cell's first error, cut to 300 characters, or `None` when the cell has none to show.
fn first_error(setup: &Setup, r: &CellRecord) -> Option<String> {
    let dir = store::cell_dir(&setup.coordinates.identity());
    let f = classify::failure(&dir, r)?;
    let line: String = f
        .first_error
        .lines()
        .next()?
        .trim()
        .chars()
        .take(300)
        .collect();
    (!line.is_empty()).then_some(line)
}

/// The rung of a commit from the accept probe alone, `L1` or `L0`. L1 is the probe, so a bisection of a refusal builds nothing and stores no cell.
fn probe_rung(setup: &Setup, jobs: usize) -> Result<String, String> {
    let dir = fetch::cache_dir()
        .join("scratch")
        .join(format!("probe-{}", setup.coordinates.short_id()));
    let probe = cell::probe(setup, &dir, jobs);
    let _ = std::fs::remove_dir_all(&dir);
    Ok(if probe?.passes() { "L1" } else { "L0" }.into())
}

/// Mark the commits of [`UNBUILDABLE`] between the two ends as skipped: those that descend from the commit that broke the build and not from the one that fixed it. A stretch whose first commit the clone does not have is left alone.
fn skip_unbuildable(history: &Path, old_tag: &str, new_tag: &str) -> Result<(), String> {
    // Strict descendants of `c` between the ends.
    let after = |c: &str| -> Result<std::collections::BTreeSet<String>, String> {
        let list = git(
            history,
            &[
                "rev-list",
                &format!("--ancestry-path={c}"),
                new_tag,
                &format!("^{old_tag}"),
                &format!("^{c}"),
            ],
        )?;
        Ok(list.split_whitespace().map(str::to_owned).collect())
    };
    let between = |c: &str| {
        git(history, &["merge-base", "--is-ancestor", old_tag, c]).is_ok()
            && git(history, &["merge-base", "--is-ancestor", c, new_tag]).is_ok()
    };
    for (broke, fixed) in UNBUILDABLE {
        let Ok(broke) = git(
            history,
            &[
                "rev-parse",
                "--verify",
                "-q",
                &format!("{broke}^{{commit}}"),
            ],
        ) else {
            continue;
        };
        let broke = broke.trim();
        let fixed = git(
            history,
            &["rev-parse", "--verify", &format!("{fixed}^{{commit}}")],
        )?;
        let fixed = fixed.trim();
        let mut skip = after(broke)?;
        if between(broke) {
            skip.insert(broke.to_owned());
        }
        for c in after(fixed)? {
            skip.remove(&c);
        }
        skip.remove(fixed);
        if skip.is_empty() {
            continue;
        }
        println!(
            "skipping {} commits that have {} and not {}",
            skip.len(),
            &broke[..12],
            &fixed[..12]
        );
        let skip: Vec<&str> = skip.iter().map(String::as_str).collect();
        for chunk in skip.chunks(500) {
            let mut args = vec!["bisect", "skip"];
            args.extend(chunk);
            git(history, &args)?;
        }
    }
    Ok(())
}

/// Export a commit's tree into the cache.
fn export(history: &Path, commit: &str) -> Result<PathBuf, String> {
    let tree = fetch::cache_dir()
        .join("trees")
        .join(format!("git-{commit}"));
    let _ = std::fs::remove_dir_all(&tree);
    std::fs::create_dir_all(&tree).map_err(|e| format!("creating {}: {e}", tree.display()))?;
    let mut archive = Command::new("git")
        .arg("-C")
        .arg(history)
        .args(["archive", "--format=tar", commit])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("running git archive: {e}"))?;
    let status = Command::new("tar")
        .arg("-xf")
        .arg("-")
        .arg("-C")
        .arg(&tree)
        .stdin(archive.stdout.take().ok_or("git archive has no output")?)
        .status()
        .map_err(|e| format!("running tar: {e}"))?;
    // Waiting reaps git, which otherwise stays a zombie for the rest of a bisection, one per step.
    let archived = archive.wait().is_ok_and(|s| s.success());
    if !status.success() || !archived {
        let _ = std::fs::remove_dir_all(&tree);
        return Err(format!("exporting {commit} failed"));
    }
    Ok(tree)
}

/// Bisect between two releases for the first commit on which the cell of `gcc` on `platform` changes whether it reaches the rung.
pub fn run(
    repo: &Repo,
    from: &str,
    to: &str,
    gcc: &str,
    platform: &str,
    opts: &Options,
) -> Result<(), String> {
    let history = history_dir();
    if !history.join("HEAD").is_file() {
        return Err(format!(
            "{} is not there yet; run gk history update",
            history.display()
        ));
    }
    let lock = history.join("gk-bisect.lock");
    if lock.exists() {
        return Err(format!(
            "{} exists, so another bisection is running on this clone",
            lock.display()
        ));
    }
    std::fs::write(&lock, std::process::id().to_string())
        .map_err(|e| format!("writing {}: {e}", lock.display()))?;
    let result = bisect(repo, &history, from, to, gcc, platform, opts);
    let _ = git(&history, &["bisect", "reset"]);
    let _ = std::fs::remove_file(&lock);
    result
}

#[allow(clippy::too_many_lines)]
fn bisect(
    repo: &Repo,
    history: &Path,
    from: &str,
    to: &str,
    gcc: &str,
    platform: &str,
    opts: &Options,
) -> Result<(), String> {
    let (old_tag, new_tag) = (tag(from), tag(to));
    for t in [&old_tag, &new_tag] {
        git(
            history,
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{t}^{{commit}}"),
            ],
        )
        .map_err(|_| format!("{t} is not in the history clone"))?;
    }
    if opts.unit.is_some() && opts.rung.as_deref().is_some_and(|r| r != "L3") {
        return Err("--unit bisects a build failure, which is at L3".into());
    }
    let boot = opts.rung.as_deref().is_some_and(|r| r >= "L5");
    let r_old = release_rung(repo, from, gcc, platform, opts, boot)?;
    let r_new = release_rung(repo, to, gcc, platform, opts, boot)?;
    let rung = match (&opts.unit, &opts.rung) {
        (Some(_), _) => "L3".to_owned(),
        (None, Some(r)) => r.clone(),
        (None, None) => r_old.clone().max(r_new.clone()),
    };
    let passes = |r: &str| r >= rung.as_str();
    let (p_old, p_new) = (passes(&r_old), passes(&r_new));
    if p_old == p_new {
        return Err(format!(
            "{from} reaches {r_old} and {to} reaches {r_new}, so both ends {} {rung} and there is nothing to bisect",
            if p_old { "reach" } else { "miss" }
        ));
    }
    let term = |p: bool| if p { "reaches" } else { "misses" };
    let (old_term, new_term) = (term(p_old), term(p_new));
    match &opts.unit {
        Some(object) => println!(
            "bisecting {old_tag}..{new_tag} for the first commit on which {object} {}",
            if p_new { "builds" } else { "fails" }
        ),
        None => println!(
            "bisecting {old_tag}..{new_tag} for the first commit whose cell {new_term} {rung}"
        ),
    }
    let _ = git(history, &["bisect", "reset"]);
    git(
        history,
        &[
            "bisect",
            "start",
            "--no-checkout",
            &format!("--term-old={old_term}"),
            &format!("--term-new={new_term}"),
            &new_tag,
            &old_tag,
        ],
    )?;
    skip_unbuildable(history, &old_tag, &new_tag)?;
    let mut tested = Vec::new();
    let mut first = None;
    for _ in 0..MAX_STEPS {
        let commit = git(history, &["rev-parse", "BISECT_HEAD"])?
            .trim()
            .to_owned();
        let tree = export(history, &commit)?;
        let makefile = std::fs::read_to_string(tree.join("Makefile")).unwrap_or_default();
        let outcome = match makefile_version(&makefile) {
            Some(version) => Setup::at_commit(
                repo,
                &commit,
                &version,
                tree.clone(),
                gcc,
                platform,
                opts.config,
            )
            .and_then(|s| with_binutils(repo, s, opts))
            .and_then(|s| if boot { s.booting(repo) } else { Ok(s) })
            .and_then(|s| {
                if let Some(object) = &opts.unit {
                    unit_rung(&s, object, opts.jobs)
                } else if rung == "L1" {
                    probe_rung(&s, opts.jobs).map(|r| (r, None))
                } else {
                    run_or_load(repo, &s, opts.jobs).map(|r| {
                        let error = (!passes(&r.rung)).then(|| first_error(&s, &r)).flatten();
                        (r.rung, error)
                    })
                }
            })
            .map(|(r, error)| (version, r, error)),
            None => Err("the Makefile has no version".into()),
        };
        let _ = std::fs::remove_dir_all(&tree);
        let said = match &outcome {
            Ok((version, r, error)) => {
                let t = term(passes(r));
                println!("{} {:<10} {:<3} {t}", &commit[..12], version.as_str(), r);
                if let Some(e) = error {
                    println!("             {e}");
                }
                tested.push(Tested {
                    commit: commit.clone(),
                    version: version.as_str().to_owned(),
                    rung: r.clone(),
                    term: t.into(),
                    error: error.clone(),
                });
                git(history, &["bisect", t])?
            }
            Err(e) => {
                println!("{} skipped: {e}", &commit[..12]);
                git(history, &["bisect", "skip"])?
            }
        };
        if let Some(line) = said
            .lines()
            .find(|l| l.contains(&format!("is the first {new_term} commit")))
        {
            first = line.split_whitespace().next().map(str::to_owned);
            break;
        }
        if said.contains("only 'skip'ped commits left") {
            return Err(format!("only skipped commits are left:\n{said}"));
        }
    }
    let first = first.ok_or_else(|| format!("no answer after {MAX_STEPS} steps"))?;
    let subject = git(history, &["log", "-1", "--format=%s", &first])?
        .trim()
        .to_owned();
    let contained_in = git(
        history,
        &["describe", "--contains", "--match", "v*", &first],
    )
    .map(|s| {
        s.trim()
            .split(['~', '^'])
            .next()
            .unwrap_or_default()
            .to_owned()
    })
    .unwrap_or_default();
    println!(
        "first commit whose cell {new_term} {rung}: {} {subject}{}",
        &first[..12],
        if contained_in.is_empty() {
            String::new()
        } else {
            format!(" (in {contained_in})")
        }
    );
    let record = Bisection {
        from: from.into(),
        to: to.into(),
        gcc: gcc.into(),
        platform: platform.into(),
        config: opts.config.into(),
        rung,
        unit: opts.unit.clone(),
        binutils: opts.binutils.clone(),
        first,
        subject,
        contained_in,
        tested,
    };
    let dir = store::root().join("bisections");
    std::fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let unit = opts
        .unit
        .as_ref()
        .map(|u| format!("-{}", u.replace('/', "_")))
        .unwrap_or_default();
    let binutils = opts
        .binutils
        .as_ref()
        .map(|b| format!("-{b}"))
        .unwrap_or_default();
    let name = format!(
        "{from}-{to}-{gcc}{binutils}-{platform}-{}{unit}.json",
        opts.config
    );
    let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())? + "\n";
    std::fs::write(dir.join(&name), json).map_err(|e| format!("writing bisections/{name}: {e}"))?;
    Ok(())
}

/// The bisections in the store, oldest range first.
fn stored() -> Vec<Bisection> {
    let mut out: Vec<Bisection> = std::fs::read_dir(store::root().join("bisections"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|text| serde_json::from_str(&text).ok())
        .collect();
    let key = |b: &Bisection| b.from.parse::<Version>().ok();
    out.sort_by(|a, b| key(a).cmp(&key(b)).then(a.gcc.cmp(&b.gcc)));
    out
}

/// Whether a fixing commit as the catalog abbreviates it and a full commit id are the same commit.
fn same_commit(fix: &str, full: &str) -> bool {
    !fix.is_empty() && full.starts_with(fix)
}

/// `reports/bisections.md`: which fixing commits of the catalog a bisection has confirmed, and every bisection in the store with what it found.
#[must_use]
pub fn report(repo: &Repo, m: &crate::publish::Matrix) -> String {
    report_of(repo, &stored(), &failing(m))
}

/// How many cells of the matrix each class has, counting only the newest run of each coordinate, since a rerun can clear an old failure.
fn failing(m: &crate::publish::Matrix) -> BTreeMap<String, usize> {
    let mut newest: BTreeMap<(&str, &str, &str, &str, &str), &crate::publish::Entry> =
        BTreeMap::new();
    for e in &m.cells {
        let key = (
            e.kernel.as_str(),
            e.gcc.as_str(),
            e.binutils.as_str(),
            e.platform.as_str(),
            e.config.as_str(),
        );
        let slot = newest.entry(key).or_insert(e);
        if e.started > slot.started {
            *slot = e;
        }
    }
    let mut out = BTreeMap::new();
    for e in newest.values().filter(|e| !e.class.is_empty()) {
        *out.entry(e.class.clone()).or_insert(0) += 1;
    }
    out
}

fn report_of(repo: &Repo, runs: &[Bisection], cells: &BTreeMap<String, usize>) -> String {
    let mut out = String::from(
        "# Bisections\n\nEvery `fixed-by` commit of the failure catalog is confirmed by bisecting between a kernel the signature matches and one it does not, with the signature's GCC (spec 03.5 and 08.2). When the bisection finds another commit, the bisected commit wins and the signature is corrected. A signature can name several commits, one for each place the kernel had to change, and a bisection confirms the one its range and platform reach. Written by `gk publish` from `signatures.toml` and the bisections in the result store.\n\n",
    );
    let with_fix: Vec<_> = repo
        .signatures
        .signatures
        .iter()
        .filter(|s| !s.fixed_by.is_empty())
        .collect();
    let confirmed =
        |commit: &str, flag: bool| flag || runs.iter().any(|b| same_commit(commit, &b.first));
    let done = with_fix
        .iter()
        .filter(|s| s.fixed_by.iter().any(|f| confirmed(&f.commit, f.bisected)))
        .count();
    let unseen = with_fix
        .iter()
        .filter(|s| !s.fixed_by.iter().any(|f| confirmed(&f.commit, f.bisected)))
        .filter(|s| !cells.contains_key(&s.class))
        .count();
    let _ = write!(
        out,
        "{} bisections in the store. {done} of the {} signatures with a fixing commit have at least one of their commits confirmed. {unseen} of the others match no cell of the matrix yet, so there is no failing kernel to bisect from until one does. Cells counts the newest run of each coordinate that the signature matches.\n\n## Signatures\n\n| Signature | Rung | Kind | Fixing commits | Confirmed | Cells |\n|---|---|---|---|---|--:|\n",
        runs.len(),
        with_fix.len()
    );
    for s in &with_fix {
        let commits: Vec<String> = s
            .fixed_by
            .iter()
            .map(|f| format!("`{}`", f.commit))
            .collect();
        let found: Vec<String> = s
            .fixed_by
            .iter()
            .filter(|f| confirmed(&f.commit, f.bisected))
            .map(|f| format!("`{}`", f.commit))
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            s.class,
            s.rung,
            s.kind,
            commits.join(" "),
            found.join(" "),
            cells.get(&s.class).copied().unwrap_or(0)
        );
    }
    out.push_str("\n## Runs\n\n");
    if runs.is_empty() {
        out.push_str("No bisections yet.\n");
        return out;
    }
    out.push_str("| Range | GCC | Platform | Judged by | Steps | First commit | Subject | In | Signature |\n|---|---|---|---|--:|---|---|---|---|\n");
    for b in runs {
        let classes: Vec<&str> = repo
            .signatures
            .signatures
            .iter()
            .filter(|s| s.fixed_by.iter().any(|f| same_commit(&f.commit, &b.first)))
            .map(|s| s.class.as_str())
            .collect();
        let _ = writeln!(
            out,
            "| {}..{} | {} | {} | {} | {} | `{}` | {} | {} | {} |",
            b.from,
            b.to,
            b.binutils
                .as_ref()
                .map_or_else(|| b.gcc.clone(), |u| format!("{} with {u}", b.gcc)),
            b.platform,
            b.unit
                .as_ref()
                .map_or_else(|| b.rung.clone(), |u| format!("`{u}`")),
            b.tested.len(),
            b.first.get(..12).unwrap_or(&b.first),
            crate::census::escape(&b.subject),
            b.contained_in,
            if classes.is_empty() {
                "none".to_owned()
            } else {
                classes.join(", ")
            }
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_unit_says_what_failed() {
        let dir = std::env::temp_dir().join(format!("gk-unit-error-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(unit_error(&dir), None);
        std::fs::write(dir.join("unit.log"), "  AS      arch/x86_64/kernel/head.o\n/src/arch/x86_64/kernel/head.S:329: Error: missing ')'\nmake[1]: *** [arch/x86_64/kernel/head.o] Error 1\n").unwrap();
        assert_eq!(unit_error(&dir).as_deref(), Some("/src/arch/x86_64/kernel/head.S:329: Error: missing ')'"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_tree_says_its_version_in_the_makefile() {
        let top = "# SPDX\nVERSION = 3\nPATCHLEVEL = 17\nSUBLEVEL = 0\nEXTRAVERSION = -rc3\nNAME = Shuffling Zombie Juror\n";
        assert_eq!(makefile_version(top).unwrap().as_str(), "3.17");
        let stable = "VERSION = 4\nPATCHLEVEL = 9\nSUBLEVEL = 337\nEXTRAVERSION =\n";
        assert_eq!(makefile_version(stable).unwrap().as_str(), "4.9.337");
        assert!(makefile_version("all:\n").is_none());
    }

    #[test]
    fn the_report_matches_a_bisection_to_the_signature_it_confirms() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let run = |first: &str| Bisection {
            from: "5.7".into(),
            to: "5.8".into(),
            gcc: "gcc-4.8.5".into(),
            platform: "x86_64".into(),
            config: "defconfig+gk".into(),
            rung: "L1".into(),
            unit: None,
            binutils: None,
            first: first.into(),
            subject: "compiler.h: raise minimum | something".into(),
            contained_in: "v5.8-rc1".into(),
            tested: Vec::new(),
        };
        let text = report_of(
            &repo,
            &[run("6ec4476ac82512f09c94aff5972654b70f3772b2")],
            &BTreeMap::new(),
        );
        assert!(text.contains("| gcc-min-49 | L1 | refusal | `6ec4476ac825` | `6ec4476ac825` |"));
        assert!(
            text.contains("`6ec4476ac825` | compiler.h: raise minimum \\| something | v5.8-rc1 |")
        );
        assert!(text.contains("gcc-min-49"));
        let other = report_of(&repo, &[run("0123456789abcdef")], &BTreeMap::new());
        assert!(other.contains("| none |"));
        // gcc-min-49 is marked bisected in signatures.toml, so the entry left open is too-old-generic, which names the same commit.
        assert!(other.contains("| too-old-generic | L3 | too-old | `6ec4476ac825` |  |"));
        let with = Bisection {
            binutils: Some("binutils-2.14".into()),
            ..run("0123456789abcdef")
        };
        assert!(
            report_of(&repo, &[with], &BTreeMap::new())
                .contains("| 5.7..5.8 | gcc-4.8.5 with binutils-2.14 | x86_64 |")
        );
        let counted = report_of(
            &repo,
            &[],
            &BTreeMap::from([("too-old-generic".to_owned(), 3)]),
        );
        assert!(counted.contains("| too-old-generic | L3 | too-old | `6ec4476ac825` |  | 3 |"));
        // A record from before --binutils still reads.
        let old: Bisection = serde_json::from_str(
            r#"{"from":"5.7","to":"5.8","gcc":"gcc-4.8.5","platform":"x86_64","config":"defconfig+gk","rung":"L1","first":"0123","subject":"","contained_in":""}"#,
        )
        .unwrap();
        assert_eq!(old.binutils, None);
    }

    #[test]
    fn a_rerun_that_cleared_a_failure_is_not_counted() {
        let cell = |kernel: &str, started: u64, class: &str| {
            serde_json::from_value::<crate::publish::Entry>(serde_json::json!({
                "cell": format!("{kernel}-{started}"), "kernel": kernel, "gcc": "gcc-4.8.5",
                "binutils": "binutils-2.25.1", "platform": "x86_64", "config": "defconfig+gk",
                "host": "", "rung": "L2", "verdict": "fails", "class": class, "runs": 1,
                "gk": "", "date": "", "started": started, "seconds": 0.0, "machine": ""
            }))
            .unwrap()
        };
        let m = crate::publish::Matrix {
            schema: crate::publish::SCHEMA,
            cells: vec![
                cell("2.6.28", 1, "gcc48-mutex-slowpath-unused"),
                cell("2.6.28", 2, ""),
                cell("2.6.27", 1, "gcc48-mutex-slowpath-unused"),
            ],
        };
        assert_eq!(
            failing(&m),
            BTreeMap::from([("gcc48-mutex-slowpath-unused".to_owned(), 1)])
        );
    }

    #[test]
    fn a_tested_commit_keeps_its_error_and_old_records_still_read() {
        let old: Tested = serde_json::from_str(
            r#"{"commit":"8165984acf82","version":"2.6.37","rung":"L2","term":"misses"}"#,
        )
        .unwrap();
        assert_eq!(old.error, None);
        let text = serde_json::to_string(&old).unwrap();
        assert!(!text.contains("error"));
        let new = Tested {
            error: Some("entry_32.S:398: Error: too many positional arguments".into()),
            ..old
        };
        let back: Tested = serde_json::from_str(&serde_json::to_string(&new).unwrap()).unwrap();
        assert_eq!(back.error, new.error);
    }

    #[test]
    fn releases_are_tagged_with_a_v() {
        assert_eq!(tag("3.0"), "v3.0");
        assert_eq!(tag("linux-4.9.337"), "v4.9.337");
    }
}
