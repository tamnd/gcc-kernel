//! `gk`, the gcc-kernel command line.
//!
//! Only `check`, `pins`, `fetch`, `forge`, `hosts check`, `pins changed`, `init`, `boot`, `probe`, `cell`, `search`, `store`, `publish`, `report`, `config-diff`, `gates`, `flags-diff`, `classify`, `triage`, `explain`, `ladder`, `history`, `bisect-kernel` and `version` work so far. The rest of the commands in `docs/spec/10-gcc-kernel-repo.md` land with the milestones that need them.

mod bisect;
mod boot;
mod build;
mod cell;
mod census;
mod changed;
mod classify;
mod differential;
mod distro;
mod ext2;
mod fetch;
mod forge;
mod gnu;
mod gpg;
mod history;
mod hosts;
mod html;
mod initramfs;
mod kconfig;
mod kernelorg;
mod minix;
mod net;
mod pins;
mod publish;
mod report;
mod search;
mod status;
mod store;
mod surface;
mod sweep;
mod tap;

use gk_model::repo::Repo;
use gk_model::{Rung, Verdict};
use std::process::ExitCode;

const USAGE: &str = "usage: gk <command>

commands:
  check      read every pin file and check that they agree with each other
  pins       apply sets.toml to kernel.org and the GNU mirror and print how the pins change,
             or write them with --write
  pins changed BASE
             print the accept probes that the pins changed since the git revision BASE call for,
             one \"K G\" line each, and with --bundles the GCC columns they need,
             or with --sweep the incremental runs of spec 09.7 as gk command lines
  fetch      download and check pinned tarballs into the cache:
             --kernel K, --gcc G, --binutils B (each can repeat), or --all
  forge      build a static toolchain bundle: forge G [--target T] [--jobs N], or binutils alone: forge B --target T
  forge verify [G...]
             check every bundle in the cache, in every host
  hosts check
             run every host and forge container and compare its tools with hosts.toml
  init       build the init program and initramfs that booted cells use: init [P...],
             every platform with an init pin when none is named
  boot       boot a kernel image in the gk-boot container and run a gk-init suite:
             boot IMAGE --platform P --kernel K [--suite S] [--dir D], writing boot.log and boot.json
  probe      the accept probe: probe K G --platform P
  cell       run one cell and write its directory, booting it to L6 where gk-init covers the kernel:
             cell K G --platform P [--config C] [--binutils B] [--jobs N] [--keep] [--ungraded] [--no-boot],
             where C is defconfig+gk (the default), tinyconfig+gk, allnoconfig+gk or allmodconfig,
             which is built and never booted
  binutils-sweep
             sweep the binutils of one row with its era GCC: binutils-sweep K --platform P [--config C] [--jobs N] [--rerun]
  search     search one row for the edges of its working set, or run every cell with --dense: search K --platform P [--dense] [--seed N] [--config C] [--jobs N] [--rerun]
             [--keep] [--ungraded] [--no-boot]. Cells already in the store are not run again.
  publish    write matrix/matrix.json, the heat maps, warning census and config differential in reports/
             and the status section of README.md
             from the graded cells in the store [--ungraded], and with --html [DIR] the static site
             into DIR, which is site when not given. --site-only makes the site from the committed
             matrix/matrix.json and writes nothing else
  config-diff
             the configuration differences between GCC columns on one kernel (spec 11.3):
             config-diff K [G1 G2] --platform P [--config C], every pair of neighbouring
             columns in the store when no GCCs are named
  gates      gates K: every place the kernel's tree tests the GCC version, with the release it flips at
             and the first GCC column that sees it (spec 11.3)
  flags-diff flags-diff K G1 G2 --platform P [--config C]: the flags kbuild passes to the units with one
             GCC and not the other, from the cells' compile.jsonl
  report     report new-gcc G [--config C] [--ungraded] [--stdout]: write reports/new-gcc-<version>.md,
             the release report of spec 10.8 over the Current set, or report persona-surface
             [--ungraded] [--stdout]: write reports/persona-surface.md, the gates, Kconfig symbols and
             flags that change at each GCC column (spec 11.3)
  classify   run signatures.toml over every failed cell in the store and write the class into
             its cell.json, or only print them with --dry-run. Writing also grades again the KUnit
             runs of cells that had no era cell to grade against when they ran
  triage     the failed cells no signature names, clustered by first error, largest first: triage [--all]
  explain    explain K G P [--config C]: the cell's first error, its class, and whether the fix
             is in the kernel's tree
  history    history update: clone Linus's tree into the history clone, or bring it up to date,
             with the tags of the stable tree
  bisect-kernel
             bisect-kernel K1 K2 G --platform P [--config C] [--binutils B] [--rung R] [--jobs N]: the first commit
             between two releases where the cell of G changes whether it reaches rung R, which is
             the higher rung of the two ends when R is not given. With --unit OBJECT, such as
             drivers/gpu/drm/i915/i915_gem.o, each commit builds that object alone instead
             of the kernel. With --binutils B, each cell uses that binutils over the GCC bundle's own.
  store      list the cells in the result store, or:
             store show ID, store check, store pack FILE.tar.zst
  ladder     print the outcome ladder and the verdict each rung earns
  version    print the gk version
  help       print this text

