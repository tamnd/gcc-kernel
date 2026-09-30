//! The pin files of a checkout, read together and checked against each other.
//!
//! `gk check` runs [`Repo::check`] on every pull request, so a pin that names an era, a host, a platform or a binutils that does not exist never reaches main.

use crate::eras::Eras;
use crate::hosts::Hosts;
use crate::kernels::Kernels;
use crate::platforms::Platforms;
use crate::sets::Sets;
use crate::toolchains::{AllBinutils, Gccs};
use crate::{is_sha256, step_for};
use serde::de::DeserializeOwned;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Every pin file of a checkout.
#[derive(Debug, Clone, Default)]
pub struct Repo {
    /// The checkout.
    pub root: PathBuf,
    /// `kernels.toml`.
    pub kernels: Kernels,
    /// `gccs.toml`.
    pub gccs: Gccs,
    /// `binutils.toml`.
    pub binutils: AllBinutils,
    /// `eras.toml`.
    pub eras: Eras,
    /// `hosts.toml`.
    pub hosts: Hosts,
    /// `platforms.toml`.
    pub platforms: Platforms,
    /// `sets.toml`.
    pub sets: Sets,
}

fn read<T: DeserializeOwned + Default>(root: &Path, name: &str) -> Result<T, String> {
    let path = root.join(name);
    match std::fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).map_err(|e| format!("{name}: {e}")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(format!("reading {}: {e}", path.display())),
    }
}

impl Repo {
    /// Read every pin file under `root`. A missing file reads as empty, and [`Repo::check`] says so where it matters.
    pub fn load(root: &Path) -> Result<Self, String> {
        Ok(Repo {
            root: root.to_path_buf(),
            kernels: read(root, "kernels.toml")?,
            gccs: read(root, "gccs.toml")?,
            binutils: read(root, "binutils.toml")?,
            eras: read(root, "eras.toml")?,
            hosts: read(root, "hosts.toml")?,
            platforms: read(root, "platforms.toml")?,
            sets: read(root, "sets.toml")?,
        })
    }

    /// Find the checkout from the current directory: the nearest directory above it with an `eras.toml`.
    pub fn find() -> Result<Self, String> {
        let here = std::env::current_dir().map_err(|e| format!("current directory: {e}"))?;
        let root = here
            .ancestors()
            .find(|d| d.join("eras.toml").is_file())
            .ok_or("not inside a gcc-kernel checkout: no eras.toml here or above")?;
        Self::load(root)
    }

    /// Every problem with the pin files, one line each. Empty means they agree.
    #[must_use]
    pub fn check(&self) -> Vec<String> {
        let mut problems = Vec::new();
        self.check_kernels(&mut problems);
        self.check_toolchains(&mut problems);
        self.check_eras(&mut problems);
        self.check_platforms(&mut problems);
        problems
    }

    fn check_kernels(&self, problems: &mut Vec<String>) {
        let mut seen = BTreeSet::new();
        for k in &self.kernels.kernels {
            if !seen.insert(k.version.clone()) {
                problems.push(format!("kernels.toml: {} is pinned twice", k.version));
            }
            if !is_sha256(&k.sha256) {
                problems.push(format!("kernels.toml: {} has no SHA-256", k.version));
            }
            if !k.url.starts_with("https://") || !k.file_name().starts_with("linux-") {
                problems.push(format!(
                    "kernels.toml: {} has an odd url {}",
                    k.version, k.url
                ));
            }
            if k.sets.is_empty() {
                problems.push(format!("kernels.toml: {} is in no set", k.version));
            }
            if self.eras.of(&k.version).is_none() {
                problems.push(format!("kernels.toml: {} is in no era", k.version));
            }
        }
        let versions: Vec<_> = self.kernels.kernels.iter().map(|k| &k.version).collect();
        if versions.windows(2).any(|w| w[0] > w[1]) {
            problems.push("kernels.toml: kernels are not oldest first".into());
        }
    }

