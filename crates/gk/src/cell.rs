//! `gk probe` and `gk cell`: one kernel, one GCC, one platform, from the pinned tarball up the ladder (spec 07).
//!
//! Every step runs in the kernel's host container with no network. The tree is unpacked once into the cache and mounted read only at `/src`, kbuild writes under `O=/out`, and the bundle is mounted at `/opt/gk/t/<id>`, where `gk forge` installed it. `CROSS_COMPILE` names the bundle's tools on every platform, `x86_64` included, so nothing of the host's own GCC reaches the kernel. `CC` is `gk-cc`, copied into the scratch directory with a `gk-cc.toml` that names the bundle's GCC, so every compiler call kbuild makes lands in `compile.jsonl`.
//!
//! A kernel from before 2.6 has no `O=`, so its tree is copied into `/out` and built there, and `make dep` runs before the build. The museum's Makefiles put flags into `CC`, so instead of `CC` the shim is reached through a `CROSS_COMPILE` prefix of links in its directory, and before 2.0, which has no `CROSS_COMPILE`, each tool is named on the command line.
//!
//! A cell on a platform with an init pin, from 2.6 on, then boots the image with `gk boot` for L5 and runs the smoke suite for L6. Its coordinates then carry the boot container and the initramfs. A cell run with `--no-boot` stops at L4 and keeps the identity it had before booting existed.

use crate::build::{self, Calls};
use crate::fetch::{self, Request};
use crate::forge::{self, Manifest};
use crate::{boot, initramfs, kconfig, kernelorg, net, store, tap};
use gk_cc::config::ShimConfig;
use gk_model::cell::{Coordinates, Named};
use gk_model::platforms::Platform;
use gk_model::repo::Repo;
use gk_model::{Rung, Verdict, Version};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// The configuration every G0 cell uses.
pub const CONFIG: &str = "defconfig+gk";

/// Every configuration a cell can be built with (spec 07.2), as the name, the target that starts it, and the file of options added before the era's fragment, if any.
pub const CONFIGS: &[(&str, &str, Option<&str>)] = &[
    (CONFIG, "", None),
    ("tinyconfig+gk", "tinyconfig", Some("configs/tiny.gk")),
    ("allnoconfig+gk", "allnoconfig", Some("configs/tiny.gk")),
    ("allmodconfig", "allmodconfig", None),
];

/// The configurations that are built and never booted. They take only the switches the era's fragment turns off, so that `allmodconfig` stays what the kernel makes of it.
pub const BUILD_ONLY: &[&str] = &["allmodconfig"];

/// The make target a configuration starts from on one kernel: its own, or the platform's defconfig.
fn config_target(target: &str, defconfig: &str, version: &Version) -> Result<String, String> {
    if !target.is_empty() && kernelorg::is_museum(version) {
        return Err(format!(
            "{version} has no {target}, which came in 2.6; use {CONFIG}"
        ));
    }
    if target == "tinyconfig" && version.series(2) < [3, 17].to_vec() {
        return Err(format!(
            "{version} has no tinyconfig, which came in 3.17; use allnoconfig+gk"
        ));
    }
    Ok(if target.is_empty() { defconfig } else { target }.to_owned())
}

/// The make target that settles a `.config` after a fragment is laid over it. `olddefconfig` came in 3.7. Before that, `oldconfig` asks about each new symbol, and an empty answer takes the default, which is the same thing. Before 1.2 there is neither, as Configure only asks questions and never reads a `.config`, so there the fragment is not laid over at all and what it asks for and did not get is only noted.
fn settle_target(version: &Version) -> Option<&'static str> {
    if version.series(2) < [1, 2].to_vec() {
        None
    } else if version.series(2) < [3, 7].to_vec() {
        Some("oldconfig")
    } else {
        Some("olddefconfig")
    }
}

/// Shell commands that run before make for a tree that `O=` alone does not build.
///
/// From 2.6.5 to 2.6.8 `arch/x86_64/pci/Makefile` adds `-I arch/i386/pci` with a space, so kbuild does not see a `-I` word to point at the source tree, and the path is read from the output directory, where `pci.h` is not. Linking the headers of `arch/i386/pci` there builds what a tree built in place builds. 2.6.9 drops the space.
fn o_links(arch: &str, version: &Version) -> &'static str {
    let s = version.series(3);
    if arch == "x86_64" && s >= [2, 6, 5].to_vec() && s < [2, 6, 9].to_vec() {
        "mkdir -p /out/arch/i386/pci && for h in /src/arch/i386/pci/*.h; do ln -sf \"$h\" /out/arch/i386/pci/ || exit 1; done; "
    } else {
        ""
    }
}

/// A kernel's tree, fetched and unpacked into the cache if it is not there yet.
pub(crate) fn fetched_tree(
    repo: &Repo,
    version: &Version,
    file_name: &str,
) -> Result<PathBuf, String> {
    fetch::run(
        repo,
        &Request {
            kernels: vec![version.as_str().to_owned()],
            ..Request::default()
        },
    )?;
    unpack_tree(&fetch::cache_dir().join("kernels").join(file_name), version)
}

/// A configuration's entry in [`CONFIGS`], by name.
pub fn config_named(
    name: &str,
) -> Result<&'static (&'static str, &'static str, Option<&'static str>), String> {
    CONFIGS.iter().find(|c| c.0 == name).ok_or_else(|| {
        let names: Vec<&str> = CONFIGS.iter().map(|c| c.0).collect();
        format!("no configuration {name:?}; there is {}", names.join(", "))
    })
}

/// What kbuild would otherwise take from the clock and the machine.
const REPRODUCIBLE: [(&str, &str); 4] = [
    ("KBUILD_BUILD_TIMESTAMP", "Thu Jan  1 00:00:00 UTC 1970"),
    ("KBUILD_BUILD_USER", "gk"),
    ("KBUILD_BUILD_HOST", "gk"),
    ("KBUILD_BUILD_VERSION", "1"),
];

/// Everything a cell is made of, resolved and on disk.
pub struct Setup {
    /// The kernel version.
    pub version: Version,
    /// The platform.
    pub platform: Platform,
    /// The bundle's manifest.
    pub bundle: Manifest,
    /// The unpacked bundle.
    pub bundle_dir: PathBuf,
    /// The unpacked tree.
    pub tree: PathBuf,
    /// Its image.
    pub image: String,
    /// The era.
    pub era: String,
    /// The fragment's path.
    pub fragment: PathBuf,
    /// kbuild's `ARCH`.
    pub arch: String,
    /// The platform's defconfig target, which the accept probe runs.
    pub defconfig: String,
    /// The configuration's name, as `tinyconfig+gk`.
    pub config: &'static str,
    /// The target the configuration step starts from.
    pub target: String,
    /// The options added before the fragment, from files in `configs/`: the configuration's own, then the platform's for that configuration when it has one, as `configs/tiny.i386`.
    pub extra: Vec<PathBuf>,
    /// The platform's own options, added after the fragment, from `configs/platform.<name>` when the platform has one.
    pub platform_fragment: Option<PathBuf>,
    /// The boot image, relative to `arch/<ARCH>/boot`, or `vmlinux` at the top of the tree.
    pub image_name: String,
    /// The six coordinates, and the boot ones once [`Setup::booting`] has run.
    pub coordinates: Coordinates,
    /// Whether the cell boots after L4.
    pub boots: bool,
    /// The KUnit test options the fragment turns off, from its `# gk:kunit-skip` lines.
    pub kunit_skip: Vec<String>,
    /// Whether a failed build is run again with `make -k` to find every failing unit (spec 09.5).
    pub keep_going: bool,
}

impl Setup {
    /// Resolve and fetch everything a cell needs, which is rung L0.
    pub fn new(
        repo: &Repo,
        kernel: &str,
        gcc: &str,
        platform: &str,
        config: &str,
    ) -> Result<Self, String> {
        let version: Version = kernel
            .trim_start_matches("linux-")
            .parse()
            .map_err(|e| format!("kernel {kernel}: {e:?}"))?;
        let k = repo
            .kernels
            .get(&version)
            .ok_or_else(|| format!("{version} is not in kernels.toml"))?;
        let kernel = Named {
            name: format!("linux-{version}"),
            digest: format!("sha256:{}", k.sha256),
        };
        let tree = fetched_tree(repo, &version, k.file_name())?;
        Self::resolve(repo, version, kernel, tree, gcc, platform, config)
    }

    /// A cell on one commit of the history clone, whose tree is already exported to `tree` (spec 03.5). The kernel is named after the version its `Makefile` gives and the commit, and its digest is the commit.
    pub fn at_commit(
        repo: &Repo,
        commit: &str,
        version: &Version,
        tree: PathBuf,
        gcc: &str,
        platform: &str,
        config: &str,
    ) -> Result<Self, String> {
        let kernel = Named {
            name: format!("linux-{version}-g{}", &commit[..12.min(commit.len())]),
            digest: format!("git:{commit}"),
        };
        Self::resolve(repo, version.clone(), kernel, tree, gcc, platform, config)
    }

