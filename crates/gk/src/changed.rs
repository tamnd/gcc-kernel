//! `gk pins changed`: the accept probes a pull request owes for the pins it adds or changes.
//!
//! A kernel pin is probed with its era GCC, which is the column it has to accept, or with the pinned column of the era GCC's release series when the era names a point release that is not pinned, as M14's `gcc-14.2.0` is next to the column `gcc-14.4.0`, or a distribution build such as `debian-bullseye-gcc-10` is next to `gcc-10.5.0`. A GCC pin is probed against the newest kernel of the `current` set. Both run on `x86_64`, the one platform every column targets. A probe is left out, with the reason on stderr, when the GCC it would use is not in `gccs.toml` or does not target `x86_64`, which is the case for the eras before G0's columns.

use gk_model::Version;
use gk_model::repo::Repo;
use std::process::Command;

const TARGET: &str = "x86_64-linux-gnu";
const FILES: [&str; 7] = [
    "kernels.toml",
    "gccs.toml",
    "binutils.toml",
    "eras.toml",
    "hosts.toml",
    "platforms.toml",
    "sets.toml",
];

/// The pin files as they were at `base`, a git revision.
fn at(repo: &Repo, base: &str) -> Result<Repo, String> {
    let dir = std::env::temp_dir().join(format!("gk-base-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let result = (|| {
        for file in FILES {
            let out = Command::new("git")
                .arg("-C")
                .arg(&repo.root)
                .args(["show", &format!("{base}:{file}")])
                .output()
                .map_err(|e| format!("running git: {e}"))?;
            // A file the base does not have reads as empty, so everything in it counts as new.
            if out.status.success() {
                std::fs::write(dir.join(file), &out.stdout)
                    .map_err(|e| format!("writing {file}: {e}"))?;
            }
        }
        Repo::load(&dir)
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// The probes, as kernel and GCC column, that the pins changed since `base` call for.
pub fn probes(repo: &Repo, base: &str) -> Result<Vec<(Version, String)>, String> {
    plan(repo, &at(repo, base)?)
}

fn plan(repo: &Repo, old: &Repo) -> Result<Vec<(Version, String)>, String> {
    let usable = |id: &str| {
        repo.gccs
            .get(id)
            .is_some_and(|g| g.targets.iter().any(|t| t == TARGET))
    };
    let mut out = Vec::new();
    for k in &repo.kernels.kernels {
        if old.kernels.get(&k.version) == Some(k) {
            continue;
        }
        let Some(era) = repo.eras.of(&k.version) else {
            return Err(format!("{} has no era in eras.toml", k.version));
        };
        if let Some(column) = crate::search::era_column(repo, &era.gcc, TARGET) {
            out.push((k.version.clone(), column));
        } else {
            eprintln!(
                "{}: its era GCC {} has no {TARGET} bundle in G0, so it is not probed",
                k.version, era.gcc
            );
        }
    }
    let newest = repo
        .kernels
        .in_set("current")
        .into_iter()
        .map(|k| &k.version)
        .max()
        .ok_or("no kernel is in the current set")?;
    for g in &repo.gccs.gccs {
        if old.gccs.get(&g.id) == Some(g) {
            continue;
        }
        if !usable(&g.id) {
            eprintln!("{}: it does not target {TARGET}, so it is not probed", g.id);
        } else if !out.iter().any(|(k, id)| k == newest && *id == g.id) {
            out.push((newest.clone(), g.id.clone()));
        }
    }
    Ok(out)
}

/// The incremental runs of spec 09.7 that the pins changed since `base` call for, as `gk` command lines.
pub fn sweep(repo: &Repo, base: &str) -> Result<Vec<String>, String> {
    Ok(sweep_plan(repo, &at(repo, base)?))
}

/// A new kernel is searched on every tier 1 platform it is graded on. A new GCC column is run on the current stripe, every kernel of the `current` set on every tier 1 platform it targets, with both configurations of the stripe. Its search against every other row is the weekly sweep's job, not this one's.
fn sweep_plan(repo: &Repo, old: &Repo) -> Vec<String> {
    let tier1: Vec<_> = repo
        .platforms
        .platforms
        .iter()
        .filter(|p| p.tier == 1)
        .collect();
    let mut out = Vec::new();
    for k in &repo.kernels.kernels {
        if old.kernels.get(&k.version) == Some(k) {
            continue;
        }
        for p in tier1.iter().filter(|p| p.first_kernel <= k.version) {
            out.push(format!("search {} --platform {}", k.version, p.name));
        }
    }
    let current = repo.kernels.in_set("current");
    for g in &repo.gccs.gccs {
        if old.gccs.get(&g.id) == Some(g) || g.flavor != "upstream" {
            continue;
        }
        for p in tier1.iter().filter(|p| g.targets.contains(&p.triple)) {
            for k in current.iter().filter(|k| p.applies(&k.version, &g.version)) {
                for config in ["defconfig+gk", "tinyconfig+gk"] {
                    out.push(format!(
                        "cell {} {} --platform {} --config {config}",
                        k.version, g.id, p.name
                    ));
                }
            }
        }
    }
    out
}

/// The bundles the probes need, as GCC columns, without repeats.
#[must_use]
pub fn bundles(probes: &[(Version, String)]) -> Vec<&str> {
    let mut ids: Vec<&str> = probes.iter().map(|(_, g)| g.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn repo() -> Repo {
        Repo::load(Path::new("../..")).unwrap()
    }

    #[test]
    fn nothing_changed_needs_no_probe() {
        let r = repo();
        assert!(plan(&r, &r).unwrap().is_empty());
    }

    #[test]
    fn a_new_kernel_is_probed_with_its_era_gcc_and_a_new_gcc_against_current() {
        let r = repo();
        let mut old = r.clone();
        let k = old
            .kernels
            .kernels
            .iter()
            .position(|k| k.version.to_string() == "7.2.8")
            .unwrap();
        old.kernels.kernels.remove(k);
        let g = old
            .gccs
            .gccs
            .iter()
            .position(|g| g.id == "gcc-15.3.0")
            .unwrap();
        old.gccs.gccs.remove(g);
        let probes = plan(&r, &old).unwrap();
        assert!(probes.contains(&("7.2.8".parse().unwrap(), "gcc-14.2.0".to_string())));
        assert!(probes.iter().any(|(_, g)| g == "gcc-15.3.0"));
        assert!(bundles(&probes).len() <= probes.len());
        let runs = sweep_plan(&r, &old);
        for p in ["x86_64", "i386", "arm64"] {
            assert!(
                runs.contains(&format!("search 7.2.8 --platform {p}")),
                "{runs:?}"
            );
            assert!(runs.contains(&format!(
                "cell 7.2.8 gcc-15.3.0 --platform {p} --config tinyconfig+gk"
            )));
        }
        assert!(sweep_plan(&r, &r).is_empty());
    }
}
