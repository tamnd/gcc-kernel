//! `gk pins`: apply `sets.toml` to kernel.org and the GNU mirror, and show or write how the pin files change.
//!
//! Kernel hashes come from kernel.org's `sha256sums.asc`. GCC and binutils hashes are taken by downloading the tarball once into `work/cache`, because the GNU mirror publishes signatures but no hash lists. Without `--write` nothing is downloaded but the listings, and the changes are only printed.

use crate::{gnu, kernelorg, net};
use gk_model::Version;
use gk_model::kernels::{Kernel, Kernels};
use gk_model::repo::Repo;
use gk_model::toolchains::{AllBinutils, Binutils, Gccs};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Run `gk pins`. Returns whether anything changed.
pub fn run(repo: &Repo, write: bool) -> Result<bool, String> {
    let cache = repo.root.join("work/cache");
    let mut lines = Vec::new();

    let kernels = kernels(repo)?;
    describe_kernels(&repo.kernels, &kernels, &mut lines);

    let binutils = binutils(repo, write, &cache, &mut lines)?;
    let gccs = gccs(repo, write, &cache, &mut lines)?;

    for line in &lines {
        println!("{line}");
    }
    if lines.is_empty() {
        println!("the pins are up to date");
    }
    if write {
        save(&repo.root.join("kernels.toml"), KERNELS_HEADER, &kernels)?;
        save(&repo.root.join("binutils.toml"), BINUTILS_HEADER, &binutils)?;
        save(&repo.root.join("gccs.toml"), GCCS_HEADER, &gccs)?;
    }
    Ok(!lines.is_empty())
}

const KERNELS_HEADER: &str = "# Every kernel tree that is a row of the matrix, with its tarball and SHA-256.\n#\n# Written by `gk pins --write` from the rules in sets.toml, kernel.org's releases.json and the signed sha256sums.asc of each directory. Pins in a set that sets.toml has no rule for are kept as they are.\n";
const BINUTILS_HEADER: &str = "# Every binutils release a GCC column can be paired with (spec 04.5).\n#\n# Written by `gk pins --write` from the GNU mirror. The date is the one the mirror's listing shows for the tarball, and the SHA-256 is taken from a download.\n";
const GCCS_HEADER: &str = "# The GCC columns of the matrix (spec 04).\n#\n# Written by hand. `gk pins --write` fills in the url, the release date and the SHA-256 of upstream columns where they are left empty.\n";

/// The kernel pins the rules give, together with the pins no rule covers.
fn kernels(repo: &Repo) -> Result<Kernels, String> {
    let sets = &repo.sets;
    let releases = kernelorg::parse_releases(&net::fetch_text(kernelorg::RELEASES_URL)?)?;
    let mut sums: BTreeMap<u32, BTreeMap<String, String>> = BTreeMap::new();
    let mut sums_of = |major: u32| -> Result<BTreeMap<String, String>, String> {
        if let Some(s) = sums.get(&major) {
            return Ok(s.clone());
        }
        let s = kernelorg::parse_sums(&net::fetch_text(&kernelorg::sums_url(major))?);
        sums.insert(major, s.clone());
        Ok(s)
    };
    let mut out: BTreeMap<Version, Kernel> = BTreeMap::new();
    let mut newest_major = 0;
    for r in &releases {
        let Ok(version) = r.version.parse::<Version>() else {
            continue;
        };
        let major = version.parts()[0];
        newest_major = newest_major.max(major);
        let in_sets = sets.for_moniker(&r.moniker, r.iseol);
        if in_sets.is_empty() {
            continue;
        }
        let name = format!("linux-{}.tar.xz", r.version);
        let sha256 = sums_of(major)?
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("sha256sums.asc of v{major}.x does not list {name}"))?;
        out.insert(
            version.clone(),
            Kernel {
                version,
                url: kernelorg::tarball_url(major, &r.version),
                sha256,
                sets: in_sets,
                moniker: r.moniker.clone(),
            },
        );
    }
    if let Some(first) = sets.first_release() {
        for major in first.parts()[0]..=newest_major {
            for (name, sha256) in sums_of(major)? {
                let Some(v) = name
                    .strip_prefix("linux-")
                    .and_then(|n| n.strip_suffix(".tar.xz"))
                else {
                    continue;
                };
                let Ok(version) = v.parse::<Version>() else {
                    continue;
                };
                if version.parts().len() != 2 || version.is_pre() {
                    continue;
                }
                let in_sets = sets.for_release(&version);
                if in_sets.is_empty() {
                    continue;
                }
                let url = kernelorg::tarball_url(major, v);
                let k = out.entry(version.clone()).or_insert_with(|| Kernel {
                    version,
                    url,
                    sha256,
                    sets: Vec::new(),
                    moniker: String::new(),
                });
                merge(&mut k.sets, &in_sets);
            }
        }
    }
    for old in &repo.kernels.kernels {
        if old.sets.iter().all(|s| sets.has(s)) {
            continue;
        }
        match out.get_mut(&old.version) {
            Some(k) => {
                let kept: Vec<String> = old.sets.iter().filter(|s| !sets.has(s)).cloned().collect();
                merge(&mut k.sets, &kept);
            }
            None => {
                out.insert(old.version.clone(), old.clone());
            }
        }
    }
    Ok(Kernels {
        kernels: out.into_values().collect(),
    })
}

