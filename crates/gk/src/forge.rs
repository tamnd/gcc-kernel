//! `gk forge` and `gk forge verify`: static toolchain bundles, built in a forge container (spec 04.4).
//!
//! `gk forge` fetches and checks the GCC and binutils tarballs, then GMP, MPFR, MPC and ISL at the versions the GCC release names in `contrib/download_prerequisites`. Those are checked against the release's own `contrib/prerequisites.sha512`, which the GCC signature covers. It then runs `provision/forge/build.sh` in the forge container with no network, and writes the bundle and a manifest to `bundles/` in the cache. A bundle's digest is the SHA-256 of its tarball, and that digest is what a cell's identity names.
//!
//! `gk forge verify` unpacks each bundle and runs `provision/forge/verify.sh` over it in every host that has a Dockerfile, which checks that it is static, reports the pinned version, and compiles kernel style code in each of them.

use crate::fetch::{self, Request};
use crate::net;
use gk_model::repo::Repo;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Where GCC's prerequisites live.
pub const INFRASTRUCTURE: &str = "https://gcc.gnu.org/pub/gcc/infrastructure";

/// What `gk forge` writes next to each bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// The GCC column, as in `gcc-16.2.0`.
    pub id: String,
    /// The GCC version.
    pub gcc: String,
    /// The binutils version it is paired with.
    pub binutils: String,
    /// The GNU triple.
    pub target: String,
    /// The forge container.
    pub forge: String,
    /// The image it ran in, by digest when the image is published and by local tag otherwise.
    pub image: String,
    /// The prerequisite tarballs built in tree.
    pub prerequisites: Vec<String>,
    /// The bundle tarball's file name in the cache.
    pub file: String,
    /// `sha256:` and the tarball's SHA-256.
    pub digest: String,
    /// How long the build took, in seconds.
    pub seconds: u64,
    /// The gk version that built it.
    pub gk: String,
}

/// The bundle directory of the cache.
#[must_use]
pub fn bundles_dir() -> PathBuf {
    fetch::cache_dir().join("bundles")
}

