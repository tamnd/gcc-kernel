//! `gk boot`: the boot rig of spec 07.4 for kernels from 2.6 on.
//!
//! QEMU runs in the `gk-boot` container with no network, the kernel image and the platform's initramfs mounted read only, and the serial console on stdout. The machine, CPU and console come from `platforms.toml` for the kernel's version, and acceleration is always TCG, so a cell boots the same way on every build machine. Every console line goes to `boot.log` as it arrives, and what the rig read from them goes to `boot.json`.
//!
//! The run ends when QEMU exits, which `gk-init` causes by powering off. It is cut short when the boot budget runs out, and when the end marker has been printed and the console then stays quiet for five seconds, which is how a kernel that cannot power off is told apart from a hung one.

use crate::{forge, initramfs, tap};
use gk_model::Version;
use gk_model::platforms::Platform;
use gk_model::repo::Repo;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The memory every boot gets, in MB. Enough for the 64 MB check and a defconfig kernel on every tier 1 platform.
pub const MEMORY: u32 = 1024;

/// The CPUs every boot gets. Two, so the kernel brings up a second CPU and the CPU count check means something.
pub const CPUS: u32 = 2;

/// How long the console may stay quiet after the end marker before the rig stops QEMU itself.
const QUIET_AFTER_END: Duration = Duration::from_secs(5);

/// The start of a line that means the kernel has died or is about to.
const PANICS: [&str; 6] = [
    "Kernel panic",
    "Oops:",
    "Oops ",
    "BUG:",
    "general protection fault",
    "Unable to handle kernel",
];

/// The start of a line that means the kernel complained but went on, which L8 grades.
const SPLATS: [&str; 6] = [
    "WARNING:",
    "UBSAN:",
    "KASAN:",
    "BUG:",
    "INFO: task ",
    "rcu: INFO:",
];

/// One boot to run.
pub struct Boot<'a> {
    /// The platform.
    pub platform: &'a Platform,
    /// The kernel's version, which picks the machine, CPU and console, and which `gk-init` checks `uname` against.
    pub version: &'a Version,
    /// The kernel image.
    pub image: &'a Path,
    /// The suite `gk-init` runs.
    pub suite: &'a str,
    /// Where the console log and the outcome go.
    pub dir: &'a Path,
    /// Their file name without the extension, as `boot` for `boot.log` and `boot.json`.
    pub stem: &'a str,
}

/// One `GK-CHECK` line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Check {
    /// The check's name.
    pub name: String,
    /// Whether it passed.
    pub pass: bool,
}

/// What the rig read from one boot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    /// The QEMU command, as run in the container.
    pub command: Vec<String>,
    /// The boot container, by digest.
    pub qemu: String,
    /// The initramfs, by digest.
    pub initramfs: String,
    /// `tcg`, which is all the build machines have.
    pub accel: String,
    /// The release `GK-BOOTED` reported, once init has run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub booted: Option<String>,
    /// The suite named by `GK-BEGIN`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite: Option<String>,
    /// Every check, in the order they ran.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<Check>,
    /// The status `GK-END` reported, `pass` or `fail`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The first line that says the kernel died.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub panic: Option<String>,
    /// Every line that says the kernel complained.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub splats: Vec<String>,
    /// The KUnit suites that ran before init, for the `kunit` suite.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kunit: Vec<tap::Suite>,
    /// Whether the budget ran out.
    pub timed_out: bool,
    /// Whether the rig stopped QEMU after the end marker because the kernel did not power off.
    pub stopped_after_end: bool,
    /// How long it took, in seconds.
    pub seconds: f64,
}

impl Outcome {
    fn new(command: Vec<String>, qemu: String, initramfs: String) -> Self {
        Outcome {
            command,
            qemu,
            initramfs,
            accel: "tcg".into(),
            booted: None,
            suite: None,
            checks: Vec::new(),
            status: None,
            panic: None,
            splats: Vec::new(),
            kunit: Vec::new(),
            timed_out: false,
            stopped_after_end: false,
            seconds: 0.0,
        }
    }

    /// Whether init ran, which is L5.
    #[must_use]
    pub fn booted(&self) -> bool {
        self.booted.is_some()
    }

