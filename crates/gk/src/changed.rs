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

/// A GCC's release series: the major version from GCC 5 on, major and minor before.
fn series(v: &Version) -> Vec<u32> {
    v.series(if v.series(1)[0] >= 5 { 1 } else { 2 })
}

/// The column that stands for the era GCC `id`: the column itself when it is pinned for `x86_64`,
/// or else the newest pinned `x86_64` column of the same release series.
fn column_for(repo: &Repo, id: &str) -> Option<String> {
    let usable = |g: &&gk_model::toolchains::Gcc| g.targets.iter().any(|t| t == TARGET);
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
        if let Some(column) = column_for(repo, &era.gcc) {
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
    }
}