    /// Everything but the kernel, which the caller has found.
    #[allow(clippy::too_many_lines)]
    fn resolve(
        repo: &Repo,
        version: Version,
        kernel: Named,
        tree: PathBuf,
        gcc: &str,
        platform: &str,
        config: &str,
    ) -> Result<Self, String> {
        let &(config, target, extra) = config_named(config)?;
        let g = repo
            .gccs
            .gccs
            .iter()
            .find(|g| g.id == gcc || g.version.as_str() == gcc)
            .ok_or_else(|| format!("gcc {gcc} is not in gccs.toml"))?;
        let p = repo
            .platforms
            .get(platform)
            .ok_or_else(|| format!("platform {platform} is not in platforms.toml"))?
            .clone();
        if !p.applies(&version, &g.version) {
            return Err(format!(
                "{version} with {} on {platform} is n/a: the platform starts at {} and GCC {}",
                g.id, p.first_kernel, p.first_gcc
            ));
        }
        let era = repo
            .eras
            .of(&version)
            .ok_or_else(|| format!("{version} is in no era"))?;
        // A distribution column runs in its own image whatever the era, since its links resolve nowhere else (spec 04.3).
        let host = if g.flavor == "upstream" {
            repo.eras
                .host_for(&version)
                .ok_or_else(|| format!("{version} has no host in eras.toml"))?
                .to_owned()
        } else {
            g.forge.clone()
        };
        let arch = p.arch_for(&version).unwrap_or_default().to_owned();
        let defconfig = p.defconfig_for(&version).unwrap_or_default().to_owned();
        let target = config_target(target, &defconfig, &version)?;
        // The platform's file sits next to the configuration's, so a platform with none keeps the digest it had.
        let extra: Vec<PathBuf> = extra
            .map(|f| repo.root.join(f))
            .into_iter()
            .flat_map(|f| {
                let own = f.with_extension(&p.name);
                [Some(f), Some(own).filter(|o| o.is_file())]
            })
            .flatten()
            .collect();
        let image_name = p.image_for(&version).unwrap_or_default().to_owned();
        let bundle = forge::manifest(&g.id, &p.triple)?;
        // A distribution's binutils is its package, which the bundle's digest covers through the hashes it records.
        let binutils = if g.flavor == "upstream" {
            let b = repo
                .binutils
                .releases
                .iter()
                .find(|b| b.version.as_str() == bundle.binutils)
                .ok_or_else(|| format!("binutils {} is not in binutils.toml", bundle.binutils))?;
            Named {
                name: b.id.clone(),
                digest: format!("sha256:{}", b.sha256),
            }
        } else {
            Named {
                name: format!("{}-binutils-{}", g.flavor, bundle.binutils),
                digest: bundle.digest.clone(),
            }
        };
        let bundle_dir = forge::unpacked(&bundle)?;
        let image = forge::image_for(repo, &host)?;
        let fragment = repo.root.join(era.fragment());
        let platform_fragment = Some(
            repo.root
                .join("configs")
                .join(format!("platform.{}", p.name)),
        )
        .filter(|f| f.is_file());
        // The extra options go first and the platform's last, so the digest of a defconfig cell on a platform with no file of its own is the fragment's alone, as it was before there were other configurations.
        let mut fragment_text = Vec::new();
        for f in extra.iter().chain([&fragment]).chain(&platform_fragment) {
            fragment_text
                .extend(std::fs::read(f).map_err(|e| format!("reading {}: {e}", f.display()))?);
        }
        let coordinates = Coordinates {
            kernel,
            gcc: Named {
                name: g.id.clone(),
                digest: bundle.digest.clone(),
            },
            binutils,
            platform: p.name.clone(),
            config: Named {
                name: config.to_owned(),
                digest: format!("sha256:{}", net::sha256_bytes(&fragment_text)),
            },
            host: Named {
                name: host.clone(),
                digest: image_digest(&image)?,
            },
            qemu: String::new(),
            initramfs: String::new(),
        };
        Ok(Setup {
            version,
            platform: p,
            bundle,
            bundle_dir,
            tree,
            image,
            era: era.name.clone(),
            fragment,
            arch,
            defconfig,
            config,
            target,
            extra,
            platform_fragment,
            image_name,
            coordinates,
            boots: false,
            kunit_skip: kunit_skip(&String::from_utf8_lossy(&fragment_text)),
            keep_going: false,
        })
    }

    /// The same cell with another binutils laid over the bundle's own, for the binutils sweep (spec 04.6). The binutils coordinate names the release and the digest of its bundle. With the paired release the cell is left as it is, so the sweep's baseline is the matrix cell itself.
    pub fn with_binutils(mut self, repo: &Repo, binutils: &str) -> Result<Self, String> {
        let b = repo
            .binutils
            .releases
            .iter()
            .find(|b| b.id == binutils || b.version.as_str() == binutils)
            .ok_or_else(|| format!("binutils {binutils} is not in binutils.toml"))?;
        if b.version.as_str() == self.bundle.binutils {
            return Ok(self);
        }
        if !self.bundle.package.is_empty() {
            return Err(format!(
                "{} is a distribution column, whose binutils is its own",
                self.bundle.id
            ));
        }
        let m = forge::manifest(&b.id, &self.bundle.target)?;
        self.bundle_dir = forge::combined(&self.bundle, &m)?;
        self.coordinates.binutils = Named {
            name: b.id.clone(),
            digest: m.digest,
        };
        Ok(self)
    }

    /// The compiler `build.json` names. A distribution column's driver is read from its manifest, since its links do not resolve on the machine running gk.
    fn compiler(&self) -> build::Compiler {
        let mut c = build::Compiler::of(&self.bundle_dir, &self.bundle.target, &self.real_cc());
        if !self.bundle.driver.is_empty() {
            c.version.clone_from(&self.bundle.driver);
            c.sha256.clone_from(&self.bundle.driver_sha256);
        }
        c
    }

    /// Whether this cell can boot: the platform has an init pin, and the kernel is 2.6 or later, or 1.0 or later on i386, which `gk-init-museum` covers. The floppy boot of the kernels before 0.99.10 is not done yet.
    #[must_use]
    pub fn can_boot(&self) -> bool {
        let museum = initramfs::museum(&self.version);
        self.platform.init.is_some()
            && (!museum || self.platform.name == "i386")
            && self.version.series(1) >= [1].to_vec()
            && !BUILD_ONLY.contains(&self.config)
    }

    /// The build budget: the platform's, or ten times it for a build-only configuration, which compiles about ten times as much (spec 06.7), times the machine's [`budget_factor`].
    pub fn build_seconds(&self) -> u64 {
        let minutes = u64::from(self.platform.budget.build_minutes) * budget_factor();
        if BUILD_ONLY.contains(&self.config) {
            minutes * 600
        } else {
            minutes * 60
        }
    }

    /// Make the cell boot, which builds the initramfs if it has to and adds the boot container and the initramfs to the coordinates. A cell that cannot boot is left as it is.
    pub fn booting(mut self, repo: &Repo) -> Result<Self, String> {
        if !self.can_boot() {
            return Ok(self);
        }
        let (init, _) = initramfs::for_kernel(repo, &self.platform, &self.version)?;
        self.coordinates.qemu = image_digest(&forge::image_for(repo, "gk-boot")?)?;
        self.coordinates.initramfs = init.digest;
        self.boots = true;
        Ok(self)
    }

    /// Where the bundle is mounted in the container.
    fn bundle_mount(&self) -> String {
        format!("/opt/gk/t/{}", self.bundle.id)
    }

    /// The bundle's GCC, inside the container.
    fn real_cc(&self) -> String {
        format!("{}/bin/{}-gcc", self.bundle_mount(), self.bundle.target)
    }

    /// A make command in the host container, writing under `out`, with its output going to `log`. `cc` replaces `CC` when given. Returns the exit code.
    fn make(&self, step: &Make<'_>) -> Result<i32, String> {
        let file = std::fs::File::create(step.log)
            .map_err(|e| format!("creating {}: {e}", step.log.display()))?;
        let err = file.try_clone().map_err(|e| e.to_string())?;
        let mut cmd = Command::new("docker");
        cmd.args(["run", "--rm", "--network=none"])
            .arg("-v")
            .arg(format!("{}:/src:ro", self.tree.display()))
            .arg("-v")
            .arg(format!(
                "{}:{}:ro",
                self.bundle_dir.display(),
                self.bundle_mount()
            ))
            .arg("-v")
            .arg(format!("{}:/out", step.out.display()));
        if let Some(bin) = step.shim {
            cmd.arg("-v").arg(format!("{}:/gk/bin:ro", bin.display()));
        }
        for (k, v) in REPRODUCIBLE {
            cmd.args(["-e", &format!("{k}={v}")]);
        }
        let museum = kernelorg::is_museum(&self.version);
        cmd.args(["-w", "/out"]).arg(&self.image);
        cmd.args(["timeout", &step.seconds.to_string()]);
        if museum {
            // The tree is copied in the first time. Configure asks about every symbol the defconfig does not name, even with -d, and gives up at the end of its input, so it is answered with the default each time, as olddefconfig would. The Configure of 1.0 and 2.0 reads some answers from /dev/tty instead, which a container without a terminal cannot open, so /dev/tty becomes a pipe that a loop of yes keeps feeding. Some 1.x drivers include their headers by the absolute path /usr/src/linux, so the tree is there as well.
            cmd.args([
                "sh",
                "-c",
                "[ -f /out/Makefile ] || cp -a /src/. /out/ || exit 1; [ -e /usr/src/linux ] || { mkdir -p /usr/src && ln -s /out /usr/src/linux; } || exit 1; rm -f /dev/tty && mkfifo /dev/tty || exit 1; while :; do yes '' > /dev/tty; done & yes '' | make \"$@\"",
                "sh",
                "-C",
                "/out",
            ]);
        } else if settle_target(&self.version) == Some("oldconfig") {
            // oldconfig takes the default when its input ends, except in the 2.6.16 cycle, whose conf stops with "Console input is closed" at the first new symbol. A tree between releases often has a defconfig behind its Kconfig, so it has new symbols even with no fragment. An empty line is the default too, and every conf takes it.
            let script = format!("{}yes '' | make \"$@\"", o_links(&self.arch, &self.version));
            cmd.args(["sh", "-c", &script, "sh", "-C", "/src", "O=/out"]);
        } else {
            cmd.args(["make", "-C", "/src", "O=/out"]);
        }
        cmd.arg(format!("ARCH={}", self.arch))
            .args(self.tools(step.shim.is_some()))
            .arg(format!("-j{}", step.jobs))
            .args(step.args);
        let status = cmd
            .stdin(std::process::Stdio::null())
            .stdout(file)
            .stderr(err)
            .status()
            .map_err(|e| format!("running docker: {e}"))?;
        Ok(status.code().unwrap_or(-1))
    }
}

