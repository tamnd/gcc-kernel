//! `gk`, the gcc-kernel command line.
//!
//! Only `check`, `ladder` and `version` work so far. The rest of the commands in `docs/spec/10-gcc-kernel-repo.md` land with the milestones that need them.

use gk_model::repo::Repo;
use gk_model::{Rung, Verdict};
use std::process::ExitCode;

const USAGE: &str = "usage: gk <command>

commands:
  check      read every pin file and check that they agree with each other
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