    /// Whether the suite ended with every check passed and the kernel never died, which is L6 for `smoke`.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.status.as_deref() == Some("pass")
            && self.panic.is_none()
            && !self.checks.is_empty()
            && self.checks.iter().all(|c| c.pass)
    }

    /// Whether the end marker has been read.
    #[must_use]
    pub fn ended(&self) -> bool {
        self.status.is_some()
    }

    /// Take one console line, without its line ending.
    pub fn read_line(&mut self, raw: &str) {
        let line = strip_timestamp(raw.trim_end_matches(['\r', '\n']));
        if let Some(at) = line.find("GK-") {
            // The kernel can print into the middle of a marker line, as in `GK-CHECK exec pass[    6.65] init (81) used greatest stack depth`. Markers never hold a `[`, so the marker ends there and the rest is a console line of its own.
            let mut marker = &line[at..];
            if let Some(cut) = marker.find('[') {
                self.read_line(&marker[cut..]);
                marker = &marker[..cut];
            }
            let mut words = marker.split_whitespace();
            match words.next() {
                Some("GK-BOOTED") if self.booted.is_none() => {
                    self.booted = Some(words.next().unwrap_or_default().to_owned());
                }
                Some("GK-BEGIN") if self.suite.is_none() => {
                    self.suite = words.next().map(str::to_owned);
                }
                Some("GK-CHECK") => {
                    if let (Some(name), Some(result)) = (words.next(), words.next()) {
                        self.checks.push(Check {
                            name: name.to_owned(),
                            pass: result == "pass",
                        });
                    }
                }
                Some("GK-END") if self.status.is_none() => {
                    let _suite = words.next();
                    self.status = words.next().map(str::to_owned);
                }
                _ => {}
            }
            return;
        }
        if self.panic.is_none() && PANICS.iter().any(|p| line.starts_with(p)) {
            self.panic = Some(line.to_owned());
        }
        if SPLATS.iter().any(|p| line.starts_with(p)) {
            self.splats.push(line.to_owned());
        }
    }
}

/// What two splats from different boots share when they are the same complaint: the line without the CPU, the task, and the offsets into functions, which move between builds.
#[must_use]
pub fn splat_key(line: &str) -> String {
    // 6.x ends the line with `, CPU#1: task/123`, and older kernels put `CPU: 1 PID: 123` after the kind.
    let line = line.split(", CPU#").next().unwrap_or(line);
    let mut words = Vec::new();
    let mut it = line.split_whitespace().peekable();
    while let Some(w) = it.next() {
        if (w == "CPU:" || w == "PID:")
            && it
                .peek()
                .is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit()))
        {
            it.next();
            continue;
        }
        words.push(w.split("+0x").next().unwrap_or(w));
    }
    // A warning is known by its file and line. The function it names is whatever the line was inlined into, so gcc-8.5.0 says drm_calc_scale where gcc-14.2.0 says drm_rect_calc_hscale for the same WARN_ON.
    if words.first() == Some(&"WARNING:")
        && let Some(at) = words.iter().find(|w| is_source_line(w))
    {
        return format!("WARNING: {at}");
    }
    words.join(" ")
}

/// Whether a word is a `file.c:123` source location.
fn is_source_line(word: &str) -> bool {
    word.rsplit_once(':').is_some_and(|(file, line)| {
        !line.is_empty()
            && line.bytes().all(|b| b.is_ascii_digit())
            && [".c", ".h", ".rs", ".S"].iter().any(|e| file.ends_with(e))
    })
}

/// A console line without the kernel's `[    1.234567] ` time stamp.
#[must_use]
pub fn strip_timestamp(line: &str) -> &str {
    let Some(rest) = line.strip_prefix('[') else {
        return line;
    };
    match rest.split_once("] ") {
        Some((stamp, after))
            if !stamp.is_empty()
                && stamp
                    .trim_start()
                    .bytes()
                    .all(|b| b.is_ascii_digit() || b == b'.') =>
        {
            after
        }
        _ => line,
    }
}

/// The kernel command line for a boot.
///
/// The fragments build every KUnit suite in, and they run before init, which under TCG takes longer than the whole boot budget. So every suite but `kunit` turns them off: `kunit.enable=0` from 6.2, and a filter that matches no suite from 5.10 to 6.1. A kernel ignores the one it does not know.
#[must_use]
pub fn append(console: &str, suite: &str, version: &Version) -> String {
    let quiet = if suite == "kunit" {
        ""
    } else {
        " kunit.enable=0 kunit.filter_glob=gk-none"
    };
    format!(
        "console={console} panic=-1 oops=panic gk.suite={suite} gk.kernel={version} gk.cpus={CPUS}{quiet}"
    )
}