fn merge(sets: &mut Vec<String>, more: &[String]) {
    for s in more {
        if !sets.contains(s) {
            sets.push(s.clone());
        }
    }
}

fn describe_kernels(old: &Kernels, new: &Kernels, lines: &mut Vec<String>) {
    for k in &new.kernels {
        match old.get(&k.version) {
            None => lines.push(format!("+ linux {} ({})", k.version, k.sets.join(", "))),
            Some(o) if o != k => lines.push(format!("~ linux {} changed", k.version)),
            Some(_) => {}
        }
    }
    for k in &old.kernels {
        if new.get(&k.version).is_none() {
            lines.push(format!("- linux {}", k.version));
        }
    }
}

/// Every binutils release from `binutils-from` on, keeping what is already pinned.
fn binutils(
    repo: &Repo,
    write: bool,
    cache: &Path,
    lines: &mut Vec<String>,
) -> Result<AllBinutils, String> {
    let Some(from) = &repo.sets.gnu.binutils_from else {
        return Ok(repo.binutils.clone());
    };
    let listing = gnu::parse_listing(&net::fetch_text(&format!("{}/", gnu::BINUTILS_URL))?);
    let mut out: BTreeMap<Version, Binutils> = repo
        .binutils
        .releases
        .iter()
        .map(|b| (b.version.clone(), b.clone()))
        .collect();
    for (v, date) in gnu::binutils_releases(&listing) {
        let version: Version = v.parse().map_err(|e| format!("binutils {v}: {e:?}"))?;
        if version < *from || out.contains_key(&version) {
            continue;
        }
        let url = gnu::binutils_url(&v);
        let sha256 = if write {
            hash_of(&url, cache)?
        } else {
            String::new()
        };
        lines.push(format!("+ binutils {v} ({date})"));
        out.insert(
            version.clone(),
            Binutils {
                id: format!("binutils-{v}"),
                version,
                released: date,
                url,
                sha256,
            },
        );
    }
    Ok(AllBinutils {
        releases: out.into_values().collect(),
    })
}

/// The GCC columns with empty fields filled in, and a note for every series whose last point on the mirror is not pinned.
fn gccs(repo: &Repo, write: bool, cache: &Path, lines: &mut Vec<String>) -> Result<Gccs, String> {
    let listing = gnu::parse_listing(&net::fetch_text(&format!("{}/", gnu::GCC_URL))?);
    let dates: BTreeMap<Version, String> = gnu::gcc_releases(&listing)
        .into_iter()
        .filter_map(|(v, d)| Some((v.parse().ok()?, d)))
        .collect();
    let mut out = repo.gccs.clone();
    for g in out.gccs.iter_mut().filter(|g| g.flavor == "upstream") {
        let v = g.version.as_str().to_owned();
        if g.url.is_empty() {
            g.url = gnu::gcc_url(&v);
            lines.push(format!("~ {} url", g.id));
        }
        if g.released.is_empty() {
            g.released = dates
                .get(&g.version)
                .cloned()
                .ok_or_else(|| format!("{} is not on the GNU mirror", g.id))?;
            lines.push(format!("~ {} released {}", g.id, g.released));
        }
        if g.sha256.is_empty() {
            lines.push(format!("~ {} sha256", g.id));
            if write {
                g.sha256 = hash_of(&g.url, cache)?;
            }
        }
    }
    let oldest = repo
        .gccs
        .gccs
        .iter()
        .filter(|g| g.flavor == "upstream")
        .map(|g| g.version.parts()[0])
        .min();
    let mut last: BTreeMap<u32, &Version> = BTreeMap::new();
    for v in dates.keys() {
        last.insert(v.parts()[0], v);
    }
    for (major, v) in last {
        if oldest.is_some_and(|o| major >= o) && !repo.gccs.gccs.iter().any(|g| g.version == *v) {
            lines.push(format!(
                "? gcc {v} is the last point of GCC {major} on the mirror and is not a column"
            ));
        }
    }
    Ok(out)
}

/// Download a tarball into the cache, unless it is there, and return its SHA-256.
fn hash_of(url: &str, cache: &Path) -> Result<String, String> {
    let name = url.rsplit('/').next().unwrap_or("download");
    let path: PathBuf = cache.join(name);
    if !path.is_file() {
        eprintln!("downloading {url}");
        net::download(url, &path)?;
    }
    net::sha256_file(&path)
}

/// Write a pin file, keeping the comment block at the top of the old one.
fn save<T: Serialize>(path: &Path, header: &str, value: &T) -> Result<(), String> {
    let old = std::fs::read_to_string(path).unwrap_or_default();
    let kept: String = old
        .lines()
        .take_while(|l| l.starts_with('#') || l.is_empty())
        .fold(String::new(), |mut s, l| {
            s.push_str(l);
            s.push('\n');
            s
        });
    let head = if kept.trim().is_empty() {
        header.to_owned()
    } else {
        kept.trim_end().to_owned() + "\n"
    };
    let body = toml::to_string(value).map_err(|e| format!("{}: {e}", path.display()))?;
    std::fs::write(path, format!("{head}\n{body}"))
        .map_err(|e| format!("writing {}: {e}", path.display()))
}
