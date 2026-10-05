//! `gk config-diff`: what the kernel's configuration notices about a GCC (spec 11.3).
//!
//! Kconfig asks the compiler what it can do, and from 4.18 it records the answers as `CC_HAS_*` symbols, with `GCC_VERSION` and `CC_VERSION_TEXT` next to them. Two columns that build the same kernel with the same configuration target differ exactly where a probe answered differently or a symbol depends on the version. The cells' `config` files are compared, and with no columns named every pair of neighbouring columns in the store is, which is the list a persona has to reproduce.

use crate::{kconfig, store};
use gk_model::Version;
use gk_model::repo::Repo;
use std::path::{Path, PathBuf};

/// A cell's `.config`, which cells from before 0.2 kept as `config`.
pub fn config_of(dir: &Path) -> Option<PathBuf> {
    [".config", "config"]
        .iter()
        .map(|name| dir.join(name))
        .find(|p| p.is_file())
}

/// The newest cell for each GCC column on one kernel, platform and configuration, oldest GCC first.
fn columns(
    repo: &Repo,
    kernel: &Version,
    platform: &str,
    config: &str,
) -> Result<Vec<(String, PathBuf)>, String> {
    let mut newest: Vec<(String, PathBuf, u64)> = Vec::new();
    for (dir, r) in store::cells()? {
        let c = &r.coordinates;
        let k = c
            .kernel
            .name
            .trim_start_matches("linux-")
            .parse::<Version>();
        if c.platform != platform
            || c.config.name != config
            || k.as_ref() != Ok(kernel)
            || crate::sweep::swept(repo, c)
        {
            continue;
        }
        if config_of(&dir).is_none() {
            continue;
        }
        match newest.iter_mut().find(|(g, _, _)| *g == c.gcc.name) {
            Some(n) if n.2 >= r.started => {}
            Some(n) => *n = (c.gcc.name.clone(), dir, r.started),
            None => newest.push((c.gcc.name.clone(), dir, r.started)),
        }
    }
    let version = |id: &str| {
        repo.gccs
            .gccs
            .iter()
            .find(|g| g.id == id)
            .map(|g| g.version.clone())
    };
    newest.sort_by_key(|(g, _, _)| version(g));
    Ok(newest.into_iter().map(|(g, d, _)| (g, d)).collect())
}

/// The differences between two cells' configurations: a count and a table, or a line saying there are none.
pub fn differences(from: &(String, PathBuf), to: &(String, PathBuf)) -> Result<String, String> {
    let load = |dir: &Path| {
        config_of(dir)
            .ok_or_else(|| format!("{} has no .config", dir.display()))
            .and_then(|p| kconfig::load(&p))
    };
    let (a, b) = (load(&from.1)?, load(&to.1)?);
    let d = kconfig::diff(&a, &b);
    if d.is_empty() {
        return Ok("No differences.\n".into());
    }
    Ok(format!(
        "{} symbols differ.\n\n{}",
        d.len(),
        kconfig::table(&d, &from.0, &to.0)
    ))
}

/// The differences between two cells' configurations, as a section of the report.
fn section(from: &(String, PathBuf), to: &(String, PathBuf)) -> Result<String, String> {
    Ok(format!(
        "\n## {} to {}\n\n{}",
        from.0,
        to.0,
        differences(from, to)?
    ))
}

/// `gk config-diff K [G1 G2] --platform P [--config C]`.
pub fn command(repo: &Repo, args: &[String]) -> Result<String, String> {
    let usage = "usage: gk config-diff K [G1 G2] --platform P [--config C]";
    let (mut words, mut platform, mut config) = (Vec::new(), None, crate::cell::CONFIG.to_owned());
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--platform" | "--config" => {
                let v = it.next().ok_or_else(|| format!("{a} needs a value"))?;
                if a == "--platform" {
                    platform = Some(v.clone());
                } else {
                    config.clone_from(v);
                }
            }
            other if !other.starts_with('-') => words.push(other.to_owned()),
            other => return Err(format!("gk config-diff: unexpected {other:?}")),
        }
    }
    let Some(platform) = platform else {
        return Err(usage.into());
    };
    let (kernel, named) = match words.as_slice() {
        [k] => (k, None),
        [k, g1, g2] => (k, Some((g1.clone(), g2.clone()))),
        _ => return Err(usage.into()),
    };
    let version: Version = kernel
        .trim_start_matches("linux-")
        .parse()
        .map_err(|e| format!("kernel {kernel}: {e}"))?;
    let found = columns(repo, &version, &platform, &config)?;
    let mut out = format!("# {version} on {platform}, {config}\n");
    let pick = |g: &str| {
        found
            .iter()
            .find(|(id, _)| id == g || id.strip_prefix("gcc-") == Some(g))
            .ok_or_else(|| format!("no cell for {version} with {g} on {platform} in the store"))
    };
    let pairs: Vec<_> = if let Some((g1, g2)) = &named {
        vec![(pick(g1)?, pick(g2)?)]
    } else {
        found.windows(2).map(|w| (&w[0], &w[1])).collect()
    };
    if pairs.is_empty() {
        return Err(format!(
            "{version} on {platform} has {} cells with a config in the store, and a differential needs two",
            found.len()
        ));
    }
    for (from, to) in pairs {
        out.push_str(&section(from, to)?);
    }
    Ok(out)
}
