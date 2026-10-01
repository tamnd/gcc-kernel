//! `gk probe` and `gk cell`: one kernel, one GCC, one platform, from the pinned tarball up the ladder (spec 07).
//!
//! Every step runs in the kernel's host container with no network. The tree is unpacked once into the cache and mounted read only at `/src`, kbuild writes under `O=/out`, and the bundle is mounted at `/opt/gk/t/<id>`, where `gk forge` installed it. `CROSS_COMPILE` names the bundle's tools on every platform, `x86_64` included, so nothing of the host's own GCC reaches the kernel. `CC` is `gk-cc`, copied into the scratch directory with a `gk-cc.toml` that names the bundle's GCC, so every compiler call kbuild makes lands in `compile.jsonl`.
//!
//! A cell on a platform with an init pin, from 2.6 on, then boots the image with `gk boot` for L5 and runs the smoke suite for L6. Its coordinates then carry the boot container and the initramfs. A cell run with `--no-boot` stops at L4 and keeps the identity it had before booting existed.

use crate::build::{self, Calls};
use crate::fetch::{self, Request};
use crate::forge::{self, Manifest};
use crate::{boot, initramfs, kconfig, net, store, tap};
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
    /// The configuration target.
    pub defconfig: String,
    /// The boot image, relative to `arch/<ARCH>/boot`.
    pub image_name: String,
    /// The six coordinates, and the boot ones once [`Setup::booting`] has run.
    pub coordinates: Coordinates,
    /// Whether the cell boots after L4.
    pub boots: bool,
}

impl Setup {
    /// Resolve and fetch everything a cell needs, which is rung L0.
    pub fn new(repo: &Repo, kernel: &str, gcc: &str, platform: &str) -> Result<Self, String> {
        let version: Version = kernel
            .trim_start_matches("linux-")
            .parse()
            .map_err(|e| format!("kernel {kernel}: {e:?}"))?;
        let k = repo
            .kernels
            .get(&version)
            .ok_or_else(|| format!("{version} is not in kernels.toml"))?;
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
        let host = repo
            .eras
            .host_for(&version)
            .ok_or_else(|| format!("{version} has no host in eras.toml"))?
            .to_owned();
        let arch = p.arch_for(&version).unwrap_or_default().to_owned();
        let defconfig = p.defconfig_for(&version).unwrap_or_default().to_owned();
        let image_name = p.image_for(&version).unwrap_or_default().to_owned();
        let bundle = forge::manifest(&g.id, &p.triple)?;
        let binutils = repo
            .binutils
            .releases
            .iter()
            .find(|b| b.version.as_str() == bundle.binutils)
            .ok_or_else(|| format!("binutils {} is not in binutils.toml", bundle.binutils))?;
        fetch::run(
            repo,
            &Request {
                kernels: vec![version.as_str().to_owned()],
                ..Request::default()
            },
        )?;
        let tree = unpack_tree(
            &fetch::cache_dir().join("kernels").join(k.file_name()),
            &version,
        )?;
        let bundle_dir = forge::unpacked(&bundle)?;
        let image = forge::image_for(repo, &host)?;
        let fragment = repo.root.join(era.fragment());
        let fragment_text =
            std::fs::read(&fragment).map_err(|e| format!("reading {}: {e}", fragment.display()))?;
        let coordinates = Coordinates {
            kernel: Named {
                name: format!("linux-{version}"),
                digest: format!("sha256:{}", k.sha256),
            },
            gcc: Named {
                name: g.id.clone(),
                digest: bundle.digest.clone(),
            },
            binutils: Named {
                name: binutils.id.clone(),
                digest: format!("sha256:{}", binutils.sha256),
            },
            platform: p.name.clone(),
            config: Named {
                name: CONFIG.to_owned(),
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
            image_name,
            coordinates,
            boots: false,
        })
    }

    /// Whether this cell can boot: the platform has an init pin and the kernel is 2.6 or later, which is all `gk-init` covers so far.
    #[must_use]
    pub fn can_boot(&self) -> bool {
        self.platform.init.is_some() && self.version.series(2) >= [2, 6].to_vec()
    }