/// The QEMU command, with the kernel and the initramfs at the paths the container mounts them on.
pub fn qemu_command(p: &Platform, version: &Version, suite: &str) -> Result<Vec<String>, String> {
    let missing = |what: &str| format!("platforms.toml has no {what} for {version} on {}", p.name);
    let machine = p.machine_for(version).ok_or_else(|| missing("machine"))?;
    let cpu = p.cpu_for(version).ok_or_else(|| missing("cpu"))?;
    let console = p.console_for(version).ok_or_else(|| missing("console"))?;
    // An empty CPU is a machine with one CPU model, where QEMU takes no -cpu.
    let cpu: &[&str] = if cpu.is_empty() { &[] } else { &["-cpu", cpu] };
    Ok([p.qemu.as_str(), "-machine", machine]
        .iter()
        .chain(cpu)
        .chain(&[
            "-accel",
            "tcg",
            "-smp",
            &CPUS.to_string(),
            "-m",
            &MEMORY.to_string(),
            "-kernel",
            "/boot/kernel",
            "-initrd",
            "/boot/initramfs.cpio",
            "-append",
            &append(console, suite, version),
            "-nographic",
            "-monitor",
            "none",
            "-nic",
            "none",
            "-no-reboot",
        ])
        .map(|s| (*s).to_owned())
        .collect())
}

/// What the reader thread passes back.
enum Read {
    Line(String),
    Closed,
}