impl Setup {
    /// The make variables that name the compiler and the binutils. With the shim, the museum reaches it through the `gk-` links of [`Setup::shim_links`], because its Makefiles put flags into `CC`.
    fn tools(&self, shim: bool) -> Vec<String> {
        let bundle = format!("{}/bin/{}-", self.bundle_mount(), self.bundle.target);
        if !kernelorg::is_museum(&self.version) {
            let mut args = vec![format!("CROSS_COMPILE={bundle}")];
            if shim {
                args.push("CC=/gk/bin/gk-cc".into());
            }
            return args;
        }
        let prefix = if shim { "/gk/bin/gk-" } else { bundle.as_str() };
        if self.version.series(1) >= [2].to_vec() {
            let mut args = vec![format!("CROSS_COMPILE={prefix}")];
            if self.version.series(2) < [2, 1].to_vec() {
                // 2.0 makes piggy.o with encaps when `hash $(ENCAPS)` finds it, and bash's hash takes any name with a slash in it as found, so the cell names it plainly, as a native build would, and gets the objcopy path.
                args.push("ENCAPS=encaps".into());
            }
            return args;
        }
        // 1.x has no CROSS_COMPILE, and its CC carries the flags its Makefile gives it. 1.2 names the include directory by $(TOPDIR), which an old make loses the `$` of when it passes CC down, and 1.0 names none, as it expects /usr/include/linux to be a link into /usr/src/linux, so the cell names it by the path the tree is built in. The decompressor of 1.2 includes <stdlib.h>, which a native GCC of the time found among the libc5 headers and the bundle has no copy of, so the host's libc5 headers are searched last.
        let mut args = vec![format!(
            "CC={prefix}gcc -D__KERNEL__ -I/out/include -idirafter /usr/i486-linuxlibc1/include"
        )];
        for tool in ["as", "ld", "ar", "nm", "strip"] {
            args.push(format!("{}={prefix}{tool}", tool.to_ascii_uppercase()));
        }
        if self.version.series(2) < [1, 1].to_vec() {
            // The Makefiles of 1.0 run `for i in $(SUBDIRS)` where the list can be empty, which the bash of the time took and the bash of hamm calls a syntax error. Ash takes it. They also build the decompressor's xtract and piggyback, which run on the host, by make's own rule for a program, which links with CC, so the rule is pointed at the host's GCC with the flags of zBoot's Makefile.
            args.push("SHELL=/bin/ash".into());
            args.push("LINK.c=gcc -O2 -DSTDC_HEADERS".into());
        }
        args
    }

    /// The links of the `gk-` prefix in the shim's directory: `gk-gcc` to the shim, the binutils to the bundle's, and as86 and ld86, which build the real mode code of 2.0 and before, to the host's.
    fn shim_links(&self) -> Vec<(String, String)> {
        let mut links = vec![("gk-gcc".to_owned(), "gk-cc".to_owned())];
        for tool in [
            "as", "ld", "ar", "nm", "strip", "objcopy", "objdump", "ranlib",
        ] {
            links.push((
                format!("gk-{tool}"),
                format!("{}/bin/{}-{tool}", self.bundle_mount(), self.bundle.target),
            ));
        }
        for tool in ["as86", "ld86"] {
            links.push((format!("gk-{tool}"), format!("/usr/bin/{tool}")));
        }
        links
    }

    /// The script that stands for `gk-as` where the bundle's assembler cannot take the Makefile's flags as they are. The Makefiles of 1.0 call it with `-c`, which gas 1.x took and gas 2 refuses, so before 1.2 the script drops it.
    fn as_script(&self) -> Option<String> {
        (self.version.series(2) < [1, 2].to_vec()).then(|| {
            format!(
                "#!/bin/sh\nif [ \"$1\" = -c ]; then shift; fi\nexec {}/bin/{}-as \"$@\"\n",
                self.bundle_mount(),
                self.bundle.target
            )
        })
    }
}

/// One make step.
struct Make<'a> {
    out: &'a Path,
    shim: Option<&'a Path>,
    args: &'a [&'a str],
    log: &'a Path,
    jobs: usize,
    seconds: u64,
}

/// Unpack a kernel tarball into the cache once, and return the tree.
fn unpack_tree(tarball: &Path, version: &Version) -> Result<PathBuf, String> {
    let trees = fetch::cache_dir().join("trees");
    let tree = trees.join(format!("linux-{version}"));
    if tree.is_dir() {
        return Ok(tree);
    }
    let partial = trees.join(format!(".part-{version}"));
    let _ = std::fs::remove_dir_all(&partial);
    std::fs::create_dir_all(&partial)
        .map_err(|e| format!("creating {}: {e}", partial.display()))?;
    let status = Command::new("tar")
        .arg("-xf")
        .arg(tarball)
        .arg("-C")
        .arg(&partial)
        .status()
        .map_err(|e| format!("running tar: {e}"))?;
    // The tarballs before 2.0 unpack to `linux`.
    let mut top = partial.join(format!("linux-{version}"));
    if !top.is_dir() {
        top = partial.join("linux");
    }
    if !status.success() || !top.is_dir() {
        let _ = std::fs::remove_dir_all(&partial);
        return Err(format!(
            "{} did not unpack to linux-{version}",
            tarball.display()
        ));
    }
    std::fs::rename(&top, &tree).map_err(|e| format!("renaming {}: {e}", top.display()))?;
    let _ = std::fs::remove_dir_all(&partial);
    Ok(tree)
}

/// The digest of an image: the one it is pinned by, or the local image id for a `:dev` tag.
pub(crate) fn image_digest(image: &str) -> Result<String, String> {
    if let Some((_, digest)) = image.split_once('@') {
        return Ok(digest.to_owned());
    }
    let out = Command::new("docker")
        .args(["image", "inspect", "-f", "{{.Id}}", image])
        .output()
        .map_err(|e| format!("running docker: {e}"))?;
    let id = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if id.is_empty() {
        return Err(format!("docker knows no image {image}"));
    }
    Ok(id)
}

/// What the accept probe found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Probe {
    /// `accepted`, `refused`, or `inconclusive` when something other than a compiler check failed.
    pub result: String,
    /// The step that decided it.
    pub step: String,
    /// The lines of the log that say why, for anything but `accepted`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub why: Vec<String>,
    /// How long it took.
    pub seconds: f64,
}

impl Probe {
    /// Whether the cell passes L1.
    #[must_use]
    pub fn passes(&self) -> bool {
        self.result != "refused"
    }
}

/// What every build budget is multiplied by on this machine, from `GK_BUDGET_FACTOR`, for a machine shared with other work that leaves the builds a fraction of its cores. It applies to every cell the machine runs, so no column gets more time than another (spec 06.7).
pub fn budget_factor() -> u64 {
    std::env::var("GK_BUDGET_FACTOR")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .filter(|&f| f >= 1)
        .unwrap_or(1)
}

/// The accept probe of spec 09.4: the configuration target, which runs the Kconfig and `cc-version.sh` checks, then `init/main.i`, which runs every `#error` in the compiler headers with kbuild's own flags.
///
/// A refusal is a configuration step that fails, an `#error` directive that fires, or a missing `compiler-gccN.h`, which is how 2.6.29 to 4.1 refuse a GCC major they have no header for. Anything else that fails is inconclusive and is left to the build, because `init/main.i` depends on `prepare`, which compiles `bounds.c` and `asm-offsets.c` for real.
pub fn probe(s: &Setup, dir: &Path, jobs: usize) -> Result<Probe, String> {
    let clock = Instant::now();
    let out = dir.join("out");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;
    let seconds = u64::from(s.platform.budget.build_minutes) * 60 * budget_factor();
    let config_log = dir.join("probe-config.log");
    let code = s.make(&Make {
        out: &out,
        shim: None,
        args: &[s.defconfig.as_str()],
        log: &config_log,
        jobs: 1,
        seconds,
    })?;
    if code != 0 || !out.join(".config").is_file() {
        return Ok(Probe {
            result: "refused".into(),
            step: s.defconfig.clone(),
            why: tail_of(&config_log),
            seconds: clock.elapsed().as_secs_f64(),
        });
    }
    // Before 2.6 there is no init/main.i to make, and the compiler headers have no #error to fire, so a kernel that configures accepts.
    if kernelorg::is_museum(&s.version) {
        return Ok(Probe {
            result: "accepted".into(),
            step: s.defconfig.clone(),
            why: Vec::new(),
            seconds: clock.elapsed().as_secs_f64(),
        });
    }
    let main_log = dir.join("probe-main.log");
    let code = s.make(&Make {
        out: &out,
        shim: None,
        args: &["init/main.i"],
        log: &main_log,
        jobs,
        seconds,
    })?;
    let (result, why) = if code == 0 {
        ("accepted", Vec::new())
    } else {
        let errors = refusals(&std::fs::read_to_string(&main_log).unwrap_or_default());
        if errors.is_empty() {
            ("inconclusive", tail_of(&main_log))
        } else {
            ("refused", errors)
        }
    };
    Ok(Probe {
        result: result.into(),
        step: "init/main.i".into(),
        why,
        seconds: clock.elapsed().as_secs_f64(),
    })
}

/// One `make` in a cell's output directory: the arguments, the log's name and the jobs, to the exit code.
type MakeStep<'a> = dyn Fn(&[&str], &str, usize) -> Result<i32, String> + 'a;

/// The configuration of a cell, made in `out` by `make`: the configuration target, then the fragments, settled by the kernel's own Kconfig. Whether that worked, and the options of the fragments the kernel did not take.
fn configure(s: &Setup, out: &Path, make: &MakeStep<'_>) -> Result<(bool, Vec<String>), String> {
    let dot = out.join(".config");
    if make(&[s.target.as_str()], "config.log", 1)? != 0 || !dot.is_file() {
        return Ok((false, Vec::new()));
    }
    let mut fragment = Vec::new();
    for f in s
        .extra
        .iter()
        .chain([&s.fragment])
        .chain(&s.platform_fragment)
    {
        let text =
            std::fs::read_to_string(f).map_err(|e| format!("reading {}: {e}", f.display()))?;
        fragment.extend(kconfig::parse_fragment(&text));
    }
    if BUILD_ONLY.contains(&s.config) {
        fragment.retain(|(_, v)| v == "n");
    }
    let mut configured = true;
    if let Some(settle) = settle_target(&s.version) {
        let before =
            std::fs::read_to_string(&dot).map_err(|e| format!("reading {}: {e}", dot.display()))?;
        std::fs::write(&dot, kconfig::merge(&before, &fragment))
            .map_err(|e| format!("writing {}: {e}", dot.display()))?;
        configured = make(&[settle], "fragment.log", 1)? == 0;
    }
    Ok((
        configured,
        kconfig::missed(&kconfig::load(&dot)?, &fragment),
    ))
}