    /// Make the cell boot, which builds the initramfs if it has to and adds the boot container and the initramfs to the coordinates. A cell that cannot boot is left as it is.
    pub fn booting(mut self, repo: &Repo) -> Result<Self, String> {
        if !self.can_boot() {
            return Ok(self);
        }
        let (init, _) = initramfs::for_platform(repo, &self.platform)?;
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
        cmd.args(["-w", "/out"])
            .arg(&self.image)
            .args([
                "timeout",
                &step.seconds.to_string(),
                "make",
                "-C",
                "/src",
                "O=/out",
            ])
            .arg(format!("ARCH={}", self.arch))
            .arg(format!(
                "CROSS_COMPILE={}/bin/{}-",
                self.bundle_mount(),
                self.bundle.target
            ));
        if step.shim.is_some() {
            cmd.arg("CC=/gk/bin/gk-cc");
        }
        cmd.arg(format!("-j{}", step.jobs)).args(step.args);
        let status = cmd
            .stdout(file)
            .stderr(err)
            .status()
            .map_err(|e| format!("running docker: {e}"))?;
        Ok(status.code().unwrap_or(-1))
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
    let top = partial.join(format!("linux-{version}"));
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

/// The accept probe of spec 09.4: the configuration target, which runs the Kconfig and `cc-version.sh` checks, then `init/main.i`, which runs every `#error` in the compiler headers with kbuild's own flags.
///
/// A refusal is a configuration step that fails or an `#error` directive that fires. Anything else that fails is inconclusive and is left to the build, because `init/main.i` depends on `prepare`, which compiles `bounds.c` and `asm-offsets.c` for real.
pub fn probe(s: &Setup, dir: &Path, jobs: usize) -> Result<Probe, String> {
    let clock = Instant::now();
    let out = dir.join("out");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;
    let seconds = u64::from(s.platform.budget.build_minutes) * 60;
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
        let text = std::fs::read_to_string(&main_log).unwrap_or_default();
        let errors: Vec<String> = text
            .lines()
            .filter(|l| l.contains("error: #error"))
            .take(6)
            .map(str::to_owned)
            .collect();
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

/// Boot a linked cell, three times once it has booted at all, into `boot-1.log` and on. A kernel with KUnit built in boots a second time in each run that passed smoke, into `kunit-1.log` and on, and its suites are graded against the era GCC's cell for L7.
fn boots(
    repo: &Repo,
    s: &Setup,
    image: &Path,
    cell_dir: &Path,
    kunit: bool,
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
        let o = boot::run(
            repo,
            &boot::Boot {
                platform: &s.platform,
                version: &s.version,
                image,
                suite: "smoke",
                dir: cell_dir,
                stem: &stem,
            },
        )?;
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
            let k = boot::run(
                repo,
                &boot::Boot {
                    platform: &s.platform,
                    version: &s.version,
                    image,
                    suite: "kunit",
                    dir: cell_dir,
                    stem: &stem,
                },
            )?;
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
            repo,
            s,
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
                .filter(|k| allowed.is_some_and(|a| !a.contains(k)))
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
        .filter_map(|f| std::fs::read(cell_dir.join(f)).ok())
        .flat_map(|b| {
            String::from_utf8_lossy(&b)
                .lines()
                .filter(|l| l.contains("warning: objtool:"))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .take(100)
        .collect()
}

/// Grade the KUnit runs of a cell that passed smoke in every run against the era GCC's cell, raising each run that passed every graded suite to L7, and write `kunit.json`. Returns the L7 step.
fn grade(
    repo: &Repo,
    s: &Setup,
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
    let (reference, theirs, allowed) = match kunit_reference(repo, s)? {
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

/// Find the era GCC's cell for the same kernel, platform and boot rig in the store.
fn kunit_reference(repo: &Repo, s: &Setup) -> Result<Reference, String> {
    let Some(era) = repo.eras.of(&s.version) else {
        return Ok(Reference::None);
    };
    if era.gcc == s.coordinates.gcc.name {
        return Ok(Reference::Itself);
    }
    let Ok(other) = Setup::new(repo, s.version.as_str(), &era.gcc, &s.platform.name) else {
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
    let scratch = fetch::cache_dir()
        .join("scratch")
        .join(s.coordinates.short_id());
    let _ = std::fs::remove_dir_all(&scratch);
    let _ = std::fs::remove_dir_all(&cell_dir);
    for d in [&cell_dir, &scratch] {
        std::fs::create_dir_all(d).map_err(|e| format!("creating {}: {e}", d.display()))?;
    }
    let (gk, graded) = gk_commit(&repo.root);
    let seconds = u64::from(s.platform.budget.build_minutes) * 60;
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
    std::fs::copy(shim_binary()?, bin.join("gk-cc")).map_err(|e| format!("copying gk-cc: {e}"))?;
    let shim = ShimConfig {
        real: PathBuf::from(s.real_cc()),
        log: PathBuf::from("/out/compile.jsonl"),
        twice: false,
    };
    std::fs::write(bin.join(gk_cc::config::FILE_NAME), shim.to_toml())
        .map_err(|e| format!("writing the shim's settings: {e}"))?;
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
        configured =
            make(&[s.defconfig.as_str()], "config.log", 1)? == 0 && out.join(".config").is_file();
        if configured {
            let text = std::fs::read_to_string(&s.fragment)
                .map_err(|e| format!("reading {}: {e}", s.fragment.display()))?;
            let fragment = kconfig::parse_fragment(&text);
            let dot = out.join(".config");
            let before = std::fs::read_to_string(&dot)
                .map_err(|e| format!("reading {}: {e}", dot.display()))?;
            std::fs::write(&dot, kconfig::merge(&before, &fragment))
                .map_err(|e| format!("writing {}: {e}", dot.display()))?;
            configured = make(&["olddefconfig"], "fragment.log", 1)? == 0;
            fragment_missed = kconfig::missed(&kconfig::load(&dot)?, &fragment);
            let _ = std::fs::copy(&dot, cell_dir.join("config"));
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
    let mut image_sha256 = String::new();
    if configured {
        reached = Rung::Configured;
        let config = kconfig::load(&out.join(".config"))?;
        let modules = config.get("MODULES").is_some_and(|v| v == "y");
        let mut targets = vec![s.image_name.as_str()];
        if modules {
            targets.push("modules");
        }
        let mut args = vec!["V=1"];
        args.extend(&targets);
        let c = Instant::now();
        let code = make(&args, "make.log", jobs)?;
        built = code == 0;
        timed_out = code == 124;
        steps.push(Step {
            rung: Rung::Compiled.to_string(),
            passed: built,
            seconds: c.elapsed().as_secs_f64(),
            log: "make.log.zst".into(),
        });
        if built {
            reached = Rung::Compiled;
            let c = Instant::now();
            let srcarch = match s.arch.as_str() {
                "x86_64" | "i386" => "x86",
                other => other,
            };
            let image = out
                .join("arch")
                .join(srcarch)
                .join("boot")
                .join(&s.image_name);
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
                    let kunit = std::fs::read_to_string(out.join(".config"))
                        .is_ok_and(|c| c.lines().any(|l| l == "CONFIG_KUNIT=y"));
                    let objtool = objtool_warnings(&cell_dir);
                    let b = boots(repo, s, &image, &cell_dir, kunit, &objtool)?;
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
        config: CONFIG.to_owned(),
        era: s.era.clone(),
        gnuc: String::new(),
        std: String::new(),
        source: PathBuf::from("/src"),
        compiler: build::Compiler::of(&s.bundle_dir, &s.bundle.target, &s.real_cc()),
        persona: Vec::new(),
        targets: vec![s.image_name.clone(), "modules".into()],
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
    if log.is_file() {
        let _ = std::fs::copy(&log, cell_dir.join("compile.jsonl"));
    }
    let make_log = cell_dir.join("make.log");
    if make_log.is_file() {
        let status = Command::new("zstd")
            .args(["-q", "-f", "-19", "--rm"])
            .arg(&make_log)
            .status();
        if !status.is_ok_and(|s| s.success()) {
            return Err("compressing make.log failed".into());
        }
    }

    let verdict = Verdict::of(Some(reached), Rung::Clean);
    let record = CellRecord {
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
    };
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
    if let Some(path) = std::env::var_os("GK_SHIM") {
        return Ok(PathBuf::from(path));
    }
    let exe = std::env::current_exe().map_err(|e| format!("finding gk: {e}"))?;
    let shim = exe.with_file_name("gk-cc");
    if shim.is_file() {
        Ok(shim)
    } else {
        Err(format!(
            "gk-cc is not next to gk at {}; build it for musl and copy it there, or set GK_SHIM",
            shim.display()
        ))
    }
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
