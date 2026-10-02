//! `gk`, the gcc-kernel command line.
//!
//! Only `check`, `pins`, `fetch`, `forge`, `hosts check`, `pins changed`, `init`, `boot`, `probe`, `cell`, `search --dense`, `store`, `publish`, `ladder` and `version` work so far. The rest of the commands in `docs/spec/10-gcc-kernel-repo.md` land with the milestones that need them.

mod boot;
mod build;
mod cell;
mod changed;
mod differential;
mod fetch;
mod forge;
mod gnu;
mod gpg;
mod hosts;
mod initramfs;
mod kconfig;
mod kernelorg;
mod net;
mod pins;
mod publish;
mod report;
mod search;
mod store;
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
             one \"K G\" line each, and with --bundles the GCC columns they need
  fetch      download and check pinned tarballs into the cache:
             --kernel K, --gcc G, --binutils B (each can repeat), or --all
  forge      build a static toolchain bundle: forge G [--target T] [--jobs N]
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
             cell K G --platform P [--config C] [--jobs N] [--keep] [--ungraded] [--no-boot],
             where C is defconfig+gk (the default), tinyconfig+gk or allnoconfig+gk
  search     run every cell of one row: search K --platform P --dense [--config C] [--jobs N] [--rerun]
             [--keep] [--ungraded] [--no-boot]. Cells already in the store are not run again.
  publish    write matrix/matrix.json from the graded cells in the store [--ungraded]
  config-diff
             the configuration differences between GCC columns on one kernel (spec 11.3):
             config-diff K [G1 G2] --platform P [--config C], every pair of neighbouring
             columns in the store when no GCCs are named
  report     report new-gcc G [--config C] [--ungraded] [--stdout]: write reports/new-gcc-<version>.md,
             the release report of spec 10.8 over the Current set
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
        Some("search") => search_command(&args[1..]),
        Some("publish") => publish_command(&args[1..]),
        Some("config-diff") => config_diff(&args[1..]),
        Some("report") => report_command(&args[1..]),
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

/// `gk pins changed BASE [--bundles]`.
fn changed(args: &[String]) -> ExitCode {
    let bundles = args.iter().any(|a| a == "--bundles");
    let Some(base) = args.iter().find(|a| !a.starts_with('-')) else {
        eprintln!("gk pins changed: name the git revision to compare against");
        return ExitCode::from(2);
    };
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

/// `gk search K --platform P --dense`.
fn search_command(args: &[String]) -> ExitCode {
    let mut opts = search::Options {
        dense: false,
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
                    eprintln!("gk search: {e}");
                    return ExitCode::from(2);
                }
                None => {
                    eprintln!("gk search: --config needs a value");
                    return ExitCode::from(2);
                }
            },
            "--jobs" => {
                let Some(n) = it.next().and_then(|v| v.parse().ok()) else {
                    eprintln!("gk search: --jobs takes a number");
                    return ExitCode::from(2);
                };
                opts.jobs = n;
            }
            "--dense" => opts.dense = true,
            "--rerun" => opts.rerun = true,
            "--keep" => opts.keep = true,
            "--no-boot" => opts.boot = false,
            "--ungraded" => ungraded = true,
            other if !other.starts_with('-') && kernel.is_none() => kernel = Some(other.to_owned()),
            other => {
                eprintln!("gk search: unknown argument {other:?}");
                return ExitCode::from(2);
            }
        }
    }
    let (Some(kernel), Some(platform)) = (kernel, platform) else {
        eprintln!("gk search: as in gk search 7.2.8 --platform x86_64 --dense");
        return ExitCode::from(2);
    };
    let result = Repo::find().and_then(|repo| {
        if !ungraded && !cell::gk_commit(&repo.root).1 {
            return Err(
                "the checkout has uncommitted changes; commit them or pass --ungraded".into(),
            );
        }
        let row = search::run(&repo, &kernel, &platform, opts)?;
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

fn publish_command(args: &[String]) -> ExitCode {
    let ungraded = args.iter().any(|a| a == "--ungraded");
    match Repo::find().and_then(|repo| publish::write(&repo, ungraded)) {
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
    let mut config = cell::CONFIG.to_owned();
    let mut jobs = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);
    let (mut keep, mut ungraded, mut no_boot) = (false, false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--platform" | "--jobs" | "--config" => {
                let Some(v) = it.next() else {
                    eprintln!("gk {command}: {a} needs a value");
                    return ExitCode::from(2);
                };
                if a == "--platform" {
                    platform = Some(v.clone());
                } else if a == "--config" {
                    config.clone_from(v);
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
        let setup = cell::Setup::new(&repo, kernel, gcc, &platform, &config)?;
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