/// Whether one object of the kernel builds, for the bisection of a build failure (spec 03.5): the configuration of a cell, `prepare`, then that object alone, with the compiler and no shim, and no record. That takes minutes where a whole build takes an hour.
///
/// It is an error, which a bisection takes as a commit to skip, when the tree does not configure, when `prepare` fails somewhere else, or when the tree has no rule for the object. A failure in `prepare` that names the object's source counts as the object failing, so the units `prepare` builds itself, such as `kernel/bounds.s`, can be bisected too.
pub fn unit(s: &Setup, dir: &Path, object: &str, jobs: usize) -> Result<bool, String> {
    let out = dir.join("out");
    let _ = std::fs::remove_dir_all(&out);
    for l in ["config.log", "fragment.log"] {
        let _ = std::fs::remove_file(dir.join(l));
    }
    std::fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;
    let seconds = s.build_seconds();
    let make = |args: &[&str], log: &str, jobs: usize| {
        s.make(&Make {
            out: &out,
            shim: None,
            args,
            log: &dir.join(log),
            jobs,
            seconds,
        })
    };
    if !configure(s, &out, &make)?.0 {
        // The fragment step runs only when the first one passed, so its log names the step that failed.
        let log = ["fragment.log", "config.log"]
            .into_iter()
            .map(|l| dir.join(l))
            .find(|p| p.is_file())
            .unwrap_or_else(|| dir.join("config.log"));
        return Err(format!(
            "{} does not configure: {}",
            s.version,
            tail_of(&log).join(" | ")
        ));
    }
    if scripts_race(&s.tree) && make(&["scripts"], "scripts.log", 1)? != 0 {
        return Err("make scripts failed".into());
    }
    let source = object.rsplit_once('.').map_or(object, |(stem, _)| stem);
    if make(&["prepare"], "prepare.log", jobs)? != 0 {
        let log = std::fs::read_to_string(dir.join("prepare.log")).unwrap_or_default();
        // A `#error` is the tree refusing the compiler, as compiler.h does for a GCC outside its range, so the unit does not build on this commit any more than when its own source fails.
        if log
            .lines()
            .any(|l| (l.contains(source) && l.contains("error")) || l.contains("#error"))
        {
            return Ok(false);
        }
        return Err(format!(
            "make prepare failed: {}",
            tail_of(&dir.join("prepare.log")).join(" | ")
        ));
    }
    if make(&[object], "unit.log", jobs)? == 0 {
        return Ok(true);
    }
    let log = std::fs::read_to_string(dir.join("unit.log")).unwrap_or_default();
    if log.contains("No rule to make target") {
        return Err(format!("this tree has no rule for {object}"));
    }
    Ok(false)
}

/// The lines of a failed `init/main.i` that say the compiler was refused.
///
/// Before 2.6.17 or so, `init/main.i` with `O=` does not make the `asm` link, so every `asm/` header is missing. GCC 3.x and 4.5 on stop at the first missing header, but 4.0 to 4.4 go on, and the headers that needed `asm/` fire their `#error`s, such as "Please fix asm/byteorder.h". So when some other header is missing, only an `#error` in the compiler headers counts.
fn refusals(log: &str) -> Vec<String> {
    let missing = log
        .lines()
        .any(|l| l.contains("No such file") && !l.contains("linux/compiler-gcc"));
    log.lines()
        .filter(|l| {
            (l.contains("error: #error") && (!missing || l.contains("linux/compiler")))
                || (l.contains("linux/compiler-gcc") && l.contains("No such file"))
        })
        .take(6)
        .map(str::to_owned)
        .collect()
}

fn tail_of(log: &Path) -> Vec<String> {
    build::failure_lines(&std::fs::read_to_string(log).unwrap_or_default(), 6)
}

/// One step of the ladder as `cell.json` records it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Step {
    /// The rung, as `L3`.
    pub rung: String,
    /// Whether it passed.
    pub passed: bool,
    /// Wall seconds.
    pub seconds: f64,
    /// The log, relative to the cell directory, if the step has one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub log: String,
}

/// `cell.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CellRecord {
    /// The identity of spec 02.1.
    pub cell: String,
    /// The coordinates it is the hash of.
    pub coordinates: Coordinates,
    /// The highest rung passed, as `L4`, or empty when not even L0 passed.
    pub rung: String,
    /// The verdict.
    pub verdict: String,
    /// Each step climbed.
    pub steps: Vec<Step>,
    /// The rung each boot reached, in order, when the cell booted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boots: Vec<String>,
    /// Whether the boots disagreed (spec 02.4). The rung is then the lowest of them.
    #[serde(default)]
    pub flaky: bool,
    /// The accept probe.
    pub probe: Probe,
    /// The fragment's requests that did not take effect.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fragment_missed: Vec<String>,
    /// The era.
    pub era: String,
    /// When the cell started, in seconds since the epoch.
    pub started: u64,
    /// Total wall seconds.
    pub seconds: f64,
    /// The machine it ran on.
    pub machine: String,
    /// The gk version and commit.
    pub gk: String,
    /// False when the checkout had uncommitted changes, which makes the result one nobody should grade.
    pub graded: bool,
    /// The class of its first error, from `gk classify` (spec 08.2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<String>,
    /// The signatures that match it but whose ranges leave it out, from `gk classify`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<String>,
    /// How many units failed when the build was run again with `make -k`, for a cell that had that run (spec 09.5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units_failed: Option<usize>,
}

/// How many times a cell that reaches L5 boots (spec 02.4).
pub const BOOTS: usize = 3;

/// The rung one smoke boot reached, from L4 (it did not boot) to L6.
fn boot_rung(o: &boot::Outcome) -> Rung {
    if o.passed() {
        Rung::Smoke
    } else if o.booted() {
        Rung::Booted
    } else {
        Rung::Linked
    }
}

/// `kunit.json`: every KUnit suite over the cell's runs, and how it was graded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct KunitRecord {
    /// How many KUnit boots ran.
    pub runs: usize,
    /// The cell the suites were graded against, `self` for the era GCC's own cell, or empty when there was none to grade against.
    pub reference: String,
    /// The suites graded, which the reference passed in every run.
    pub graded: Vec<String>,
    /// The graded suites a run did not pass, by run.
    pub failed: Vec<Vec<String>>,
    /// Every suite, run by run.
    pub suites: Vec<tap::Tally>,
    /// The splats of every KUnit boot, by [`boot::splat_key`], which a cell graded against this one may show too.
    #[serde(default)]
    pub splats: Vec<String>,
}

/// `splats.json`: what kept each run from L8.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SplatRecord {
    /// The `objtool` warnings in the build logs.
    pub objtool: Vec<String>,
    /// Each run's splats on the smoke boot, every one of which counts.
    pub smoke: Vec<Vec<String>>,
    /// Each run's splats on the KUnit boot that the reference did not show, by key.
    pub kunit: Vec<Vec<String>>,
}

/// What booting a cell gave.
struct Booted {
    /// The rung of each run, after grading.
    rungs: Vec<Rung>,
    /// One step per boot.
    steps: Vec<Step>,
}

/// One boot of a cell's image.
fn boot_once(
    repo: &Repo,
    s: &Setup,
    image: &Path,
    dir: &Path,
    suite: &str,
    stem: &str,
    uniprocessor: bool,
) -> Result<boot::Outcome, String> {
    boot::run(
        repo,
        &boot::Boot {
            platform: &s.platform,
            version: &s.version,
            image,
            suite,
            dir,
            stem,
            uniprocessor,
        },
    )
}

/// Boot a linked cell, three times once it has booted at all, into `boot-1.log` and on. A kernel with KUnit built in boots a second time in each run that passed smoke, into `kunit-1.log` and on, and its suites are graded against the era GCC's cell for L7.
fn boots(
    repo: &Repo,
    s: &Setup,
    image: &Path,
    cell_dir: &Path,
    kunit: bool,
    uniprocessor: bool,
    objtool: &[String],
) -> Result<Booted, String> {
    let mut steps = Vec::new();
    let mut rungs = Vec::new();
    let mut suites = Vec::new();
    let mut smoke_splats = Vec::new();
    let mut kunit_splats = Vec::new();
    let mut kunit_seconds = 0.0;
    for n in 1..=BOOTS {
        let stem = format!("boot-{n}");
        let o = boot_once(repo, s, image, cell_dir, "smoke", &stem, uniprocessor)?;
        steps.push(Step {
            rung: Rung::Booted.to_string(),
            passed: o.booted(),
            seconds: o.seconds,
            log: format!("{stem}.log"),
        });
        if o.booted() {
            steps.push(Step {
                rung: Rung::Smoke.to_string(),
                passed: o.passed(),
                seconds: 0.0,
                log: format!("{stem}.json"),
            });
        }
        let rung = boot_rung(&o);
        rungs.push(rung);
        smoke_splats.push(o.splats.clone());
        // A cell that never booted is not one that reaches L5, so it is not repeated.
        if n == 1 && !o.booted() {
            break;
        }
        if kunit && rung == Rung::Smoke {
            let stem = format!("kunit-{n}");
            let k = boot_once(repo, s, image, cell_dir, "kunit", &stem, uniprocessor)?;
            kunit_seconds += k.seconds;
            kunit_splats.push(k.splats.iter().map(|l| boot::splat_key(l)).collect());
            suites.push(if k.ended() && k.panic.is_none() {
                Some(k.kunit)
            } else {
                None
            });
        } else {
            suites.push(None);
            kunit_splats.push(Vec::new());
        }
    }

    let allowed = if kunit {
        if !rungs.iter().all(|r| *r == Rung::Smoke) {
            return Ok(Booted { rungs, steps });
        }
        let (step, allowed) = grade(
            kunit_reference(repo, s)?,
            cell_dir,
            &mut rungs,
            &suites,
            &kunit_splats,
            kunit_seconds,
        )?;
        steps.push(step);
        allowed
    } else {
        // Before KUnit there is nothing to grade, and L7 passes vacuously.
        if s.version.series(2) < [5, 5].to_vec() {
            for r in &mut rungs {
                if *r == Rung::Smoke {
                    *r = Rung::Tested;
                }
            }
        }
        Some(Vec::new())
    };
    if rungs.contains(&Rung::Tested) {
        steps.push(clean(
            cell_dir,
            &mut rungs,
            objtool,
            smoke_splats,
            &kunit_splats,
            allowed.as_deref(),
        )?);
    }
    Ok(Booted { rungs, steps })
}