/// The manifest of a bundle that `gk forge` has built.
pub fn manifest(gcc_id: &str, target: &str) -> Result<Manifest, String> {
    let path = bundles_dir().join(format!("{gcc_id}-{target}.json"));
    let text = std::fs::read_to_string(&path).map_err(|_| {
        format!("no bundle for {gcc_id} on {target}; run gk forge {gcc_id} --target {target}")
    })?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// A bundle unpacked into the cache, unpacking it the first time.
pub fn unpacked(m: &Manifest) -> Result<PathBuf, String> {
    let dir = bundles_dir();
    let unpacked = dir.join("unpacked").join(format!("{}-{}", m.id, m.target));
    if unpacked.is_dir() {
        return Ok(unpacked);
    }
    let tarball = dir.join(&m.file);
    let partial = unpacked.with_extension("part");
    let _ = std::fs::remove_dir_all(&partial);
    std::fs::create_dir_all(&partial)
        .map_err(|e| format!("creating {}: {e}", partial.display()))?;
    let status = Command::new("tar")
        .arg("--zstd")
        .arg("-xf")
        .arg(&tarball)
        .arg("-C")
        .arg(&partial)
        .status()
        .map_err(|e| format!("running tar: {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&partial);
        return Err(format!("unpacking {} failed", tarball.display()));
    }
    std::fs::rename(&partial, &unpacked)
        .map_err(|e| format!("renaming {}: {e}", partial.display()))?;
    Ok(unpacked)
}

/// Run `gk forge`.
#[allow(clippy::too_many_lines)]
pub fn run(repo: &Repo, gcc_id: &str, targets: &[String], jobs: Option<u32>) -> Result<(), String> {
    let gcc = repo
        .gccs
        .gccs
        .iter()
        .find(|g| g.id == gcc_id || g.version.as_str() == gcc_id)
        .ok_or_else(|| format!("gcc {gcc_id} is not in gccs.toml"))?;
    let binutils = repo
        .binutils
        .pair(gcc, None)
        .ok_or_else(|| format!("{} has no binutils pairing", gcc.id))?;
    let targets: Vec<String> = if targets.is_empty() {
        gcc.targets.clone()
    } else {
        for t in targets {
            if !gcc.targets.contains(t) {
                return Err(format!("{} is not built for {t}", gcc.id));
            }
        }
        targets.to_vec()
    };
    fetch::run(
        repo,
        &Request {
            gccs: vec![gcc.id.clone()],
            binutils: vec![binutils.id.clone()],
            ..Request::default()
        },
    )?;
    let src = fetch::cache_dir().join("toolchains");
    let gcc_file = file_of(&gcc.url, &format!("gcc-{}.tar.xz", gcc.version));
    let binutils_file = file_of(
        &binutils.url,
        &format!("binutils-{}.tar.xz", binutils.version),
    );
    let gcc_tar = src.join(&gcc_file);
    let prereqs = prerequisites(&gcc_tar, &format!("gcc-{}", gcc.version), &src)?;
    let image = image_for(repo, &gcc.forge)?;
    let out = bundles_dir();
    std::fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;
    let script = repo.root.join("provision/forge/build.sh");
    for target in &targets {
        println!(
            "{}: building for {target} with binutils {} in {}",
            gcc.id, binutils.version, gcc.forge
        );
        let started = Instant::now();
        let mut cmd = Command::new("docker");
        cmd.args(["run", "--rm", "--network", "none"])
            .arg("-v")
            .arg(format!("{}:/src:ro", src.display()))
            .arg("-v")
            .arg(format!("{}:/out", out.display()))
            .arg("-v")
            .arg(format!("{}:/gk/build.sh:ro", script.display()))
            .args(["-e", &format!("GK_ID={}", gcc.id)])
            .args(["-e", &format!("GK_GCC={}", gcc.version)])
            .args(["-e", &format!("GK_BINUTILS={}", binutils.version)])
            .args(["-e", &format!("GK_GCC_TAR={gcc_file}")])
            .args(["-e", &format!("GK_BINUTILS_TAR={binutils_file}")])
            .args(["-e", &format!("GK_TARGET={target}")])
            .args(["-e", &format!("GK_PREREQS={}", prereqs.join(" "))]);
        if let Some(j) = jobs {
            cmd.args(["-e", &format!("GK_JOBS={j}")]);
        }
        let status = cmd
            .arg(&image)
            .args(["bash", "/gk/build.sh"])
            .status()
            .map_err(|e| format!("running docker: {e}"))?;
        if !status.success() {
            return Err(format!("{} for {target}: the forge build failed", gcc.id));
        }
        let file = format!("{}-{target}.tar.zst", gcc.id);
        let plain = out.join(format!("{}-{target}.tar", gcc.id));
        if plain.is_file() {
            // A forge without zstd leaves the tarball uncompressed.
            let status = Command::new("zstd")
                .args(["-q", "-f", "-19", "-T0", "--rm"])
                .arg(&plain)
                .status()
                .map_err(|e| format!("running zstd: {e}"))?;
            if !status.success() {
                return Err(format!("compressing {} failed", plain.display()));
            }
        }
        let digest = format!("sha256:{}", net::sha256_file(&out.join(&file))?);
        let manifest = Manifest {
            id: gcc.id.clone(),
            gcc: gcc.version.as_str().to_owned(),
            binutils: binutils.version.as_str().to_owned(),
            target: target.clone(),
            forge: gcc.forge.clone(),
            image: image.clone(),
            prerequisites: prereqs.clone(),
            file: file.clone(),
            digest: digest.clone(),
            seconds: started.elapsed().as_secs(),
            gk: env!("CARGO_PKG_VERSION").to_owned(),
        };
        let json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
        let path = out.join(format!("{}-{target}.json", gcc.id));
        std::fs::write(&path, json + "\n")
            .map_err(|e| format!("writing {}: {e}", path.display()))?;
        println!("{}: {target} {digest} in {}s", gcc.id, manifest.seconds);
    }
    Ok(())
}

/// The image of a forge or host: the published one when `hosts.toml` pins it, and otherwise one built here from its Dockerfile under a local tag.
pub(crate) fn image_for(repo: &Repo, name: &str) -> Result<String, String> {
    let host = repo
        .hosts
        .get(name)
        .ok_or_else(|| format!("{name} is not in hosts.toml"))?;
    if host.is_built() {
        return Ok(host.image.clone());
    }
    let dockerfile = repo.root.join(host.dockerfile());
    let dir = dockerfile
        .parent()
        .ok_or_else(|| format!("{} has no directory", dockerfile.display()))?;
    if !dockerfile.is_file() {
        return Err(format!("{name} has no image and no {}", host.dockerfile()));
    }
    let tag = format!("gk/{name}:dev");
    let status = Command::new("docker")
        .args(["build", "-q", "-t", &tag])
        .arg(dir)
        .stdout(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("running docker: {e}"))?;
    if !status.success() {
        return Err(format!("building {tag} failed"));
    }
    Ok(tag)
}

/// The file name at the end of a URL, or a fallback when the URL is empty.
fn file_of(url: &str, fallback: &str) -> String {
    match url.rsplit('/').next() {
        Some(name) if !name.is_empty() => name.to_owned(),
        _ => fallback.to_owned(),
    }
}

/// The prerequisites of releases older than `contrib/prerequisites.sha512`, with their SHA-512 as taken from gcc.gnu.org's infrastructure directory when they were first used.
const OLD_PREREQUISITES: &[(&str, &str)] = &[
    ("gmp-4.3.2.tar.bz2", GMP_4_3_2),
    ("mpfr-2.4.2.tar.bz2", MPFR_2_4_2),
    ("mpc-0.8.1.tar.gz", MPC_0_8_1),
];

const GMP_4_3_2: &str = "2e0b0fd23e6f10742a5517981e5171c6e88b0a93c83da701b296f5c0861d72c19782daab589a7eac3f9032152a0fc7eff7f5362db8fccc4859564a9aa82329cf";
const MPFR_2_4_2: &str = "c004b3dbf86c04960e4a1f8db37a409a7cc4cb76135e76e98dcc5ad93aaa8deb62334ee13ff84447a7c12a5e8cb57f25c62ac908c24920f1fb1a38d79d4a4c5e";
const MPC_0_8_1: &str = "14cb9ae3d33caed24d5ae648eed28b2e00ad047a8baeff25981129af88245b4def2948573d7a00d65c5bd34e53524aa6a7351b76703c9f888b41830c1a1daae2";

/// Fetch the prerequisites a GCC release names and check them against its `prerequisites.sha512`. Returns their file names.
///
/// Releases before 7 have no `prerequisites.sha512`, and their script names each library in upper case without its suffix, as in `MPFR=mpfr-2.4.2`. Their GMP, MPFR and MPC are checked against `OLD_PREREQUISITES`. Their ISL and `CLooG` are left out: they only enable the Graphite loop passes, which no kernel asks for.
fn prerequisites(gcc_tar: &Path, top: &str, src: &Path) -> Result<Vec<String>, String> {
    let script = tar_member(gcc_tar, &format!("{top}/contrib/download_prerequisites"))?;
    let sums = tar_member(gcc_tar, &format!("{top}/contrib/prerequisites.sha512")).ok();
    let mut out = Vec::new();
    let names: &[&str] = if sums.is_some() {
        &["gmp", "mpfr", "mpc", "isl"]
    } else {
        &["GMP", "MPFR", "MPC"]
    };
    for name in names {
        let (file, want) = if let Some(sums) = &sums {
            let file = assigned(&script, name)
                .ok_or_else(|| format!("{top}: download_prerequisites names no {name}"))?;
            let want = sums
                .lines()
                .find_map(|l| {
                    let (h, f) = l.split_once("  ")?;
                    (f.trim() == file).then(|| h.to_owned())
                })
                .ok_or_else(|| format!("{top}: prerequisites.sha512 does not list {file}"))?;
            (file, want)
        } else {
            let file = old_file(&script, name)
                .ok_or_else(|| format!("{top}: download_prerequisites names no {name}"))?;
            let want = OLD_PREREQUISITES
                .iter()
                .find(|(f, _)| *f == file)
                .map(|(_, h)| (*h).to_owned())
                .ok_or_else(|| format!("{top}: {file} has no pinned hash in forge.rs"))?;
            (file, want)
        };
        let path = src.join("infrastructure").join(&file);
        if !path.is_file() || net::sha512_file(&path)? != want {
            net::download(&format!("{INFRASTRUCTURE}/{file}"), &path)?;
            if net::sha512_file(&path)? != want {
                let _ = std::fs::remove_file(&path);
                return Err(format!(
                    "{file} does not match {top}'s prerequisites.sha512"
                ));
            }
        }
        out.push(file);
    }
    Ok(out)
}

/// The tarball an old `download_prerequisites` fetches for a library: the value of `NAME=` and the suffix of the line that downloads `$NAME`.
fn old_file(script: &str, name: &str) -> Option<String> {
    let base = assigned(script, name)?;
    let var = format!("${name}.tar.");
    let suffix = script.lines().find_map(|l| {
        let at = l.find(&var)? + var.len();
        let rest = &l[at..];
        let end = rest
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(rest.len());
        Some(rest[..end].to_owned())
    })?;
    Some(format!("{base}.tar.{suffix}"))
}

/// The value of a `name='value'` line of a shell script.
fn assigned(script: &str, name: &str) -> Option<String> {
    script.lines().find_map(|l| {
        let v = l.trim().strip_prefix(name)?.strip_prefix('=')?;
        Some(v.trim_matches(|c| c == '\'' || c == '"').to_owned())
    })
}

fn tar_member(tar: &Path, member: &str) -> Result<String, String> {
    let out = Command::new("tar")
        .arg("-xOf")
        .arg(tar)
        .arg(member)
        .output()
        .map_err(|e| format!("running tar: {e}"))?;
    if !out.status.success() {
        return Err(format!("{} has no {member}", tar.display()));
    }
    String::from_utf8(out.stdout).map_err(|_| format!("{member} is not text"))
}

/// Run `gk forge verify` over every bundle in the cache, or the columns named. Returns whether every check passed.
pub fn verify(repo: &Repo, only: &[String]) -> Result<bool, String> {
    let dir = bundles_dir();
    let mut manifests = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("reading {}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|e| e == "json") {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("reading {}: {e}", path.display()))?;
            let m: Manifest =
                serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            if only.is_empty() || only.iter().any(|o| *o == m.id || *o == m.gcc) {
                manifests.push(m);
            }
        }
    }
    manifests.sort_by(|a, b| (&a.id, &a.target).cmp(&(&b.id, &b.target)));
    if manifests.is_empty() {
        return Err("no bundles to verify".to_owned());
    }
    let hosts: Vec<&str> = repo
        .hosts
        .hosts
        .iter()
        .filter(|h| h.kind == "host" && (h.is_built() || repo.root.join(h.dockerfile()).is_file()))
        .map(|h| h.name.as_str())
        .collect();
    let script = repo.root.join("provision/forge/verify.sh");
    let mut all_ok = true;
    for m in &manifests {
        let tarball = dir.join(&m.file);
        let got = format!("sha256:{}", net::sha256_file(&tarball)?);
        if got != m.digest {
            println!(
                "{} {}: FAIL the tarball is {got}, the manifest says {}",
                m.id, m.target, m.digest
            );
            all_ok = false;
            continue;
        }
        let unpacked = unpacked(m)?;
        for host in &hosts {
            let image = image_for(repo, host)?;
            let out = Command::new("docker")
                .args(["run", "--rm", "--network", "none"])
                .arg("-v")
                .arg(format!("{}:/bundle:ro", unpacked.display()))
                .arg("-v")
                .arg(format!("{}:/gk/verify.sh:ro", script.display()))
                .args(["-e", &format!("GK_TARGET={}", m.target)])
                .args(["-e", &format!("GK_GCC={}", m.gcc)])
                .arg(&image)
                .args(["bash", "/gk/verify.sh"])
                .output()
                .map_err(|e| format!("running docker: {e}"))?;
            let text = String::from_utf8_lossy(&out.stdout);
            let ok = out.status.success();
            all_ok &= ok;
            println!(
                "{} {} in {host}: {}",
                m.id,
                m.target,
                if ok { "ok" } else { "FAIL" }
            );
            if !ok {
                for line in text
                    .lines()
                    .chain(String::from_utf8_lossy(&out.stderr).lines())
                {
                    println!("  {line}");
                }
            }
        }
    }
    Ok(all_ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prerequisite_names_read_from_the_script() {
        let script = "gmp='gmp-6.3.0.tar.bz2'\nmpfr='mpfr-4.2.2.tar.bz2'\nmpc='mpc-1.3.1.tar.gz'\nisl='isl-0.24.tar.bz2'\n";
        assert_eq!(assigned(script, "mpc").unwrap(), "mpc-1.3.1.tar.gz");
        assert_eq!(assigned(script, "mpfr").unwrap(), "mpfr-4.2.2.tar.bz2");
        assert!(assigned(script, "cloog").is_none());
    }

    #[test]
    fn old_scripts_name_the_tarball_on_the_download_line() {
        let script = "MPFR=mpfr-2.4.2\nGMP=gmp-4.3.2\nMPC=mpc-0.8.1\n\nwget ftp://gcc.gnu.org/pub/gcc/infrastructure/$MPFR.tar.bz2 || exit 1\nwget ftp://gcc.gnu.org/pub/gcc/infrastructure/$MPC.tar.gz || exit 1\n";
        assert_eq!(old_file(script, "MPFR").unwrap(), "mpfr-2.4.2.tar.bz2");
        assert_eq!(old_file(script, "MPC").unwrap(), "mpc-0.8.1.tar.gz");
        assert!(old_file(script, "GMP").is_none());
        assert!(OLD_PREREQUISITES.iter().all(|(_, h)| h.len() == 128));
    }

    #[test]
    fn bundle_sources_are_named_by_their_urls() {
        assert_eq!(
            file_of(
                "https://ftp.gnu.org/gnu/gcc/gcc-4.9.4/gcc-4.9.4.tar.bz2",
                "x"
            ),
            "gcc-4.9.4.tar.bz2"
        );
        assert_eq!(file_of("", "gcc-9.5.0.tar.xz"), "gcc-9.5.0.tar.xz");
    }
}