The full command set is planned in docs/spec/10-gcc-kernel-repo.md.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("ladder") => {
            for r in Rung::ALL {
                let v = Verdict::of(Some(r), Rung::Clean);
                println!("{r}  {:<10}  {}", r.name(), v.letter());
            }
            ExitCode::SUCCESS
        }
        Some("check") => check(),
        Some(c @ ("probe" | "cell")) => cell(c, &args[1..]),
        Some("forge") => forge(&args[1..]),
        Some("init") => init(&args[1..]),
        Some("boot") => boot_command(&args[1..]),
        Some("store") => store_command(&args[1..]),
        Some(c @ ("search" | "binutils-sweep")) => search_command(c, &args[1..]),
        Some("publish") => publish_command(&args[1..]),
        Some("config-diff") => config_diff(&args[1..]),
        Some("report") => report_command(&args[1..]),
        Some("gates") => {
            let rest = &args[1..];
            text_command(|repo| surface::gates_command(repo, rest))
        }
        Some("flags-diff") => {
            let rest = &args[1..];
            text_command(|_| surface::flags_command(rest))
        }
        Some("classify") => classify_command(&args[1..]),
        Some("triage") => {
            let all = args[1..].iter().any(|a| a == "--all");
            text_command(|repo| classify::triage(repo, all))
        }
        Some("explain") => {
            let rest = &args[1..];
            text_command(|repo| classify::explain(repo, rest))
        }
        Some("history") if args.get(1).map(String::as_str) == Some("update") => {
            text_command(|_| bisect::update().map(|()| String::new()))
        }
        Some("bisect-kernel") => bisect_command(&args[1..]),
        Some("fetch") => fetch(&args[1..]),
        Some("hosts") if args.get(1).map(String::as_str) == Some("check") => hosts_check(),
        Some("pins") if args.get(1).map(String::as_str) == Some("changed") => changed(&args[2..]),
        Some("pins") => pins(args.iter().any(|a| a == "--write")),
        Some("version" | "--version" | "-V") => {
            println!("gk {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("help" | "--help" | "-h") | None => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("gk: unknown command {other:?}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// `gk init [P...]`.
fn init(args: &[String]) -> ExitCode {
    match Repo::find().and_then(|repo| initramfs::run(&repo, args)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `gk boot`: exits 0 when the suite passed.
fn boot_command(args: &[String]) -> ExitCode {
    match Repo::find().and_then(|repo| boot::command(&repo, args)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::from(2)
        }
    }
}

/// `gk pins`: exits 0 when nothing changed, 1 when something did, so `watch.yml` can tell.
fn pins(write: bool) -> ExitCode {
    let result = Repo::find().and_then(|repo| pins::run(&repo, write));
    match result {
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) if write => ExitCode::SUCCESS,
        Ok(true) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::from(2)
        }
    }
}

/// `gk pins changed BASE [--bundles | --sweep]`.
fn changed(args: &[String]) -> ExitCode {
    let bundles = args.iter().any(|a| a == "--bundles");
    let Some(base) = args.iter().find(|a| !a.starts_with('-')) else {
        eprintln!("gk pins changed: name the git revision to compare against");
        return ExitCode::from(2);
    };
    if args.iter().any(|a| a == "--sweep") {
        return text_command(|repo| {
            changed::sweep(repo, base).map(|runs| runs.iter().map(|r| r.clone() + "\n").collect())
        });
    }
    match Repo::find().and_then(|repo| changed::probes(&repo, base)) {
        Ok(probes) if bundles => {
            for id in changed::bundles(&probes) {
                println!("{id}");
            }
            ExitCode::SUCCESS
        }
        Ok(probes) => {
            for (kernel, gcc) in &probes {
                println!("{kernel} {gcc}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::from(2)
        }
    }
}

/// `gk bisect-kernel K1 K2 G --platform P`.
fn bisect_command(args: &[String]) -> ExitCode {
    let mut opts = bisect::Options {
        config: "defconfig+gk",
        rung: None,
        jobs: std::thread::available_parallelism().map_or(8, std::num::NonZero::get),
        unit: None,
        binutils: None,
    };
    let mut platform = None;
    let mut names = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let value = |it: &mut std::slice::Iter<String>| it.next().cloned().unwrap_or_default();
        match a.as_str() {
            "--platform" => platform = Some(value(&mut it)),
            "--rung" => opts.rung = Some(value(&mut it).to_uppercase()),
            "--unit" => opts.unit = Some(value(&mut it)),
            "--binutils" => opts.binutils = Some(value(&mut it)),
            "--config" => match cell::config_named(&value(&mut it)) {
                Ok(c) => opts.config = c.0,
                Err(e) => {
                    eprintln!("gk bisect-kernel: {e}");
                    return ExitCode::from(2);
                }
            },
            "--jobs" => {
                let Ok(n) = value(&mut it).parse() else {
                    eprintln!("gk bisect-kernel: --jobs takes a number");
                    return ExitCode::from(2);
                };
                opts.jobs = n;
            }
            _ => names.push(a.clone()),
        }
    }
    let (Some(platform), [from, to, gcc]) = (platform, names.as_slice()) else {
        eprintln!(
            "usage: gk bisect-kernel K1 K2 G --platform P [--config C] [--binutils B] [--rung R | --unit OBJECT] [--jobs N]"
        );
        return ExitCode::from(2);
    };
    match Repo::find().and_then(|repo| bisect::run(&repo, from, to, gcc, &platform, &opts)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `gk search K --platform P [--dense]`.
fn search_command(command: &str, args: &[String]) -> ExitCode {
    let mut opts = search::Options {
        dense: false,
        seed: None,
        jobs: std::thread::available_parallelism().map_or(8, std::num::NonZero::get),
        rerun: false,
        keep: false,
        boot: true,
        config: cell::CONFIG,
    };
    let (mut kernel, mut platform, mut ungraded) = (None, None, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--platform" => platform = it.next().cloned(),
            "--config" => match it.next().map(|c| cell::config_named(c)) {
                Some(Ok(c)) => opts.config = c.0,
                Some(Err(e)) => {
                    eprintln!("gk {command}: {e}");
                    return ExitCode::from(2);
                }
                None => {
                    eprintln!("gk {command}: --config needs a value");
                    return ExitCode::from(2);
                }
            },
            "--jobs" => {
                let Some(n) = it.next().and_then(|v| v.parse().ok()) else {
                    eprintln!("gk {command}: --jobs takes a number");
                    return ExitCode::from(2);
                };
                opts.jobs = n;
            }
            "--dense" => opts.dense = true,
            "--seed" => {
                let Some(n) = it.next().and_then(|v| v.parse().ok()) else {
                    eprintln!("gk {command}: --seed takes a number");
                    return ExitCode::from(2);
                };
                opts.seed = Some(n);
            }
            "--rerun" => opts.rerun = true,
            "--keep" => opts.keep = true,
            "--no-boot" => opts.boot = false,
            "--ungraded" => ungraded = true,
            other if !other.starts_with('-') && kernel.is_none() => kernel = Some(other.to_owned()),
            other => {
                eprintln!("gk {command}: unknown argument {other:?}");
                return ExitCode::from(2);
            }
        }
    }
    let (Some(kernel), Some(platform)) = (kernel, platform) else {
        eprintln!("gk {command}: as in gk {command} 7.2.8 --platform x86_64");
        return ExitCode::from(2);
    };
    let result = Repo::find().and_then(|repo| {
        if !ungraded && !cell::gk_commit(&repo.root).1 {
            return Err(
                "the checkout has uncommitted changes; commit them or pass --ungraded".into(),
            );
        }
        let row = if command == "binutils-sweep" {
            sweep::run(&repo, &kernel, &platform, opts)?
        } else {
            search::run(&repo, &kernel, &platform, opts)?
        };
        let broken = row
            .iter()
            .filter(|(_, c)| matches!(c, search::Column::Broken(_)))
            .count();
        println!("{} columns, {broken} not run", row.len());
        Ok(broken == 0)
    });
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::from(2)
        }
    }
}

/// `gk config-diff`.
fn config_diff(args: &[String]) -> ExitCode {
    match Repo::find().and_then(|repo| differential::command(&repo, args)) {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::from(2)
        }
    }
}