/// Raise each run at L7 with no `objtool` warning in the build, no splat on its smoke boot, and no splat on its KUnit boot that the reference did not show to L8, and write `splats.json`. `allowed` is `None` for the era GCC's own cell, which is its own reference. Returns the L8 step.
fn clean(
    cell_dir: &Path,
    rungs: &mut [Rung],
    objtool: &[String],
    smoke: Vec<Vec<String>>,
    kunit: &[Vec<String>],
    allowed: Option<&[String]>,
) -> Result<Step, String> {
    let unexpected: Vec<Vec<String>> = kunit
        .iter()
        .map(|run| {
            let mut keys: Vec<String> = run
                .iter()
                .filter(|k| {
                    // Keys stored by an older gk still name the function, and splat_key of a key is the key, so both sides go through it again.
                    let key = boot::splat_key(k);
                    allowed.is_some_and(|a| {
                        !a.iter()
                            .any(|x| boot::same_splat(&boot::splat_key(x), &key))
                    })
                })
                .cloned()
                .collect();
            keys.dedup();
            keys
        })
        .collect();
    for (n, r) in rungs.iter_mut().enumerate() {
        if *r == Rung::Tested
            && objtool.is_empty()
            && smoke.get(n).is_none_or(Vec::is_empty)
            && unexpected.get(n).is_none_or(Vec::is_empty)
        {
            *r = Rung::Clean;
        }
    }
    let record = SplatRecord {
        objtool: objtool.to_vec(),
        smoke,
        kunit: unexpected,
    };
    let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    std::fs::write(cell_dir.join("splats.json"), json + "\n")
        .map_err(|e| format!("writing splats.json: {e}"))?;
    Ok(Step {
        rung: Rung::Clean.to_string(),
        passed: rungs.iter().all(|r| *r == Rung::Clean),
        seconds: 0.0,
        log: "splats.json".into(),
    })
}

