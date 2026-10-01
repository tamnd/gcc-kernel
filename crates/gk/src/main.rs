//! `gk`, the gcc-kernel command line.
//!
//! Only `check`, `pins`, `fetch`, `forge`, `hosts check`, `probe`, `cell`, `ladder` and `version` work so far. The rest of the commands in `docs/spec/10-gcc-kernel-repo.md` land with the milestones that need them.

mod build;
mod cell;
mod fetch;
mod forge;
mod gnu;
mod gpg;
mod hosts;
mod kconfig;
mod kernelorg;
mod net;
mod pins;

use gk_model::repo::Repo;
use gk_model::{Rung, Verdict};
use std::process::ExitCode;

const USAGE: &str = "usage: gk <command>

commands:
  check      read every pin file and check that they agree with each other
  pins       apply sets.toml to kernel.org and the GNU mirror and print how the pins change,
             or write them with --write
  fetch      download and check pinned tarballs into the cache:
             --kernel K, --gcc G, --binutils B (each can repeat), or --all
  forge      build a static toolchain bundle: forge G [--target T] [--jobs N]
  forge verify [G...]
             check every bundle in the cache, in every host
  hosts check
             run every host and forge container and compare its tools with hosts.toml
  probe      the accept probe: probe K G --platform P
  cell       run one cell up to L4 and write its directory:
             cell K G --platform P [--jobs N] [--keep] [--ungraded]
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
        Some("fetch") => fetch(&args[1..]),
        Some("hosts") if args.get(1).map(String::as_str) == Some("check") => hosts_check(),
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

/// `gk probe` and `gk cell`.
fn cell(command: &str, args: &[String]) -> ExitCode {
    let mut positional = Vec::new();
    let mut platform = None;
    let mut jobs = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);
    let (mut keep, mut ungraded) = (false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--platform" | "--jobs" => {
                let Some(v) = it.next() else {
                    eprintln!("gk {command}: {a} needs a value");
                    return ExitCode::from(2);
                };
                if a == "--platform" {
                    platform = Some(v.clone());
                } else if let Ok(n) = v.parse() {
                    jobs = n;
                } else {
                    eprintln!("gk {command}: --jobs takes a number");
                    return ExitCode::from(2);
                }
            }
            "--keep" => keep = true,
            "--ungraded" => ungraded = true,
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
        let setup = cell::Setup::new(&repo, kernel, gcc, &platform)?;
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