/// `gk publish`.
fn report_command(args: &[String]) -> ExitCode {
    match Repo::find().and_then(|repo| report::command(&repo, args)) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::from(2)
        }
    }
}

/// `gk classify [--dry-run]`.
fn classify_command(args: &[String]) -> ExitCode {
    let write = !args.iter().any(|a| a == "--dry-run");
    text_command(|repo| classify::classify(repo, write))
}

/// A command that prints what it returns.
fn text_command(run: impl FnOnce(&Repo) -> Result<String, String>) -> ExitCode {
    match Repo::find().and_then(|repo| run(&repo)) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::from(2)
        }
    }
}

/// `gk publish [--ungraded] [--html [DIR]] [--site-only]`. With `--site-only` the site is made from the committed `matrix/matrix.json` and nothing else is written.
fn publish_command(args: &[String]) -> ExitCode {
    let ungraded = args.iter().any(|a| a == "--ungraded");
    let html = args.iter().position(|a| a == "--html").map(|i| {
        args.get(i + 1)
            .filter(|a| !a.starts_with('-'))
            .map_or("site", String::as_str)
            .to_owned()
    });
    let site_only = args.iter().any(|a| a == "--site-only");
    let result = Repo::find().and_then(|repo| {
        let (n, mut reports) = if site_only {
            (0, Vec::new())
        } else {
            publish::write(&repo, ungraded)?
        };
        if let Some(dir) = &html {
            let m = if site_only {
                let path = repo.root.join("matrix").join("matrix.json");
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("reading {}: {e}", path.display()))?;
                serde_json::from_str(&text)
                    .map_err(|e| format!("reading {}: {e}", path.display()))?
            } else {
                publish::matrix(&repo, ungraded)?
            };
            reports.extend(html::write(
                &repo,
                &m,
                std::path::Path::new(dir),
                &publish::today(),
            )?);
        }
        Ok((n, reports))
    });
    match result {
        Ok((n, reports)) => {
            println!("{n} cells in matrix/matrix.json");
            for r in reports {
                println!("{r}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::from(2)
        }
    }
}

/// `gk store`: list, show, check and pack the result store.
fn store_command(args: &[String]) -> ExitCode {
    let result = match (args.first().map(String::as_str), args.get(1)) {
        (None | Some("list"), _) => store::cells().map(|cells| {
            for (_, r) in &cells {
                let c = &r.coordinates;
                println!(
                    "{}  {:<14} {:<12} {:<9} {:<3} {:<7} {:>6.0}s{}",
                    &r.cell["sha256:".len().."sha256:".len() + 16],
                    c.kernel.name,
                    c.gcc.name,
                    c.platform,
                    r.rung,
                    r.verdict,
                    r.seconds,
                    if r.graded { "" } else { "  ungraded" }
                );
            }
            eprintln!("{} cells in {}", cells.len(), store::root().display());
            true
        }),
        (Some("show"), Some(id)) => store::find(id).map(|(dir, r)| {
            println!("{}", dir.display());
            println!("{}", serde_json::to_string_pretty(&r).unwrap_or_default());
            true
        }),
        (Some("check"), _) => store::check().map(|problems| {
            for p in &problems {
                println!("{p}");
            }
            problems.is_empty()
        }),
        (Some("pack"), Some(out)) => store::pack(std::path::Path::new(out)).map(|n| {
            println!("{n} cells packed into {out}");
            true
        }),
        _ => Err("usage: gk store [list | show ID | check | pack FILE.tar.zst]".into()),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::from(2)
        }
    }
}