/// The `objtool` warnings in a cell's build logs, at most 100 of them.
fn objtool_warnings(cell_dir: &Path) -> Vec<String> {
    ["make.log", "modules.log"]
        .iter()
        .map(|f| crate::classify::log_text(cell_dir, f))
        .flat_map(|t| {
            t.lines()
                .filter(|l| l.contains("warning: objtool:"))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .take(100)
        .collect()
}

/// Grade the KUnit runs of a cell that passed smoke in every run against the era GCC's cell, raising each run that passed every graded suite to L7, and write `kunit.json`. Returns the L7 step.
fn grade(
    reference: Reference,
    cell_dir: &Path,
    rungs: &mut [Rung],
    suites: &[Option<Vec<tap::Suite>>],
    splats: &[Vec<String>],
    seconds: f64,
) -> Result<(Step, Option<Vec<String>>), String> {
    let mut seen: Vec<String> = splats.iter().flatten().cloned().collect();
    seen.sort();
    seen.dedup();
    let ran: Vec<Vec<tap::Suite>> = suites
        .iter()
        .map(|s| s.clone().unwrap_or_default())
        .collect();
    let mine = tap::tally(&ran);
    let (reference, theirs, allowed) = match reference {
        Reference::Itself => ("self".to_owned(), mine.clone(), None),
        Reference::Cell(id, tally, allowed) => (id, tally, Some(allowed)),
        Reference::None => (String::new(), Vec::new(), Some(Vec::new())),
    };
    let graded: Vec<&str> = tap::graded(&theirs, BOOTS);
    let mut failed = Vec::new();
    for (r, run) in rungs.iter_mut().zip(suites) {
        let missed = match run {
            Some(run) => tap::failed(run, &graded),
            None => graded.clone(),
        };
        if !reference.is_empty() && run.is_some() && missed.is_empty() {
            *r = Rung::Tested;
        }
        failed.push(missed.into_iter().map(str::to_owned).collect::<Vec<_>>());
    }
    let record = KunitRecord {
        runs: ran.len(),
        reference: reference.clone(),
        graded: graded.into_iter().map(str::to_owned).collect(),
        failed,
        suites: mine,
        splats: seen,
    };
    let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    std::fs::write(cell_dir.join("kunit.json"), json + "\n")
        .map_err(|e| format!("writing kunit.json: {e}"))?;
    let step = Step {
        rung: Rung::Tested.to_string(),
        passed: rungs.iter().all(|r| *r == Rung::Tested),
        seconds,
        log: "kunit.json".into(),
    };
    Ok((step, allowed))
}

/// What a cell's KUnit suites are graded against.
enum Reference {
    /// The cell is the era GCC's own.
    Itself,
    /// The era GCC's cell, by identity, its suites, and the splats its KUnit boots showed.
    Cell(String, Vec<tap::Tally>, Vec<String>),
    /// The era GCC's cell has not run, or did not reach KUnit.
    None,
}

/// Find the era GCC's cell for the same kernel, platform and boot rig in the store. When the era GCC is not a column, as the Debian build of M11 and M12 is not, the newest column of its release series stands in for it.
fn kunit_reference(repo: &Repo, s: &Setup) -> Result<Reference, String> {
    let Some(era) = repo.eras.of(&s.version) else {
        return Ok(Reference::None);
    };
    let Some(column) = crate::search::era_column(repo, &era.gcc, &s.platform.triple) else {
        return Ok(Reference::None);
    };
    if column == s.coordinates.gcc.name {
        return Ok(Reference::Itself);
    }
    let Ok(other) = Setup::new(
        repo,
        s.version.as_str(),
        &column,
        &s.platform.name,
        s.config,
    ) else {
        return Ok(Reference::None);
    };
    let other = other.booting(repo)?;
    let id = other.coordinates.identity();
    let path = store::cell_dir(&id).join("kunit.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(Reference::None);
    };
    let record: KunitRecord =
        serde_json::from_str(&text).map_err(|e| format!("reading {}: {e}", path.display()))?;
    Ok(Reference::Cell(id, record.suites, record.splats))
}

/// A cell's `kunit.json`, when it has one.
fn read_kunit(dir: &Path) -> Option<KunitRecord> {
    serde_json::from_str(&std::fs::read_to_string(dir.join("kunit.json")).ok()?).ok()
}

/// The era GCC's cell for a cell's kernel, platform, configuration and boot rig among `cells`, the newest when there are several, with its `kunit.json`. This is what [`kunit_reference`] finds by identity, found by coordinates instead, so that nothing has to be unpacked to name it.
fn era_cell(
    repo: &Repo,
    record: &CellRecord,
    cells: &[(PathBuf, CellRecord)],
) -> Option<(String, KunitRecord)> {
    let c = &record.coordinates;
    let version: Version = c.kernel.name.trim_start_matches("linux-").parse().ok()?;
    let triple = &repo.platforms.get(&c.platform)?.triple;
    let column = crate::search::era_column(repo, &repo.eras.of(&version)?.gcc, triple)?;
    if column == c.gcc.name {
        return None;
    }
    cells
        .iter()
        .filter(|(_, r)| {
            let o = &r.coordinates;
            o.gcc.name == column
                && o.kernel == c.kernel
                && o.platform == c.platform
                && o.config == c.config
                && o.qemu == c.qemu
                && o.initramfs == c.initramfs
        })
        .filter_map(|(dir, r)| Some((r.started, r.cell.clone(), read_kunit(dir)?)))
        .max_by_key(|(started, _, _)| *started)
        .map(|(_, id, k)| (id, k))
}

/// Whether a cell's KUnit runs are waiting for the era GCC's cell: every run passed smoke and was booted for KUnit, and there was nothing to grade them against when the cell ran.
fn waits_for_reference(record: &CellRecord, kunit: &KunitRecord) -> bool {
    kunit.reference.is_empty()
        && kunit.runs > 0
        && record.boots.len() == kunit.runs
        && record.boots.iter().all(|b| *b == Rung::Smoke.to_string())
}

/// A cell's `splats.json` when a KUnit splat that kept it from L8 is one the era GCC's cell showed too, as gk matches them now. A key written before keys stopped naming the function a warning was inlined into, or a warning a line or two off the reference's, held such a cell at L7.
fn stale_splats(dir: &Path, allowed: &[String]) -> Option<SplatRecord> {
    let text = std::fs::read_to_string(dir.join("splats.json")).ok()?;
    let record: SplatRecord = serde_json::from_str(&text).ok()?;
    record
        .kunit
        .iter()
        .flatten()
        .any(|k| {
            let key = boot::splat_key(k);
            allowed
                .iter()
                .any(|x| boot::same_splat(&boot::splat_key(x), &key))
        })
        .then_some(record)
}

/// Grade a cell's KUnit runs again once the era GCC's cell is in the store, when they had nothing to be graded against as the cell ran. A cell run on its own, or before its row's era cell, stops at L6 that way, and a cell whose `splats.json` holds a splat the era cell showed too is graded again. The runs are read back from `boot-N.json` and `kunit-N.json`, so nothing boots again, and the cell is raised to L7 and L8 as the first grading would have raised it. `cells` is the store, where the era cell is looked for. Returns whether the record changed.
pub fn regrade(
    repo: &Repo,
    dir: &Path,
    record: &mut CellRecord,
    cells: &[(PathBuf, CellRecord)],
) -> Result<bool, String> {
    let Some(kunit) = read_kunit(dir) else {
        return Ok(false);
    };
    // The objtool warnings come from the build log, which a cell copied without it no longer has. A cell graded before keeps them in its splats.json.
    let waiting = waits_for_reference(record, &kunit)
        && ["make.log", "make.log.zst"]
            .iter()
            .any(|f| dir.join(f).is_file());
    if !waiting && !dir.join("splats.json").is_file() {
        return Ok(false);
    }
    let Some((id, theirs)) = era_cell(repo, record, cells) else {
        return Ok(false);
    };
    let stale = if waiting {
        None
    } else {
        stale_splats(dir, &theirs.splats)
    };
    if !waiting && stale.is_none() {
        return Ok(false);
    }
    let reference = Reference::Cell(id, theirs.suites, theirs.splats);
    let outcome = |name: String| -> Result<boot::Outcome, String> {
        let path = dir.join(&name);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("reading {}: {e}", path.display()))
    };
    let mut suites = Vec::new();
    let mut smoke_splats = Vec::new();
    let mut kunit_splats = Vec::new();
    for n in 1..=kunit.runs {
        smoke_splats.push(outcome(format!("boot-{n}.json"))?.splats);
        let k = outcome(format!("kunit-{n}.json"))?;
        kunit_splats.push(k.splats.iter().map(|l| boot::splat_key(l)).collect());
        suites.push((k.ended() && k.panic.is_none()).then_some(k.kunit));
    }
    let tested = Rung::Tested.to_string();
    let clean_rung = Rung::Clean.to_string();
    let seconds = record
        .steps
        .iter()
        .find(|st| st.rung == tested)
        .map_or(0.0, |st| st.seconds);
    let mut rungs = vec![Rung::Smoke; kunit.runs];
    let (step, allowed) = grade(reference, dir, &mut rungs, &suites, &kunit_splats, seconds)?;
    record
        .steps
        .retain(|st| st.rung != tested && st.rung != clean_rung);
    record.steps.push(step);
    if rungs.contains(&Rung::Tested) {
        let objtool = stale.map_or_else(|| objtool_warnings(dir), |s| s.objtool);
        record.steps.push(clean(
            dir,
            &mut rungs,
            &objtool,
            smoke_splats,
            &kunit_splats,
            allowed.as_deref(),
        )?);
    }
    let reached = rungs.iter().copied().min().unwrap_or(Rung::Smoke);
    record.rung = reached.to_string();
    record.verdict = format!("{:?}", Verdict::of(Some(reached), Rung::Clean)).to_lowercase();
    record.boots = rungs.iter().map(ToString::to_string).collect();
    record.flaky = rungs.windows(2).any(|w| w[0] != w[1]);
    record.classes.clear();
    record.findings.clear();
    Ok(true)
}

/// Whether the top Makefile reaches `scripts/` through two phony targets, `scripts` and `scripts/fixdep`, as in the first 2.6 releases. A parallel build then runs two makes in `scripts/` at once, and they race on the temporary files of `split-include`, so the cell fails whatever the GCC. Those trees get their helpers built first, with one job, which changes nothing that is compiled for the kernel.
fn scripts_race(tree: &Path) -> bool {
    std::fs::read_to_string(tree.join("Makefile"))
        .is_ok_and(|m| m.lines().any(|l| l.starts_with("scripts/fixdep:")))
}

/// Whether the top Makefile hands each goal to a make of its own in the output directory, as from 2.6.0 until `sub-make` took over in 2.6.2x. With `-j` the image and `modules` then build side by side in two makes that know nothing of each other, and both write `include/linux/version.h` and `include/config/MARKER`, so the cell fails whatever the GCC. Those trees get their goals one after the other, which builds the same thing.
fn goals_race(tree: &Path) -> bool {
    std::fs::read_to_string(tree.join("Makefile")).is_ok_and(|m| {
        m.lines().any(|l| {
            l.starts_with("$(filter-out all,$(MAKECMDGOALS)) all:")
                || l.starts_with("$(filter-out _all,$(MAKECMDGOALS)) _all:")
        })
    })
}

/// Run a cell up to L4, or up to L6 when it boots, and write its directory. Returns the directory and the record.
#[allow(clippy::too_many_lines)]
pub fn run(
    repo: &Repo,
    s: &Setup,
    jobs: usize,
    keep: bool,
) -> Result<(PathBuf, CellRecord), String> {
    let started = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let clock = Instant::now();
    let id = s.coordinates.identity();
    let cell_dir = store::cell_dir(&id);
    // Before the old record goes: a shim that cannot run in the host would fail every cell at L3.
    let shim_path = shim_binary()?;
    let scratch = fetch::cache_dir()
        .join("scratch")
        .join(s.coordinates.short_id());
    let _ = std::fs::remove_dir_all(&scratch);
    let _ = std::fs::remove_dir_all(&cell_dir);
    for d in [&cell_dir, &scratch] {
        std::fs::create_dir_all(d).map_err(|e| format!("creating {}: {e}", d.display()))?;
    }
    let (gk, graded) = gk_commit(&repo.root);
    let seconds = s.build_seconds();
    let mut steps = vec![Step {
        rung: Rung::Fetched.to_string(),
        passed: true,
        seconds: 0.0,
        log: String::new(),
    }];
    let mut reached = Rung::Fetched;
    let mut boot_runs = Vec::new();
    let mut flaky = false;

    let probe_clock = Instant::now();
    let probe = probe(s, &scratch.join("probe"), jobs)?;
    for log in ["probe-config.log", "probe-main.log"] {
        let _ = std::fs::copy(scratch.join("probe").join(log), cell_dir.join(log));
    }
    steps.push(Step {
        rung: Rung::Accepted.to_string(),
        passed: probe.passes(),
        seconds: probe_clock.elapsed().as_secs_f64(),
        log: "probe-main.log".into(),
    });

    // The shim, next to its settings, mounted at /gk/bin.
    let bin = scratch.join("bin");
    std::fs::create_dir_all(&bin).map_err(|e| format!("creating {}: {e}", bin.display()))?;
    std::fs::copy(&shim_path, bin.join("gk-cc")).map_err(|e| format!("copying gk-cc: {e}"))?;
    let shim = ShimConfig {
        real: PathBuf::from(s.real_cc()),
        log: PathBuf::from("/out/compile.jsonl"),
        twice: false,
    };
    std::fs::write(bin.join(gk_cc::config::FILE_NAME), shim.to_toml())
        .map_err(|e| format!("writing the shim's settings: {e}"))?;
    if kernelorg::is_museum(&s.version) {
        for (link, target) in s.shim_links() {
            std::os::unix::fs::symlink(&target, bin.join(&link))
                .map_err(|e| format!("linking {link}: {e}"))?;
        }
        if let Some(script) = s.as_script() {
            use std::os::unix::fs::PermissionsExt;
            let path = bin.join("gk-as");
            let _ = std::fs::remove_file(&path);
            std::fs::write(&path, script).map_err(|e| format!("writing gk-as: {e}"))?;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("writing gk-as: {e}"))?;
        }
    }
    let out = scratch.join("out");
    std::fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;
    let make = |args: &[&str], log: &str, jobs: usize| {
        s.make(&Make {
            out: &out,
            shim: Some(&bin),
            args,
            log: &cell_dir.join(log),
            jobs,
            seconds,
        })
    };

    let mut fragment_missed = Vec::new();
    let mut configured = false;
    if probe.passes() {
        reached = Rung::Accepted;
        let c = Instant::now();
        (configured, fragment_missed) = configure(s, &out, &make)?;
        let dot = out.join(".config");
        if dot.is_file() {
            if configured && !s.kunit_skip.is_empty() && !BUILD_ONLY.contains(&s.config) {
                // With KUNIT_ALL_TESTS off the tests keep the values it gave them, and their options can be turned off one by one.
                let off: Vec<(String, String)> = std::iter::once("KUNIT_ALL_TESTS")
                    .chain(s.kunit_skip.iter().map(String::as_str))
                    .map(|o| (o.to_owned(), "n".to_owned()))
                    .collect();
                let resolved = std::fs::read_to_string(&dot)
                    .map_err(|e| format!("reading {}: {e}", dot.display()))?;
                std::fs::write(&dot, kconfig::merge(&resolved, &off))
                    .map_err(|e| format!("writing {}: {e}", dot.display()))?;
                configured = make(
                    &[settle_target(&s.version).unwrap_or("oldconfig")],
                    "kunit-skip.log",
                    1,
                )? == 0;
                fragment_missed.extend(kconfig::missed(&kconfig::load(&dot)?, &off[1..]));
            }
            let _ = std::fs::copy(&dot, cell_dir.join(".config"));
            // A 32 bit kernel in the x86_64 row would be an i386 result under the wrong name (spec 06.2).
            if s.platform.name == "x86_64"
                && kconfig::load(&dot)?.get("X86_32").is_some_and(|v| v == "y")
            {
                return Err(format!(
                    "{} {} configured a 32 bit kernel on x86_64",
                    s.version, s.config
                ));
            }
        }
        steps.push(Step {
            rung: Rung::Configured.to_string(),
            passed: configured,
            seconds: c.elapsed().as_secs_f64(),
            log: "config.log".into(),
        });
    }

    let mut built = false;
    let mut timed_out = false;
    let mut kept_going = false;
    let mut image_sha256 = String::new();
    let mut targets = vec![s.image_name.as_str()];
    if configured {
        reached = Rung::Configured;
        let config = kconfig::load(&out.join(".config"))?;
        let modules = config.get("MODULES").is_some_and(|v| v == "y");
        if modules {
            targets.push("modules");
        }
        let mut args = vec!["V=1"];
        args.extend(&targets);
        let c = Instant::now();
        let mut code = 0;
        if kernelorg::is_museum(&s.version) {
            // Before 2.6 the dependencies are made by hand, and one job at a time, before anything is built.
            code = make(&["dep"], "dep.log", 1)?;
        } else if scripts_race(&s.tree) {
            code = make(&["V=1", "scripts"], "scripts.log", 1)?;
        }
        if code == 0 && goals_race(&s.tree) {
            let log = cell_dir.join("make.log");
            for (i, target) in targets.iter().enumerate() {
                if i == 0 {
                    code = make(&["V=1", target], "make.log", jobs)?;
                } else {
                    code = make(&["V=1", target], "make-goal.log", jobs)?;
                    let more = cell_dir.join("make-goal.log");
                    let text = std::fs::read(&more)
                        .map_err(|e| format!("reading {}: {e}", more.display()))?;
                    std::fs::OpenOptions::new()
                        .append(true)
                        .open(&log)
                        .and_then(|mut f| std::io::Write::write_all(&mut f, &text))
                        .map_err(|e| format!("writing {}: {e}", log.display()))?;
                    let _ = std::fs::remove_file(&more);
                }
                if code != 0 {
                    break;
                }
            }
        } else if code == 0 {
            code = make(&args, "make.log", jobs)?;
        }
        built = code == 0;
        timed_out = code == 124;
        if !built && !timed_out && s.keep_going && !killed(&cell_dir.join("make.log")) {
            // The units that failed are tried again and the rest of the tree is built past them. Their records are added to compile.jsonl, and failing_units keeps one per unit.
            let mut again = vec!["-k", "V=1"];
            again.extend(&targets);
            make(&again, "make-k.log", jobs)?;
            kept_going = true;
        }
        if !built && killed(&cell_dir.join("make.log")) {
            let _ = std::fs::remove_dir_all(&cell_dir);
            if !keep {
                let _ = std::fs::remove_dir_all(&scratch);
            }
            return Err(
                "a build step was killed (exit 137), most likely by the OOM killer, so the cell has no verdict"
                    .into(),
            );
        }
        steps.push(Step {
            rung: Rung::Compiled.to_string(),
            passed: built,
            seconds: c.elapsed().as_secs_f64(),
            log: "make.log.zst".into(),
        });
        if built {
            reached = Rung::Compiled;
            let c = Instant::now();
            // ARCH=i386 and x86_64 build under arch/x86 from 2.6.24, and 1.0 leaves its image at the top of the tree.
            let image = if s.image_name == "vmlinux" {
                out.join("vmlinux")
            } else {
                [s.arch.as_str(), "x86"]
                    .iter()
                    .map(|a| out.join("arch").join(a).join("boot").join(&s.image_name))
                    .chain([out.join(&s.image_name)])
                    .find(|p| p.is_file())
                    .unwrap_or_default()
            };
            let mut linked = image.is_file();
            if linked && modules {
                linked = make(
                    &[
                        "INSTALL_MOD_PATH=/out/staging",
                        "INSTALL_MOD_STRIP=1",
                        "modules_install",
                    ],
                    "modules.log",
                    1,
                )? == 0;
            }
            steps.push(Step {
                rung: Rung::Linked.to_string(),
                passed: linked,
                seconds: c.elapsed().as_secs_f64(),
                log: if modules {
                    "modules.log".into()
                } else {
                    String::new()
                },
            });
            if linked {
                reached = Rung::Linked;
                image_sha256 = net::sha256_file(&image).unwrap_or_default();
                if s.boots {
                    let dot = std::fs::read_to_string(out.join(".config")).unwrap_or_default();
                    let kunit = dot.lines().any(|l| l == "CONFIG_KUNIT=y");
                    // The machine always has two CPUs, and a kernel built without SMP can only count one.
                    let uniprocessor = dot.lines().any(|l| l == "# CONFIG_SMP is not set");
                    let objtool = objtool_warnings(&cell_dir);
                    let b = boots(repo, s, &image, &cell_dir, kunit, uniprocessor, &objtool)?;
                    steps.extend(b.steps);
                    reached = b.rungs.iter().copied().min().unwrap_or(Rung::Linked);
                    flaky = b.rungs.windows(2).any(|w| w[0] != w[1]);
                    boot_runs = b.rungs;
                }
            }
        }
    }

    // build.json, in rucc-kernel's schema with the cell and rung added.
    let log = out.join("compile.jsonl");
    let (records, unreadable) = if log.is_file() {
        gk_cc::record::read_log(&log).map_err(|e| format!("reading {}: {e}", log.display()))?
    } else {
        (Vec::new(), 0)
    };
    let calls: Calls = build::count(&records, unreadable);
    let errors = build::failing_units(&records, Path::new("/src"));
    let outcome = build::Outcome {
        version: s.version.as_str().to_owned(),
        row: s.platform.name.clone(),
        config: s.config.to_owned(),
        era: s.era.clone(),
        gnuc: String::new(),
        std: String::new(),
        source: PathBuf::from("/src"),
        compiler: s.compiler(),
        persona: Vec::new(),
        targets: targets.iter().map(|t| (*t).to_owned()).collect(),
        kcflags: Vec::new(),
        fragment: format!("fragment.{}", s.era),
        fragment_missed: fragment_missed.clone(),
        configured,
        built,
        config_sha256: net::sha256_file(&out.join(".config")).unwrap_or_default(),
        image_sha256,
        wall_seconds: clock.elapsed().as_secs_f64(),
        errors: build::error_census(&records),
        failed_units: errors.iter().map(|e| e.unit.clone()).collect(),
        graded,
        stopped: build::stopped(&cell_dir, configured, built, calls.units_failed, timed_out),
        calls,
        cell: id.clone(),
        rung: reached.to_string(),
    };
    let json = serde_json::to_string_pretty(&outcome).map_err(|e| e.to_string())?;
    std::fs::write(cell_dir.join("build.json"), json + "\n")
        .map_err(|e| format!("writing build.json: {e}"))?;
    let mut lines = String::new();
    for e in &errors {
        lines.push_str(&serde_json::to_string(e).map_err(|e| e.to_string())?);
        lines.push('\n');
    }
    std::fs::write(cell_dir.join("errors.jsonl"), lines)
        .map_err(|e| format!("writing errors.jsonl: {e}"))?;
    let mut lines = String::new();
    for w in build::warning_census(&records, Path::new("/src")) {
        lines.push_str(&serde_json::to_string(&w).map_err(|e| e.to_string())?);
        lines.push('\n');
    }
    std::fs::write(cell_dir.join("warnings.jsonl"), lines)
        .map_err(|e| format!("writing warnings.jsonl: {e}"))?;
    if log.is_file() {
        let _ = std::fs::copy(&log, cell_dir.join("compile.jsonl"));
    }
    for name in ["make.log", "make-k.log"] {
        let make_log = cell_dir.join(name);
        if make_log.is_file() {
            let status = Command::new("zstd")
                .args(["-q", "-f", "-19", "--rm"])
                .arg(&make_log)
                .status();
            if !status.is_ok_and(|s| s.success()) {
                return Err(format!("compressing {name} failed"));
            }
        }
    }

    let verdict = Verdict::of(Some(reached), Rung::Clean);
    let mut record = CellRecord {
        cell: id,
        coordinates: s.coordinates.clone(),
        rung: reached.to_string(),
        verdict: format!("{verdict:?}").to_lowercase(),
        steps,
        boots: boot_runs.iter().map(ToString::to_string).collect(),
        flaky,
        probe,
        fragment_missed,
        era: s.era.clone(),
        started,
        seconds: clock.elapsed().as_secs_f64(),
        machine: hostname(),
        gk,
        graded,
        classes: Vec::new(),
        findings: Vec::new(),
        units_failed: kept_going.then_some(errors.len()),
    };
    if let Some(f) = crate::classify::failure(&cell_dir, &record) {
        let catalog = crate::classify::compile(repo);
        let v = crate::classify::judge(repo, &catalog, &record, &f);
        record.classes = crate::classify::classes(repo, &catalog, &cell_dir, &record, &f, &v);
        record.findings = v.findings;
    }
    let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    std::fs::write(cell_dir.join("cell.json"), json + "\n")
        .map_err(|e| format!("writing cell.json: {e}"))?;
    if !keep {
        // kbuild ran as root in the container, so the scratch tree is removed from a container too.
        let _ = Command::new("docker")
            .args(["run", "--rm", "--network=none", "-v"])
            .arg(format!("{}:/scratch", scratch.display()))
            .arg(&s.image)
            .args([
                "rm",
                "-rf",
                "/scratch/out",
                "/scratch/probe",
                "/scratch/bin",
            ])
            .status();
        let _ = std::fs::remove_dir_all(&scratch);
    }
    Ok((cell_dir, record))
}

