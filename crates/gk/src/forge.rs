//! `gk forge` and `gk forge verify`: static toolchain bundles, built in a forge container (spec 04.4).
//!
//! `gk forge` fetches and checks the GCC and binutils tarballs, then GMP, MPFR, MPC and ISL at the versions the GCC release names in `contrib/download_prerequisites`. Those are checked against the release's own `contrib/prerequisites.sha512`, which the GCC signature covers. It then runs `provision/forge/build.sh` in the forge container with no network, and writes the bundle and a manifest to `bundles/` in the cache. A bundle's digest is the SHA-256 of its tarball, and that digest is what a cell's identity names.
//!
//! `gk forge verify` unpacks each bundle and runs `provision/forge/verify.sh` over it in every host that has a Dockerfile, which checks that it is static, reports the pinned version, and compiles kernel style code in each of them.

use crate::fetch::{self, Request};
use crate::net;
use gk_model::Version;
use gk_model::repo::Repo;
use gk_model::toolchains::{Binutils, Gcc};
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
    /// The GNU triple its tools are named for, which is the platform's unless the column says otherwise.
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
    /// For a distribution column, the package and its version, as `gcc-6 6.3.0-18+deb9u1`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub package: String,
    /// For a distribution column, the first line of the driver's `--version`, read in its image, since the bundle's links do not resolve outside it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub driver: String,
    /// For a distribution column, the SHA-256 of the driver the links lead to.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub driver_sha256: String,
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