/// `gk probe` and `gk cell`.
fn cell(command: &str, args: &[String]) -> ExitCode {
    let mut positional = Vec::new();
    let mut platform = None;
    let mut binutils: Option<String> = None;
    let mut config = cell::CONFIG.to_owned();
    let mut jobs = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);
    let (mut keep, mut ungraded, mut no_boot) = (false, false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--platform" | "--jobs" | "--config" | "--binutils" => {
                let Some(v) = it.next() else {
                    eprintln!("gk {command}: {a} needs a value");
                    return ExitCode::from(2);
                };
                if a == "--platform" {
                    platform = Some(v.clone());
                } else if a == "--config" {
                    config.clone_from(v);
                } else if a == "--binutils" {
                    binutils = Some(v.clone());
                } else if let Ok(n) = v.parse() {
                    jobs = n;
                } else {
                    eprintln!("gk {command}: --jobs takes a number");
                    return ExitCode::from(2);
                }
            }
            "--keep" => keep = true,
            "--ungraded" => ungraded = true,
            "--no-boot" => no_boot = true,
            other if !other.starts_with('-') => positional.push(other.to_owned()),
            other => {
                eprintln!("gk {command}: unknown argument {other:?}");
                return ExitCode::from(2);
            }
        }
    }
    let ([kernel, gcc], Some(platform)) = (positional.as_slice(), platform) else {
        eprintln!("gk {command}: as in gk {command} 7.2.8 gcc-16.2.0 --platform x86_64");
        return ExitCode::from(2);
    };
    let result = Repo::find().and_then(|repo| {
        let mut setup = cell::Setup::new(&repo, kernel, gcc, &platform, &config)?;
        if let Some(b) = &binutils {
            setup = setup.with_binutils(&repo, b)?;
        }
        if command == "probe" {
            let dir = fetch::cache_dir()
                .join("scratch")
                .join(format!("probe-{}", setup.coordinates.short_id()));
            let probe = cell::probe(&setup, &dir, jobs)?;
            println!(
                "{kernel} {gcc} {platform}: {} at {} in {:.0}s",
                probe.result, probe.step, probe.seconds
            );
            for line in &probe.why {
                println!("  {line}");
            }
            return Ok(probe.passes());
        }
        if !ungraded && !cell::gk_commit(&repo.root).1 {
            return Err(
                "the checkout has uncommitted changes; commit them or pass --ungraded".into(),
            );
        }
        let setup = if no_boot {
            setup
        } else {
            setup.booting(&repo)?
        };
        let (dir, record) = cell::run(&repo, &setup, jobs, keep)?;
        println!(
            "{kernel} {gcc} {platform}: {} at {} in {:.0}s, {}",
            record.verdict,
            record.rung,
            record.seconds,
            dir.display()
        );
        Ok(true)
    });
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `gk hosts check`.
fn hosts_check() -> ExitCode {
    match Repo::find().and_then(|repo| hosts::check(&repo)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `gk forge` and `gk forge verify`.
fn forge(args: &[String]) -> ExitCode {
    let repo = match Repo::find() {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("gk: {e}");
            return ExitCode::FAILURE;
        }
    };
    if args.first().map(String::as_str) == Some("verify") {
        return match forge::verify(&repo, &args[1..]) {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::FAILURE,
            Err(e) => {
                eprintln!("gk: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let mut gcc = None;
    let mut targets = Vec::new();
    let mut jobs = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--target" | "--jobs" => {
                let Some(v) = it.next() else {
                    eprintln!("gk forge: {a} needs a value");
                    return ExitCode::from(2);
                };
                if a == "--target" {
                    targets.push(v.clone());
                } else if let Ok(n) = v.parse() {
                    jobs = Some(n);
                } else {
                    eprintln!("gk forge: --jobs takes a number");
                    return ExitCode::from(2);
                }
            }
            other if gcc.is_none() && !other.starts_with('-') => gcc = Some(other.to_owned()),
            other => {
                eprintln!("gk forge: unknown argument {other:?}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(gcc) = gcc else {
        eprintln!("gk forge: which GCC? as in gk forge gcc-16.2.0");
        return ExitCode::from(2);
    };
    match forge::run(&repo, &gcc, &targets, jobs) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `gk fetch`.
fn fetch(args: &[String]) -> ExitCode {
    let mut req = fetch::Request::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let list = match a.as_str() {
            "--all" => {
                req.all = true;
                continue;
            }
            "--kernel" => &mut req.kernels,
            "--gcc" => &mut req.gccs,
            "--binutils" => &mut req.binutils,
            other => {
                eprintln!("gk fetch: unknown argument {other:?}");
                return ExitCode::from(2);
            }
        };
        let Some(value) = it.next() else {
            eprintln!("gk fetch: {a} needs a value");
            return ExitCode::from(2);
        };
        list.push(value.clone());
    }
    match Repo::find().and_then(|repo| fetch::run(&repo, &req)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gk: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `gk check`: every pin file read, cross-checked, and counted.
fn check() -> ExitCode {
    let repo = match Repo::find() {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("gk: {e}");
            return ExitCode::FAILURE;
        }
    };
    let problems = repo.check();
    for p in &problems {
        println!("{p}");
    }
    println!(
        "{} kernels, {} gccs, {} binutils, {} eras, {} hosts, {} platforms: {}",
        repo.kernels.kernels.len(),
        repo.gccs.gccs.len(),
        repo.binutils.releases.len(),
        repo.eras.eras.len(),
        repo.hosts.hosts.len(),
        repo.platforms.platforms.len(),
        if problems.is_empty() {
            "they agree".to_owned()
        } else {
            format!("{} problems", problems.len())
        }
    );
    if problems.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