/// Where `gk-cc` is: next to the running `gk`. It runs inside the host container, so it has to be a static build, as `cargo build --release --target x86_64-unknown-linux-musl -p gk-cc` gives.
fn shim_binary() -> Result<PathBuf, String> {
    let shim = if let Some(path) = std::env::var_os("GK_SHIM") {
        PathBuf::from(path)
    } else {
        let exe = std::env::current_exe().map_err(|e| format!("finding gk: {e}"))?;
        exe.with_file_name("gk-cc")
    };
    if !shim.is_file() {
        return Err(format!(
            "gk-cc is not at {}; build it for musl and copy it there, or set GK_SHIM",
            shim.display()
        ));
    }
    let bytes = std::fs::read(&shim).map_err(|e| format!("reading {}: {e}", shim.display()))?;
    if interpreted(&bytes) {
        return Err(format!(
            "{} is linked against the C library of the machine that built it, which the host images do not have; build gk-cc for musl, or set GK_SHIM to a static one",
            shim.display()
        ));
    }
    Ok(shim)
}

/// Whether a 64-bit little-endian ELF file names a program interpreter, that is, whether it needs the dynamic loader of the C library it was linked against.
fn interpreted(elf: &[u8]) -> bool {
    const PT_INTERP: u32 = 3;
    if elf.len() < 64 || &elf[..4] != b"\x7fELF" || elf[4] != 2 || elf[5] != 1 {
        return false;
    }
    let u16_at = |o: usize| elf.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let u32_at = |o: usize| {
        elf.get(o..o + 4)
            .and_then(|b| b.try_into().ok())
            .map(u32::from_le_bytes)
    };
    let u64_at = |o: usize| {
        elf.get(o..o + 8)
            .and_then(|b| b.try_into().ok())
            .map(u64::from_le_bytes)
    };
    let (Some(phoff), Some(size), Some(count)) = (u64_at(32), u16_at(54), u16_at(56)) else {
        return false;
    };
    let Ok(phoff) = usize::try_from(phoff) else {
        return false;
    };
    (0..usize::from(count)).any(|i| u32_at(phoff + i * usize::from(size)) == Some(PT_INTERP))
}

/// The gk version and commit, and whether the checkout is clean.
pub fn gk_commit(root: &Path) -> (String, bool) {
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
    };
    let commit = git(&["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let clean =
        git(&["status", "--porcelain", "--untracked-files=no"]).is_some_and(|s| s.is_empty());
    (format!("{} {commit}", env!("CARGO_PKG_VERSION")), clean)
}

fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_owned())
        .unwrap_or_default()
}

