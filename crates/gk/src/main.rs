//! `gk`, the gcc-kernel command line.
//!
//! Only `check`, `pins`, `fetch`, `forge`, `ladder` and `version` work so far. The rest of the commands in `docs/spec/10-gcc-kernel-repo.md` land with the milestones that need them.

mod fetch;
mod forge;
mod gnu;
mod gpg;
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
        Some("forge") => forge(&args[1..]),
        Some("fetch") => fetch(&args[1..]),
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