/// A bundle unpacked into the cache, unpacking it the first time. The directory is named for the tarball's digest as well, so a bundle forged again is not served from the tree of the one before.
pub fn unpacked(m: &Manifest) -> Result<PathBuf, String> {
    let dir = bundles_dir();
    let short = m
        .digest
        .trim_start_matches("sha256:")
        .get(..12)
        .unwrap_or_default();
    let unpacked = dir
        .join("unpacked")
        .join(format!("{}-{}-{short}", m.id, m.target));
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

/// Run `gk forge`. A binutils id in place of a GCC one forges binutils alone, for the binutils sweep (spec 04.6).
pub fn run(repo: &Repo, gcc_id: &str, targets: &[String], jobs: Option<u32>) -> Result<(), String> {
    if let Some(b) = repo.binutils.get(gcc_id) {
        return binutils_only(repo, b, targets, jobs);
    }
    let gcc = repo
        .gccs
        .gccs
        .iter()
        .find(|g| g.id == gcc_id || g.version.as_str() == gcc_id)
        .ok_or_else(|| format!("gcc {gcc_id} is not in gccs.toml or binutils.toml"))?;
    if gcc.flavor != "upstream" {
        return distribution(repo, gcc);
    }
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
    let gcc_tar = src.join(&gcc_file);
    let prerequisites = prerequisites(
        &gcc_tar,
        &format!("gcc-{}", gcc.version),
        &gcc.version,
        &src,
    )?;
    let build = Build {
        id: gcc.id.clone(),
        gcc: gcc.version.as_str().to_owned(),
        gcc_file,
        binutils: binutils.version.as_str().to_owned(),
        binutils_file: file_of(
            &binutils.url,
            &format!("binutils-{}.tar.xz", binutils.version),
        ),
        forge: gcc.forge.clone(),
        image: image_for(repo, &gcc.forge)?,
        prerequisites,
    };
    for target in &targets {
        let tools = if gcc.tools.is_empty() {
            target
        } else {
            &gcc.tools
        };
        build.run(repo, target, tools, jobs)?;
    }
    Ok(())
}

/// Forge a binutils release alone, in the forge of the newest upstream GCC released on or before it, which is a forge that already builds binutils of that age. A cell lays it over a GCC bundle with [`combined`].
fn binutils_only(
    repo: &Repo,
    b: &Binutils,
    targets: &[String],
    jobs: Option<u32>,
) -> Result<(), String> {
    if targets.is_empty() {
        return Err(format!("name the targets of {} with --target", b.id));
    }
    let gcc = repo
        .gccs
        .gccs
        .iter()
        .filter(|g| g.flavor == "upstream" && !g.forge.is_empty())
        .filter(|g| !g.released.is_empty() && g.released <= b.released)
        .max_by(|x, y| x.released.cmp(&y.released))
        .ok_or_else(|| format!("no GCC column is older than {}", b.id))?;
    fetch::run(
        repo,
        &Request {
            binutils: vec![b.id.clone()],
            ..Request::default()
        },
    )?;
    let build = Build {
        id: b.id.clone(),
        gcc: String::new(),
        gcc_file: String::new(),
        binutils: b.version.as_str().to_owned(),
        binutils_file: file_of(&b.url, &format!("binutils-{}.tar.xz", b.version)),
        forge: gcc.forge.clone(),
        image: image_for(repo, &gcc.forge)?,
        prerequisites: Vec::new(),
    };
    for target in targets {
        build.run(repo, target, target, jobs)?;
    }
    Ok(())
}

/// One run of `provision/forge/build.sh`, for every target of a bundle.
struct Build {
    id: String,
    /// Empty for binutils alone.
    gcc: String,
    gcc_file: String,
    binutils: String,
    binutils_file: String,
    forge: String,
    image: String,
    prerequisites: Vec<String>,
}

impl Build {
    /// Build for one target, whose tools are named for `tools`, and write the manifest.
    fn run(&self, repo: &Repo, target: &str, tools: &str, jobs: Option<u32>) -> Result<(), String> {
        let src = fetch::cache_dir().join("toolchains");
        let out = bundles_dir();
        std::fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;
        let script = repo.root.join("provision/forge/build.sh");
        println!(
            "{}: building for {target} with binutils {} in {}, as {tools}",
            self.id, self.binutils, self.forge
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
            .args(["-e", &format!("GK_ID={}", self.id)])
            .args(["-e", &format!("GK_GCC={}", self.gcc)])
            .args(["-e", &format!("GK_BINUTILS={}", self.binutils)])
            .args(["-e", &format!("GK_BINUTILS_TAR={}", self.binutils_file)])
            .args(["-e", &format!("GK_TARGET={tools}")])
            .args([
                "-e",
                &format!("GK_PREREQS={}", self.prerequisites.join(" ")),
            ]);
        if !self.gcc_file.is_empty() {
            cmd.args(["-e", &format!("GK_GCC_TAR={}", self.gcc_file)]);
        }
        if let Some(j) = jobs {
            cmd.args(["-e", &format!("GK_JOBS={j}")]);
        }
        let status = cmd
            .arg(&self.image)
            .args(["bash", "/gk/build.sh"])
            .status()
            .map_err(|e| format!("running docker: {e}"))?;
        if !status.success() {
            return Err(format!("{} for {target}: the forge build failed", self.id));
        }
        let file = format!("{}-{tools}.tar.zst", self.id);
        let plain = out.join(format!("{}-{tools}.tar", self.id));
        let tree = out.join(format!("{}-{tools}.tree", self.id));
        if tree.is_dir() {
            // A forge whose tar cannot set member times leaves the installed tree.
            pack(&tree, &plain)?;
        }
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
            id: self.id.clone(),
            gcc: self.gcc.clone(),
            binutils: self.binutils.clone(),
            target: tools.to_owned(),
            forge: self.forge.clone(),
            image: self.image.clone(),
            prerequisites: self.prerequisites.clone(),
            file,
            digest: digest.clone(),
            seconds: started.elapsed().as_secs(),
            gk: env!("CARGO_PKG_VERSION").to_owned(),
            package: String::new(),
            driver: String::new(),
            driver_sha256: String::new(),
        };
        let json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
        let path = out.join(format!("{}-{target}.json", self.id));
        std::fs::write(&path, json + "\n")
            .map_err(|e| format!("writing {}: {e}", path.display()))?;
        println!("{}: {target} {digest} in {}s", self.id, manifest.seconds);
        Ok(())
    }
}

/// The names binutils installs under `bin/`, after the target prefix, and under `<target>/bin/`. Every one of them is taken out of a GCC bundle before another binutils is laid over it, so none of the paired release's tools is left behind.
const BINUTILS_TOOLS: &[&str] = &[
    "addr2line",
    "ar",
    "as",
    "c++filt",
    "dlltool",
    "dllwrap",
    "dwp",
    "elfedit",
    "gprof",
    "gprofng",
    "ld",
    "ld.bfd",
    "ld.gold",
    "nm",
    "nlmconv",
    "objcopy",
    "objdump",
    "ranlib",
    "readelf",
    "size",
    "strings",
    "strip",
    "windmc",
    "windres",
];

/// A GCC bundle with the tools of a binutils bundle in place of its own, for a cell of the binutils sweep (spec 04.6). The GCC tree is linked rather than copied, and the result is kept in the cache under both digests. The GCC driver finds `as` and `ld` in `<prefix>/<target>/bin`, so both places are replaced.
pub fn combined(gcc: &Manifest, binutils: &Manifest) -> Result<PathBuf, String> {
    let short = |m: &Manifest| {
        m.digest
            .trim_start_matches("sha256:")
            .get(..12)
            .unwrap_or_default()
            .to_owned()
    };
    let dir = bundles_dir().join("unpacked").join(format!(
        "{}-{}-{}+{}-{}",
        gcc.id,
        gcc.target,
        short(gcc),
        binutils.id,
        short(binutils)
    ));
    if dir.is_dir() {
        return Ok(dir);
    }
    let from_gcc = unpacked(gcc)?;
    let from_binutils = unpacked(binutils)?;
    let partial = dir.with_extension("part");
    let _ = std::fs::remove_dir_all(&partial);
    let status = Command::new("cp")
        .arg("-al")
        .arg(&from_gcc)
        .arg(&partial)
        .status()
        .map_err(|e| format!("running cp: {e}"))?;
    if !status.success() {
        return Err(format!("linking {} failed", from_gcc.display()));
    }
    let t = &gcc.target;
    for tool in BINUTILS_TOOLS {
        let _ = std::fs::remove_file(partial.join("bin").join(format!("{t}-{tool}")));
        let _ = std::fs::remove_file(partial.join(t).join("bin").join(tool));
    }
    let _ = std::fs::remove_dir_all(partial.join(t).join("lib/ldscripts"));
    for sub in [
        "bin".to_owned(),
        format!("{t}/bin"),
        format!("{t}/lib/ldscripts"),
    ] {
        let from = from_binutils.join(&sub);
        if !from.is_dir() {
            continue;
        }
        let to = partial.join(&sub);
        std::fs::create_dir_all(&to).map_err(|e| format!("creating {}: {e}", to.display()))?;
        for entry in std::fs::read_dir(&from).map_err(|e| format!("{}: {e}", from.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            let target = to.join(entry.file_name());
            let _ = std::fs::remove_file(&target);
            std::fs::hard_link(entry.path(), &target)
                .map_err(|e| format!("linking {}: {e}", target.display()))?;
        }
    }
    std::fs::rename(&partial, &dir).map_err(|e| format!("renaming {}: {e}", partial.display()))?;
    Ok(dir)
}

/// Forge the bundle of a distribution column: links to the GCC and binutils of its host image, made by `provision/forge/distribution.sh` in that image, with what the image's packages say recorded in the manifest.
fn distribution(repo: &Repo, gcc: &Gcc) -> Result<(), String> {
    let image = image_for(repo, &gcc.forge)?;
    let out = bundles_dir();
    std::fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;
    let script = repo.root.join("provision/forge/distribution.sh");
    for target in &gcc.targets {
        println!(
            "{}: linking {} for {target} in {}",
            gcc.id, gcc.package, gcc.forge
        );
        let started = Instant::now();
        let status = Command::new("docker")
            .args(["run", "--rm", "--network", "none"])
            .arg("-v")
            .arg(format!("{}:/out", out.display()))
            .arg("-v")
            .arg(format!("{}:/gk/distribution.sh:ro", script.display()))
            .args(["-e", &format!("GK_ID={}", gcc.id)])
            .args(["-e", &format!("GK_TARGET={target}")])
            .args(["-e", &format!("GK_PACKAGE={}", gcc.package)])
            .arg(&image)
            .args(["bash", "/gk/distribution.sh"])
            .status()
            .map_err(|e| format!("running docker: {e}"))?;
        if !status.success() {
            return Err(format!("{} for {target}: the forge failed", gcc.id));
        }
        let tree = out.join(format!("{}-{target}.tree", gcc.id));
        let recorded = tree.join("bin/.gk-distribution");
        let text = std::fs::read_to_string(&recorded)
            .map_err(|e| format!("reading {}: {e}", recorded.display()))?;
        let field = |k: &str| {
            text.lines()
                .find_map(|l| l.strip_prefix(k)?.strip_prefix('='))
                .unwrap_or_default()
                .to_owned()
        };
        let plain = out.join(format!("{}-{target}.tar", gcc.id));
        pack(&tree, &plain)?;
        let status = Command::new("zstd")
            .args(["-q", "-f", "-19", "--rm"])
            .arg(&plain)
            .status()
            .map_err(|e| format!("running zstd: {e}"))?;
        if !status.success() {
            return Err(format!("compressing {} failed", plain.display()));
        }
        let file = format!("{}-{target}.tar.zst", gcc.id);
        let digest = format!("sha256:{}", net::sha256_file(&out.join(&file))?);
        let manifest = Manifest {
            id: gcc.id.clone(),
            gcc: gcc.version.as_str().to_owned(),
            binutils: field("binutils"),
            target: target.clone(),
            forge: gcc.forge.clone(),
            image: image.clone(),
            prerequisites: Vec::new(),
            file,
            digest: digest.clone(),
            seconds: started.elapsed().as_secs(),
            gk: env!("CARGO_PKG_VERSION").to_owned(),
            package: field("package"),
            driver: field("driver"),
            driver_sha256: field("driver-sha256"),
        };
        if manifest.binutils.is_empty() || manifest.driver.is_empty() {
            return Err(format!(
                "{} for {target}: {} is incomplete",
                gcc.id,
                recorded.display()
            ));
        }
        let json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
        let path = out.join(format!("{}-{target}.json", gcc.id));
        std::fs::write(&path, json + "\n")
            .map_err(|e| format!("writing {}: {e}", path.display()))?;
        println!("{}: {target} {} {digest}", gcc.id, manifest.driver);
    }
    Ok(())
}

/// Pack an installed tree the way build.sh does, with sorted members, no times and no owners, and remove the tree.
fn pack(tree: &Path, tar: &Path) -> Result<(), String> {
    let status = Command::new("tar")
        .args([
            "--sort=name",
            "--mtime=@0",
            "--owner=0",
            "--group=0",
            "--numeric-owner",
            "-C",
        ])
        .arg(tree)
        .arg("-cf")
        .arg(tar)
        .arg(".")
        .status()
        .map_err(|e| format!("running tar: {e}"))?;
    if !status.success() {
        return Err(format!("packing {} failed", tree.display()));
    }
    std::fs::remove_dir_all(tree).map_err(|e| format!("removing {}: {e}", tree.display()))
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
        // An image loaded from another machine has no build cache, so the build starts over and needs the network. The image itself is still good.
        let have = Command::new("docker")
            .args(["image", "inspect", "--format", "{{.Id}}", &tag])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if !have {
            return Err(format!("building {tag} failed"));
        }
        eprintln!("gk: building {tag} failed, so the image already here is used");
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
fn prerequisites(
    gcc_tar: &Path,
    top: &str,
    version: &Version,
    src: &Path,
) -> Result<Vec<String>, String> {
    let Ok(script) = tar_member(gcc_tar, &format!("{top}/contrib/download_prerequisites")) else {
        return before_the_script(version, src);
    };
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
        infrastructure(src, &file, &want)?;
        out.push(file);
    }
    Ok(out)
}

/// The prerequisites of a release older than `contrib/download_prerequisites`, which came with 4.6. Before 4.3 GCC needs none. 4.3 and 4.4 need GMP and MPFR, and 4.5 needs MPC as well, and they get the versions 4.6's script named first, which their installation notes accept.
fn before_the_script(version: &Version, src: &Path) -> Result<Vec<String>, String> {
    let wanted = match version.series(2)[..] {
        [4, 3 | 4] => 2,
        [4, 5] => 3,
        _ => 0,
    };
    let mut out = Vec::new();
    for (file, want) in &OLD_PREREQUISITES[..wanted] {
        infrastructure(src, file, want)?;
        out.push((*file).to_owned());
    }
    Ok(out)
}

/// Make sure one prerequisite tarball is in the cache with the SHA-512 it must have.
fn infrastructure(src: &Path, file: &str, want: &str) -> Result<(), String> {
    let path = src.join("infrastructure").join(file);
    if !path.is_file() || net::sha512_file(&path)? != want {
        net::download(&format!("{INFRASTRUCTURE}/{file}"), &path)?;
        if net::sha512_file(&path)? != want {
            let _ = std::fs::remove_file(&path);
            return Err(format!("{file} does not have the SHA-512 it must have"));
        }
    }
    Ok(())
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
            // A binutils bundle of the sweep has no compiler to try, and the cells that lay it over a GCC are its check.
            if m.gcc.is_empty() {
                continue;
            }
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
        // A distribution column's links only resolve in its own image.
        let own = [m.forge.as_str()];
        let hosts = if m.package.is_empty() {
            &hosts[..]
        } else {
            &own[..]
        };
        for host in hosts {
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