/// The options named on the `# gk:kunit-skip` lines of a fragment, which Kconfig reads as comments. A KUnit test goes there when it fails under TCG on a loaded host for reasons that have nothing to do with the compiler. `KUNIT_ALL_TESTS` hides its option, so a plain `=n` in the fragment would not take, and the configuration step turns these off in a second pass with `KUNIT_ALL_TESTS` off instead. Being in the fragment puts the list in the cell's identity.
fn kunit_skip(fragment: &str) -> Vec<String> {
    fragment
        .lines()
        .filter_map(|l| l.strip_prefix("# gk:kunit-skip "))
        .flat_map(str::split_whitespace)
        .map(str::to_owned)
        .collect()
}

/// Whether make reports a recipe killed by SIGKILL, which on these hosts is the OOM killer. That says how much memory the machine had left, not what the GCC did, so it must not become a verdict.
fn killed(log: &Path) -> bool {
    std::fs::read_to_string(log).is_ok_and(|text| {
        text.lines()
            .any(|l| l.starts_with("make") && l.contains("***") && l.ends_with("Error 137"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_cells_at_l6_with_no_reference_wait_for_one() {
        let record = |boots: &str| -> CellRecord {
            let json = format!(
                r#"{{"cell":"sha256:00","coordinates":{{"kernel":{{"name":"linux-6.12.112","digest":""}},"gcc":{{"name":"gcc-16.2.0","digest":""}},"binutils":{{"name":"binutils-2.44","digest":""}},"platform":"x86_64","config":{{"name":"defconfig+gk","digest":""}},"host":{{"name":"bookworm","digest":""}},"qemu":"","initramfs":""}},"rung":"L6","verdict":"runs","steps":[],"boots":[{boots}],"probe":{{"result":"accepted","step":"init/main.i","seconds":1.0}},"era":"M12","started":0,"seconds":1.0,"machine":"x","gk":"0","graded":true}}"#
            );
            serde_json::from_str(&json).unwrap()
        };
        let kunit = |reference: &str| KunitRecord {
            runs: 3,
            reference: reference.into(),
            graded: Vec::new(),
            failed: vec![Vec::new(); 3],
            suites: Vec::new(),
            splats: Vec::new(),
        };
        let all = record(r#""L6","L6","L6""#);
        assert!(waits_for_reference(&all, &kunit("")));
        assert!(!waits_for_reference(&all, &kunit("self")));
        assert!(!waits_for_reference(&all, &kunit("sha256:ab")));
        assert!(!waits_for_reference(
            &record(r#""L6","L5","L6""#),
            &kunit("")
        ));
        assert!(!waits_for_reference(&record(r#""L6""#), &kunit("")));
    }

    #[test]
    fn the_fragment_names_the_skipped_tests() {
        let text = "CONFIG_KUNIT=y\n# gk:kunit-skip DRM_SCHED_KUNIT_TEST RATELIMIT_KUNIT_TEST\n# a comment\n";
        assert_eq!(
            kunit_skip(text),
            ["DRM_SCHED_KUNIT_TEST", "RATELIMIT_KUNIT_TEST"]
        );
        assert!(kunit_skip("CONFIG_KUNIT=y\n").is_empty());
    }

    #[test]
    fn an_error_that_needs_a_missing_asm_header_is_no_refusal() {
        // 2.6.16 on x86_64 with GCC 4.1.2, which goes on past the missing headers.
        let knock_on = "/src/init/main.c:51:20: error: asm/io.h: No such file or directory\n/src/include/linux/kernel.h:243:2: error: #error \"Please fix asm/byteorder.h\"\n/src/include/linux/jiffies.h:33:3: error: #error You lose.\n";
        assert!(refusals(knock_on).is_empty());
        // The same kernel with GCC 12, which its compiler.h refuses.
        let refused = "/src/include/linux/compiler.h:40:2: error: #error no compiler-gcc.h file for this gcc version\n/src/include/linux/posix_types.h:47:10: fatal error: asm/posix_types.h: No such file or directory\n";
        assert_eq!(refusals(refused).len(), 1);
        // With every header there, any #error is a refusal, and so is a missing compiler-gccN.h.
        assert_eq!(
            refusals(
                "/src/include/linux/compiler-gcc4.h:9:3: error: #error Your compiler is too buggy\n"
            )
            .len(),
            1
        );
        assert_eq!(
            refusals("/src/include/linux/compiler-gcc.h:120:30: fatal error: linux/compiler-gcc9.h: No such file or directory\n").len(),
            1
        );
    }

    #[test]
    fn a_shim_that_needs_the_dynamic_loader_is_found_out() {
        // An ELF header with two program headers at 64, 56 bytes each.
        let mut elf = vec![0u8; 64 + 2 * 56];
        elf[..4].copy_from_slice(b"\x7fELF");
        elf[4] = 2;
        elf[5] = 1;
        elf[32..40].copy_from_slice(&64u64.to_le_bytes());
        elf[54..56].copy_from_slice(&56u16.to_le_bytes());
        elf[56..58].copy_from_slice(&2u16.to_le_bytes());
        elf[64..68].copy_from_slice(&1u32.to_le_bytes());
        elf[120..124].copy_from_slice(&2u32.to_le_bytes());
        assert!(!interpreted(&elf));
        elf[120..124].copy_from_slice(&3u32.to_le_bytes());
        assert!(interpreted(&elf));
        assert!(!interpreted(b"#!/bin/sh\n"));
    }

    #[test]
    fn a_sigkill_is_not_a_verdict() {
        let dir = std::env::temp_dir().join(format!("gk-killed-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let log = dir.join("make.log");
        std::fs::write(&log, "  LD      vmlinux.o\nmake[3]: *** [/src/scripts/Makefile.vmlinux_o:79: vmlinux.o] Error 137\nmake[2]: *** [/src/Makefile:1362: vmlinux_o] Error 2\n").unwrap();
        assert!(killed(&log));
        std::fs::write(
            &log,
            "make[4]: *** [/src/scripts/Makefile.build:229: fs/x.o] Error 1\n",
        )
        .unwrap();
        assert!(!killed(&log));
        assert!(!killed(&dir.join("missing.log")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_the_first_2_6_makefiles_race_in_scripts() {
        let dir = std::env::temp_dir().join(format!("gk-scripts-race-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let makefile = dir.join("Makefile");
        std::fs::write(&makefile, ".PHONY: scripts scripts/fixdep\nscripts:\n\t$(Q)$(MAKE) $(build)=scripts\n\nscripts/fixdep:\n\t$(Q)$(MAKE) $(build)=scripts $@\n").unwrap();
        assert!(scripts_race(&dir));
        std::fs::write(
            &makefile,
            "scripts_basic:\n\t$(Q)$(MAKE) $(build)=scripts/basic\n",
        )
        .unwrap();
        assert!(!scripts_race(&dir));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!scripts_race(&dir));
    }

    #[test]
    fn goals_race_until_sub_make_takes_them_together() {
        let dir = std::env::temp_dir().join(format!("gk-goals-race-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let makefile = dir.join("Makefile");
        for (rule, races) in [
            ("$(filter-out all,$(MAKECMDGOALS)) all:\n", true),
            ("$(filter-out _all,$(MAKECMDGOALS)) _all:\n", true),
            (
                "$(filter-out _all sub-make $(CURDIR)/Makefile, $(MAKECMDGOALS)) _all: sub-make\n",
                false,
            ),
            ("__sub-make:\n", false),
        ] {
            std::fs::write(&makefile, rule).unwrap();
            assert_eq!(goals_race(&dir), races, "{rule}");
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!goals_race(&dir));
    }

    #[test]
    fn every_configuration_has_a_target_and_allmodconfig_is_build_only() {
        let v: Version = "7.2.8".parse().unwrap();
        for (name, target, _) in CONFIGS {
            let t = config_target(target, "defconfig", &v).unwrap();
            assert!(!t.is_empty(), "{name}");
        }
        assert_eq!(config_named("allmodconfig").unwrap().1, "allmodconfig");
        assert!(BUILD_ONLY.iter().all(|c| config_named(c).is_ok()));
        let old: Version = "3.16".parse().unwrap();
        assert!(config_target("tinyconfig", "defconfig", &old).is_err());
        let museum: Version = "2.4.37.11".parse().unwrap();
        assert_eq!(
            config_target("", "oldconfig", &museum).unwrap(),
            "oldconfig"
        );
        assert!(config_target("allnoconfig", "oldconfig", &museum).is_err());
    }

    #[test]
    fn x86_64_from_2_6_5_to_2_6_8_links_the_i386_pci_headers() {
        let at = |a: &str, v: &str| o_links(a, &v.parse().unwrap());
        assert_eq!(at("x86_64", "2.6.4"), "");
        assert!(at("x86_64", "2.6.5").contains("ln -sf"));
        assert!(at("x86_64", "2.6.8.1").contains("/out/arch/i386/pci"));
        assert_eq!(at("x86_64", "2.6.9"), "");
        assert_eq!(at("i386", "2.6.5"), "");
    }

    #[test]
    fn kernels_before_3_7_settle_with_oldconfig() {
        let at = |v: &str| settle_target(&v.parse().unwrap());
        assert_eq!(at("1.0"), None);
        assert_eq!(at("1.2.13"), Some("oldconfig"));
        assert_eq!(at("2.6.0"), Some("oldconfig"));
        assert_eq!(at("3.6.11"), Some("oldconfig"));
        assert_eq!(at("3.7"), Some("olddefconfig"));
        assert_eq!(at("7.2.8"), Some("olddefconfig"));
    }

    #[test]
    fn splats_the_reference_showed_are_stale() {
        let dir = std::env::temp_dir().join(format!("gk-stale-splats-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let write = |key: &str| {
            let json = format!(r#"{{"objtool": [], "smoke": [[]], "kunit": [["{key}"]]}}"#);
            std::fs::write(dir.join("splats.json"), json).unwrap();
        };
        let allowed = vec![
            "WARNING: drivers/gpu/drm/drm_rect.c:137".to_owned(),
            "WARNING: drivers/gpu/drm/drm_connector.c:232".to_owned(),
        ];
        write("WARNING: at drivers/gpu/drm/drm_rect.c:137 drm_calc_scale");
        assert!(stale_splats(&dir, &allowed).is_some());
        write("WARNING: drivers/gpu/drm/drm_connector.c:234");
        assert!(stale_splats(&dir, &allowed).is_some());
        write("WARNING: kernel/fork.c:12");
        assert!(stale_splats(&dir, &allowed).is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