/// Boot a kernel once and write `<stem>.log` and `<stem>.json` under `b.dir`.
pub fn run(repo: &Repo, b: &Boot<'_>) -> Result<Outcome, String> {
    let (init, archive) = initramfs::for_kernel(repo, b.platform, b.version)?;
    let image = forge::image_for(repo, "gk-boot")?;
    let qemu = crate::cell::image_digest(&image)?;
    let command = qemu_command(b.platform, b.version, b.suite)?;
    let kernel = std::fs::canonicalize(b.image)
        .map_err(|e| format!("reading {}: {e}", b.image.display()))?;
    std::fs::create_dir_all(b.dir).map_err(|e| format!("creating {}: {e}", b.dir.display()))?;
    let log_path = b.dir.join(format!("{}.log", b.stem));
    let mut log = std::fs::File::create(&log_path)
        .map_err(|e| format!("creating {}: {e}", log_path.display()))?;
    let name = format!(
        "gk-boot-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    );
    let mut outcome = Outcome::new(command.clone(), qemu, init.digest);
    let clock = Instant::now();
    let budget = Duration::from_secs(u64::from(if b.suite == "kunit" {
        b.platform.budget.kunit_seconds()
    } else {
        b.platform.budget.boot_seconds
    }));
    let mut child = Command::new("docker")
        .args(["run", "--rm", "--network=none", "--name", &name])
        .arg("-v")
        .arg(format!("{}:/boot/kernel:ro", kernel.display()))
        .arg("-v")
        .arg(format!("{}:/boot/initramfs.cpio:ro", archive.display()))
        .arg(&image)
        .args(&command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("running docker: {e}"))?;
    watch(&mut child, &name, budget, clock, &mut log, &mut outcome);
    outcome.seconds = clock.elapsed().as_secs_f64();
    if b.suite == "kunit" {
        let text =
            std::fs::read(&log_path).map_err(|e| format!("reading {}: {e}", log_path.display()))?;
        outcome.kunit = tap::suites(&String::from_utf8_lossy(&text));
    }
    let json_path = b.dir.join(format!("{}.json", b.stem));
    let text = serde_json::to_string_pretty(&outcome).map_err(|e| e.to_string())? + "\n";
    std::fs::write(&json_path, text)
        .map_err(|e| format!("writing {}: {e}", json_path.display()))?;
    Ok(outcome)
}

/// Read the console of a running QEMU into `log` and `outcome` until it exits, stopping it when the budget runs out or when it stays quiet after the end marker.
fn watch(
    child: &mut std::process::Child,
    name: &str,
    budget: Duration,
    clock: Instant,
    log: &mut std::fs::File,
    outcome: &mut Outcome,
) {
    let (tx, rx) = mpsc::channel();
    let mut readers = Vec::new();
    for stream in [
        child
            .stdout
            .take()
            .map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
        child
            .stderr
            .take()
            .map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
    ]
    .into_iter()
    .flatten()
    {
        let tx = tx.clone();
        readers.push(std::thread::spawn(move || {
            let mut reader = BufReader::new(stream);
            let mut buf = Vec::new();
            while reader.read_until(b'\n', &mut buf).is_ok_and(|n| n > 0) {
                let _ = tx.send(Read::Line(String::from_utf8_lossy(&buf).into_owned()));
                buf.clear();
            }
            let _ = tx.send(Read::Closed);
        }));
    }
    drop(tx);
    let mut open = readers.len();
    let mut last_line = Instant::now();
    while open > 0 {
        let left = budget.saturating_sub(clock.elapsed());
        let wait = if outcome.ended() {
            left.min(QUIET_AFTER_END.saturating_sub(last_line.elapsed()))
        } else {
            left
        };
        match rx.recv_timeout(wait) {
            Ok(Read::Line(line)) => {
                let _ = log.write_all(line.as_bytes());
                outcome.read_line(&line);
                last_line = Instant::now();
            }
            Ok(Read::Closed) => open -= 1,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if outcome.ended() {
                    outcome.stopped_after_end = true;
                } else {
                    outcome.timed_out = true;
                }
                let _ = Command::new("docker")
                    .args(["kill", name])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                break;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let _ = child.wait();
    for r in readers {
        let _ = r.join();
    }
    while let Ok(Read::Line(line)) = rx.try_recv() {
        let _ = log.write_all(line.as_bytes());
        outcome.read_line(&line);
    }
}

/// Run `gk boot IMAGE --platform P --kernel K [--suite S] [--dir D]`. Returns whether the suite passed.
pub fn command(repo: &Repo, args: &[String]) -> Result<bool, String> {
    let (mut image, mut platform, mut kernel, mut suite, mut dir) =
        (None, None, None, "smoke".to_owned(), PathBuf::from("."));
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--platform" | "--kernel" | "--suite" | "--dir" => {
                let v = it.next().ok_or_else(|| format!("{a} needs a value"))?;
                match a.as_str() {
                    "--platform" => platform = Some(v.clone()),
                    "--kernel" => kernel = Some(v.clone()),
                    "--suite" => suite.clone_from(v),
                    _ => dir = PathBuf::from(v),
                }
            }
            other if !other.starts_with('-') && image.is_none() => image = Some(other.to_owned()),
            other => return Err(format!("gk boot: unexpected {other:?}")),
        }
    }
    let usage = "usage: gk boot IMAGE --platform P --kernel K [--suite S] [--dir D]";
    let (Some(image), Some(platform), Some(kernel)) = (image, platform, kernel) else {
        return Err(usage.into());
    };
    let p = repo
        .platforms
        .get(&platform)
        .ok_or_else(|| format!("platform {platform} is not in platforms.toml"))?;
    let version: Version = kernel
        .trim_start_matches("linux-")
        .parse()
        .map_err(|e| format!("kernel {kernel}: {e:?}"))?;
    let o = run(
        repo,
        &Boot {
            platform: p,
            version: &version,
            image: Path::new(&image),
            suite: &suite,
            dir: &dir,
            stem: "boot",
        },
    )?;
    let passed = o.checks.iter().filter(|c| c.pass).count();
    println!(
        "{}: {} in {:.0}s, {passed} of {} checks passed{}{}{}",
        if o.booted() {
            o.booted.as_deref().unwrap_or_default()
        } else {
            "did not boot"
        },
        o.status.as_deref().unwrap_or("no end marker"),
        o.seconds,
        o.checks.len(),
        if o.timed_out { ", out of time" } else { "" },
        o.panic
            .as_deref()
            .map(|l| format!(", {l}"))
            .unwrap_or_default(),
        if o.splats.is_empty() {
            String::new()
        } else {
            format!(", {} splats", o.splats.len())
        },
    );
    for c in o.checks.iter().filter(|c| !c.pass) {
        println!("  {} failed", c.name);
    }
    Ok(o.passed())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(lines: &[&str]) -> Outcome {
        let mut o = Outcome::new(Vec::new(), String::new(), String::new());
        for l in lines {
            o.read_line(l);
        }
        o
    }

    #[test]
    fn time_stamps_come_off() {
        assert_eq!(strip_timestamp("[    1.234567] Oops: 0000"), "Oops: 0000");
        assert_eq!(strip_timestamp("[12345.000001] x"), "x");
        assert_eq!(strip_timestamp("[gk] not a stamp"), "[gk] not a stamp");
        assert_eq!(strip_timestamp("GK-BOOTED 7.2.8"), "GK-BOOTED 7.2.8");
    }

    #[test]
    fn a_clean_boot_passes() {
        let o = read(&[
            "[    0.000000] Linux version 7.2.8\r\n",
            "GK-BOOTED 7.2.8\r\n",
            "GK-BEGIN smoke\r\n",
            "GK-CHECK fork-wait pass\r\n",
            "GK-CHECK pipe pass\r\n",
            "GK-END smoke pass 0\r\n",
            "[    3.100000] reboot: Power down\r\n",
        ]);
        assert_eq!(o.booted.as_deref(), Some("7.2.8"));
        assert_eq!(o.suite.as_deref(), Some("smoke"));
        assert_eq!(o.checks.len(), 2);
        assert!(o.booted() && o.ended() && o.passed());
    }

    #[test]
    fn a_failed_check_or_a_panic_fails() {
        let o = read(&[
            "GK-BOOTED 7.2.8",
            "GK-BEGIN smoke",
            "GK-CHECK mmap-64m fail",
            "GK-END smoke fail 1",
        ]);
        assert!(o.booted() && !o.passed());
        let o = read(&[
            "GK-BOOTED 7.2.8",
            "GK-BEGIN smoke",
            "GK-CHECK pipe pass",
            "[    2.000000] Kernel panic - not syncing: Attempted to kill init!",
            "GK-END smoke pass 0",
        ]);
        assert!(!o.passed());
        assert!(o.panic.unwrap().starts_with("Kernel panic"));
    }

    #[test]
    fn a_marker_after_kernel_noise_still_counts() {
        let o = read(&["random: crng init doneGK-BOOTED 7.2.8"]);
        assert_eq!(o.booted.as_deref(), Some("7.2.8"));
    }

    #[test]
    fn splats_are_kept() {
        let o = read(&[
            "[    1.0] WARNING: CPU: 0 PID: 1 at mm/slub.c:123 f+0x1/0x2",
            "[    1.1] UBSAN: shift-out-of-bounds in lib/x.c:5:3",
            "[    1.2] Call Trace:",
        ]);
        assert_eq!(o.splats.len(), 2);
        assert!(o.panic.is_none());
    }

    #[test]
    fn the_command_line_names_the_suite_and_the_kernel() {
        let v: Version = "7.2.8".parse().unwrap();
        assert_eq!(
            append("ttyS0", "smoke", &v),
            "console=ttyS0 panic=-1 oops=panic gk.suite=smoke gk.kernel=7.2.8 gk.cpus=2 kunit.enable=0 kunit.filter_glob=gk-none"
        );
    }

    #[test]
    fn a_kernel_line_inside_a_marker_is_cut_off() {
        let o = read(&[
            "GK-CHECK exec pass[    6.653663] init (81) used greatest stack depth: 13792 bytes left",
            "GK-CHECK pipe pass[    7.1] WARNING: CPU: 0 PID: 1 at lib/x.c:3 f+0x1/0x2",
        ]);
        assert!(o.checks.iter().all(|c| c.pass));
        assert_eq!(o.checks.len(), 2);
        assert_eq!(o.splats.len(), 1);
    }

    #[test]
    fn splats_from_two_boots_match_by_key() {
        assert_eq!(
            splat_key(
                "WARNING: lib/math/int_log.c:63 at intlog2+0x55/0x60, CPU#0: kunit_try_catch/607"
            ),
            "WARNING: lib/math/int_log.c:63"
        );
        assert_eq!(
            splat_key("WARNING: CPU: 1 PID: 42 at kernel/fork.c:12 copy_process+0x1/0x2 [foo]"),
            "WARNING: kernel/fork.c:12"
        );
        assert_eq!(
            splat_key("WARNING: at drivers/gpu/drm/drm_rect.c:137 drm_calc_scale+0x2a/0x40"),
            splat_key("WARNING: at drivers/gpu/drm/drm_rect.c:137 drm_rect_calc_hscale+0x1c/0x50"),
        );
        assert_eq!(
            splat_key("WARNING: lib/math/int_log.c:63 at intlog2"),
            "WARNING: lib/math/int_log.c:63"
        );
        assert_eq!(
            splat_key("UBSAN: shift-out-of-bounds in lib/x.c:3:9"),
            "UBSAN: shift-out-of-bounds in lib/x.c:3:9"
        );
    }
}