    fn check_toolchains(&self, problems: &mut Vec<String>) {
        let mut ids = BTreeSet::new();
        for g in &self.gccs.gccs {
            if !ids.insert(&g.id) {
                problems.push(format!("gccs.toml: {} is listed twice", g.id));
            }
            if g.flavor == "upstream" {
                if !is_sha256(&g.sha256) || !g.url.starts_with("https://") {
                    problems.push(format!("gccs.toml: {} needs a url and a SHA-256", g.id));
                }
                if self.hosts.get(&g.forge).is_none_or(|h| h.kind != "forge") {
                    problems.push(format!(
                        "gccs.toml: {} names forge {:?}, which hosts.toml does not have",
                        g.id, g.forge
                    ));
                }
            }
            if crate::toolchains::add_days(&g.released, 0).is_none() {
                problems.push(format!("gccs.toml: {} has no release date", g.id));
            }
            if g.binutils != "auto" && self.binutils.get(&g.binutils).is_none() {
                problems.push(format!(
                    "gccs.toml: {} is paired with {}, which binutils.toml does not have",
                    g.id, g.binutils
                ));
            }
            if g.flavor == "upstream" && self.binutils.pair(g, None).is_none() {
                problems.push(format!(
                    "gccs.toml: {} has no binutils released before it",
                    g.id
                ));
            }
            for t in &g.targets {
                if !self.platforms.platforms.iter().any(|p| p.triple == *t) {
                    problems.push(format!(
                        "gccs.toml: {} targets {t}, which no platform uses",
                        g.id
                    ));
                }
            }
        }
        let mut ids = BTreeSet::new();
        for b in &self.binutils.releases {
            if !ids.insert(&b.id) {
                problems.push(format!("binutils.toml: {} is listed twice", b.id));
            }
            if !is_sha256(&b.sha256) || !b.url.starts_with("https://") {
                problems.push(format!("binutils.toml: {} needs a url and a SHA-256", b.id));
            }
        }
    }

    fn check_eras(&self, problems: &mut Vec<String>) {
        let froms: Vec<_> = self.eras.eras.iter().map(|e| &e.from).collect();
        if froms.windows(2).any(|w| w[0] >= w[1]) {
            problems.push("eras.toml: eras are not in order".into());
        }
        for era in &self.eras.eras {
            if era.hosts.is_empty() {
                problems.push(format!("eras.toml: {} has no host", era.name));
            }
            if era.hosts.first().is_some_and(|h| h.from != era.from) {
                problems.push(format!(
                    "eras.toml: {}'s first host starts at {}, not at the era's start {}",
                    era.name, era.hosts[0].from, era.from
                ));
            }
            for h in &era.hosts {
                if self.hosts.get(&h.value).is_none_or(|h| h.kind != "host") {
                    problems.push(format!(
                        "eras.toml: {} names host {}, which hosts.toml does not have",
                        era.name, h.value
                    ));
                }
            }
        }
        for k in &self.kernels.kernels {
            if let Some(era) = self.eras.of(&k.version)
                && step_for(&era.hosts, &k.version).is_none()
            {
                problems.push(format!("eras.toml: no host for {}", k.version));
            }
        }
    }

    fn check_platforms(&self, problems: &mut Vec<String>) {
        let mut names = BTreeSet::new();
        for p in &self.platforms.platforms {
            if !names.insert(&p.name) {
                problems.push(format!("platforms.toml: {} is listed twice", p.name));
            }
            if !(1..=4).contains(&p.tier) {
                problems.push(format!("platforms.toml: {} has tier {}", p.name, p.tier));
            }
            for (what, steps) in [
                ("arch", &p.arch),
                ("defconfig", &p.defconfig),
                ("image", &p.image),
                ("machine", &p.machine),
                ("cpu", &p.cpu),
                ("console", &p.console),
            ] {
                if step_for(steps, &p.first_kernel).is_none() {
                    problems.push(format!(
                        "platforms.toml: {} has no {what} for its first kernel {}",
                        p.name, p.first_kernel
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pin files committed at the top of the repository must pass their own check.
    #[test]
    fn the_committed_pins_agree() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let repo = Repo::load(&root).unwrap();
        let problems = repo.check();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
