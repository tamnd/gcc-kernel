//! `gk boot`: the boot rig of spec 07.4.
//!
//! QEMU runs in the `gk-boot` container with no network, the kernel image and the platform's initramfs mounted read only, and the serial console on stdout. The machine, CPU and console come from `platforms.toml` for the kernel's version, and acceleration is always TCG, so a cell boots the same way on every build machine. Every console line goes to `boot.log` as it arrives, and what the rig read from them goes to `boot.json`.
//!
//! The run ends when QEMU exits, which `gk-init` causes by powering off. It is cut short when the boot budget runs out, and when the end marker has been printed and the console then stays quiet for five seconds, which is how a kernel that cannot power off is told apart from a hung one.
//!
//! Kernels before 2.6 boot the museum way: one CPU, 32 MB, and `gk-init-museum` on a Minix root image, which is the initrd from 1.3.73 and the second IDE disk before. Kernels older than boot protocol 2.03 start from a SYSLINUX disk that `gk-syslinux` writes in the container, because the `-kernel` loader of QEMU writes a 2.03 field over their setup code. Before 2.1.25 there is no serial console, so QEMU runs with its monitor on stdin and stdout, the rig asks it for the VGA text screen once a second, and the lines come from the screens (spec 07.6).

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
    /// The checks the kernel has nothing for, whose failure does not count, from [`excused`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excused: Vec<String>,
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
    /// The start of a marker the kernel printed into, until the rest of it comes.
    #[serde(skip)]
    cut: Option<String>,
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
            excused: Vec::new(),
            status: None,
            panic: None,
            splats: Vec::new(),
            kunit: Vec::new(),
            timed_out: false,
            stopped_after_end: false,
            seconds: 0.0,
            cut: None,
        }
    }

    /// Whether init ran, which is L5.
    #[must_use]
    pub fn booted(&self) -> bool {
        self.booted.is_some()
    }

    /// Whether the suite ended with every check passed but the excused ones and the kernel never died, which is L6 for `smoke`.
    #[must_use]
    pub fn passed(&self) -> bool {
        let counts = |c: &Check| !self.excused.contains(&c.name);
        let status_ok = match self.status.as_deref() {
            Some("pass") => true,
            // init says fail when any check failed, so a fail is a pass when every check that failed is excused.
            Some("fail") => self.checks.iter().any(|c| !c.pass),
            _ => false,
        };
        status_ok
            && self.panic.is_none()
            && !self.checks.is_empty()
            && self.checks.iter().all(|c| c.pass || !counts(c))
    }

    /// Read a cut marker whose rest never came, as when the console stops right after it.
    pub fn finish(&mut self) {
        if let Some(head) = self.cut.take() {
            self.read_line(&head);
        }
    }

    /// Whether the end marker has been read.
    #[must_use]
    pub fn ended(&self) -> bool {
        self.status.is_some()
    }

    /// Take one console line, without its line ending.
    pub fn read_line(&mut self, raw: &str) {
        let raw = raw.trim_end_matches(['\r', '\n']);
        let line = strip_timestamp(raw);
        // The rest of a cut marker is the first line after it that is not the kernel's, as `s 0` after `GK-END smoke pas[    9.55] md: stopping all md devices.`. A line with a marker of its own means the cut was at the end of the old one.
        if line.len() == raw.len()
            && let Some(head) = self.cut.take()
        {
            if raw.contains("GK-") {
                self.read_line(&head);
            } else {
                self.read_line(&format!("{head}{raw}"));
                return;
            }
        }
        if let Some(at) = line.find("GK-") {
            // The kernel can print into the middle of a marker line, as in `GK-CHECK exec pass[    6.65] init (81) used greatest stack depth`. Markers never hold a `[`, so the marker ends there and the rest is a console line of its own. What init wrote after the cut, up to its own line end, comes on a line of its own after the kernel's, so the marker waits for it.
            let marker = &line[at..];
            if let Some(cut) = marker.find('[') {
                self.read_line(&marker[cut..]);
                self.cut = Some(marker[..cut].to_owned());
                return;
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

/// The smoke checks a kernel has nothing for. `sysfs` opens `/sys/kernel`, which 2.6.10 added, so before it the check fails on every column alike.
#[must_use]
pub fn excused(version: &Version) -> Vec<String> {
    if !initramfs::museum(version) && version.series(3) < [2, 6, 10].to_vec() {
        vec!["sysfs".into()]
    } else {
        Vec::new()
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

/// The memory of a museum boot, in MB. Kernels before 2.2 size memory with a BIOS call that stops at 64 MB, and 1.x tests every page at boot.
pub const MUSEUM_MEMORY: u32 = 32;

/// Where the container mounts what the kernel boots from: the initramfs from 2.6.6 on, and a root image before.
#[must_use]
pub fn root_path(version: &Version) -> &'static str {
    if initramfs::museum(version) || initramfs::mounts_root(version) {
        "/boot/root.img"
    } else {
        "/boot/initramfs.cpio"
    }
}

/// Whether a kernel has the boot protocol of 1.3.73 and later, which takes an initrd. Before it the root image goes on the second IDE disk, after the boot loader's.
fn takes_initrd(version: &Version) -> bool {
    version.series(3) >= [1, 3, 73].to_vec()
}

/// The boot protocol of an x86 kernel image, from its setup header, or 0 for the images before 1.3.73 that have no header.
#[must_use]
pub fn protocol(image: &[u8]) -> u16 {
    match image.get(0x202..0x208) {
        Some([b'H', b'd', b'r', b'S', lo, hi]) => u16::from_le_bytes([*lo, *hi]),
        _ => 0,
    }
}

/// Whether a kernel boots from a SYSLINUX disk. The `-kernel` loader of QEMU writes `initrd_addr_max` at offset 0x22c of the setup code whatever the protocol, and before protocol 2.03 that offset holds code, so those kernels die on an invalid opcode before they print anything.
#[must_use]
pub fn needs_loader(protocol: u16) -> bool {
    protocol < 0x203
}

/// Whether the rig reads the console from the VGA text screen, for kernels with no serial console (spec 07.6).
#[must_use]
pub fn vga(p: &Platform, version: &Version) -> bool {
    p.console_for(version) == Some("vga")
}

/// The kernel command line of a museum boot. Old kernels hand the words with an `=` they do not know to init as its environment, which is where `gk-init-museum` reads the suite and the kernel from.
///
/// From 2.1 the SMP kernels of the era find the IO-APIC in the MP table that the BIOS writes and route the timer and the disks through it, which QEMU does not wire the way they expect, so the boot turns it off with `noapic` and everything goes through the 8259.
#[must_use]
pub fn museum_append(console: &str, suite: &str, version: &Version) -> String {
    let root = if takes_initrd(version) {
        "/dev/ram0"
    } else {
        "/dev/hdb"
    };
    // `no-scroll` keeps a VGA console from scrolling by moving the screen through video memory, from 2.0. Older kernels hand it to init, which ignores it.
    let console = if console == "vga" {
        "no-scroll ".to_owned()
    } else {
        format!("console={console} ")
    };
    let noapic = if version.series(2) >= [2, 1].to_vec() {
        " noapic"
    } else {
        ""
    };
    format!("{console}root={root} rw panic=-1{noapic} gk.suite={suite} gk.kernel={version}")
}

/// The QEMU command, with the kernel and the initramfs at the paths the container mounts them on. `protocol` is the kernel's boot protocol, which picks the loader of a museum boot.
pub fn qemu_command(
    p: &Platform,
    version: &Version,
    suite: &str,
    protocol: u16,
) -> Result<Vec<String>, String> {
    let missing = |what: &str| format!("platforms.toml has no {what} for {version} on {}", p.name);
    let machine = p.machine_for(version).ok_or_else(|| missing("machine"))?;
    let cpu = p.cpu_for(version).ok_or_else(|| missing("cpu"))?;
    let console = p.console_for(version).ok_or_else(|| missing("console"))?;
    // An empty CPU is a machine with one CPU model, where QEMU takes no -cpu.
    let cpu: &[&str] = if cpu.is_empty() { &[] } else { &["-cpu", cpu] };
    let mut command: Vec<String> = [p.qemu.as_str(), "-machine", machine]
        .iter()
        .chain(cpu)
        .chain(&["-accel", "tcg"])
        .map(|s| (*s).to_owned())
        .collect();
    let rest: Vec<String> = if initramfs::museum(version) {
        let append = museum_append(console, suite, version);
        // One CPU: the SMP kernels of the era want an MP table that the machines they ran on had.
        let mut rest = vec![
            "-smp".into(),
            "1".into(),
            "-m".into(),
            MUSEUM_MEMORY.to_string(),
        ];
        if !takes_initrd(version) {
            // The image is mounted read only and the smoke suite writes to it, so the writes go to a scratch overlay.
            rest.extend([
                "-drive".into(),
                format!(
                    "file={},format=raw,if=ide,index=1,snapshot=on",
                    root_path(version)
                ),
            ]);
        }
        if needs_loader(protocol) {
            // gk-syslinux writes the boot disk, adds it as the first IDE drive and runs the rest.
            let initrd = if takes_initrd(version) {
                root_path(version)
            } else {
                "-"
            };
            command.splice(0..0, ["gk-syslinux".into(), append, initrd.into()]);
        } else {
            rest.extend(["-kernel".into(), "/boot/kernel".into()]);
            if takes_initrd(version) {
                rest.extend(["-initrd".into(), root_path(version).into()]);
            }
            rest.extend(["-append".into(), append]);
        }
        if console == "vga" {
            // The monitor takes `pmemsave` on stdin and the screen is read from memory.
            rest.extend(
                ["-display", "none", "-serial", "none", "-monitor", "stdio"].map(String::from),
            );
        } else {
            rest.extend(["-nographic", "-monitor", "none"].map(String::from));
        }
        rest.extend(["-nic", "none", "-no-reboot"].map(String::from));
        rest
    } else {
        let mut append = append(console, suite, version);
        let root: [String; 2] = if initramfs::mounts_root(version) {
            // The root disk is mounted read only and the kernel writes to it, so the writes go to a scratch overlay.
            append.push_str(" root=/dev/hda rw init=/init");
            [
                "-drive".into(),
                format!(
                    "file={},format=raw,if=ide,index=0,snapshot=on",
                    root_path(version)
                ),
            ]
        } else {
            ["-initrd".into(), root_path(version).into()]
        };
        [
            "-smp",
            &CPUS.to_string(),
            "-m",
            &MEMORY.to_string(),
            "-kernel",
            "/boot/kernel",
            &root[0],
            &root[1],
            "-append",
            &append,
            "-nographic",
            "-monitor",
            "none",
            "-nic",
            "none",
            "-no-reboot",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect()
    };
    command.extend(rest);
    Ok(command)
}

/// The VGA text screen: 25 rows of 80 cells, each a character and an attribute, at 0xb8000.
pub const SCREEN_BYTES: usize = 80 * 25 * 2;

/// The rows of a VGA text screen, as text with the trailing blanks taken off. Anything that is not printable ASCII is a blank.
#[must_use]
pub fn screen_rows(bytes: &[u8]) -> Vec<String> {
    bytes
        .chunks(160)
        .take(25)
        .map(|row| {
            row.chunks(2)
                .map(|cell| match cell[0] {
                    c @ 0x21..=0x7e => c as char,
                    _ => ' ',
                })
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect()
}

/// Turns successive VGA screens into console lines. The screen scrolls, so each new screen is matched against the last one by how far it moved, and a row is taken once a row below it has been written or it has stayed the same for one capture, so a line is never read half written.
#[derive(Debug, Default)]
pub struct Screen {
    rows: Vec<String>,
    taken: usize,
}

impl Screen {
    /// Take one capture, and return the lines it finished. `last` takes every row, for the final capture.
    pub fn take(&mut self, rows: Vec<String>, last: bool) -> Vec<String> {
        let n = rows.len();
        let mut prev = std::mem::take(&mut self.rows);
        prev.resize(n, String::new());
        // How far the screen scrolled: the least shift under which every old row is blank, the same, or the start of the new one. A cleared screen matches only far enough down to be all blank.
        let shift = (0..=n)
            .find(|&s| {
                (0..n - s).all(|i| {
                    let (a, b) = (&prev[i + s], &rows[i]);
                    a.is_empty() || b.starts_with(a.as_str())
                })
            })
            .unwrap_or(n);
        let taken = self.taken.saturating_sub(shift);
        let filled = rows
            .iter()
            .rposition(|r| !r.is_empty())
            .map_or(0, |i| i + 1);
        let settled = filled > 0
            && (last
                || prev
                    .get(filled - 1 + shift)
                    .is_some_and(|r| *r == rows[filled - 1]));
        let done = if settled {
            filled
        } else {
            filled.saturating_sub(1)
        };
        let out = if done > taken {
            rows[taken..done].to_vec()
        } else {
            Vec::new()
        };
        self.taken = done.max(taken);
        self.rows = rows;
        out
    }
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
    let kernel = std::fs::canonicalize(b.image)
        .map_err(|e| format!("reading {}: {e}", b.image.display()))?;
    let head = std::fs::read(&kernel).map_err(|e| format!("reading {}: {e}", kernel.display()))?;
    let command = qemu_command(b.platform, b.version, b.suite, protocol(&head))?;
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
    outcome.excused = excused(b.version);
    let clock = Instant::now();
    let budget = Duration::from_secs(u64::from(if b.suite == "kunit" {
        b.platform.budget.kunit_seconds()
    } else {
        b.platform.budget.boot_seconds
    }));
    let vga = vga(b.platform, b.version);
    let screen_dir = b.dir.join(format!("{}-vga", b.stem));
    let mut docker = Command::new("docker");
    docker
        .args(["run", "--rm", "--network=none", "--name", &name])
        .arg("-v")
        .arg(format!("{}:/boot/kernel:ro", kernel.display()))
        .arg("-v")
        .arg(format!("{}:{}:ro", archive.display(), root_path(b.version)));
    if vga {
        std::fs::create_dir_all(&screen_dir)
            .map_err(|e| format!("creating {}: {e}", screen_dir.display()))?;
        docker
            .arg("-i")
            .arg("-v")
            .arg(format!("{}:/vga", screen_dir.display()))
            .stdin(Stdio::piped());
    } else {
        docker.stdin(Stdio::null());
    }
    let mut child = docker
        .arg(&image)
        .args(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("running docker: {e}"))?;
    let capture = if vga {
        child.stdin.take().map(|stdin| (stdin, screen_dir.clone()))
    } else {
        None
    };
    watch(
        &mut child,
        &name,
        budget,
        clock,
        &mut log,
        &mut outcome,
        capture,
    );
    if vga {
        let _ = std::fs::remove_dir_all(&screen_dir);
    }
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

/// Read the screen of a QEMU with a VGA console once a second, through `pmemsave` on its monitor, and pass on the lines it finishes. It stops when the monitor is gone, after one last read of the screen.
fn capture(mut monitor: std::process::ChildStdin, dir: &Path, tx: &mpsc::Sender<Read>) {
    let file = dir.join("screen");
    let mut screen = Screen::default();
    let mut send = |bytes: &[u8], last: bool| {
        if bytes.len() == SCREEN_BYTES {
            for line in screen.take(screen_rows(bytes), last) {
                let _ = tx.send(Read::Line(line + "\n"));
            }
        }
    };
    loop {
        // The file name is quoted, or the monitor reads `4000 /vga/screen` as a division.
        let asked = monitor
            .write_all(format!("pmemsave 0xb8000 {SCREEN_BYTES} \"/vga/screen\"\n").as_bytes())
            .and_then(|()| monitor.flush());
        std::thread::sleep(Duration::from_secs(1));
        let bytes = std::fs::read(&file).unwrap_or_default();
        if asked.is_err() {
            send(&bytes, true);
            break;
        }
        send(&bytes, false);
    }
    let _ = tx.send(Read::Closed);
}

/// Read the console of a running QEMU into `log` and `outcome` until it exits, stopping it when the budget runs out or when it stays quiet after the end marker. With a VGA console, the lines come from the screen and what QEMU prints on stdout is its monitor, which is dropped.
fn watch(
    child: &mut std::process::Child,
    name: &str,
    budget: Duration,
    clock: Instant,
    log: &mut std::fs::File,
    outcome: &mut Outcome,
    vga: Option<(std::process::ChildStdin, PathBuf)>,
) {
    let (tx, rx) = mpsc::channel();
    let mut readers = Vec::new();
    let mut stdout = child.stdout.take();
    if let Some((monitor, dir)) = vga {
        if let Some(mut out) = stdout.take() {
            std::thread::spawn(move || std::io::copy(&mut out, &mut std::io::sink()));
        }
        let tx = tx.clone();
        readers.push(std::thread::spawn(move || capture(monitor, &dir, &tx)));
    }
    for stream in [
        stdout.map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
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
    outcome.finish();
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
        o.finish();
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

    fn rows(lines: &[&str]) -> Vec<String> {
        let mut r: Vec<String> = lines.iter().map(|l| (*l).to_owned()).collect();
        r.resize(25, String::new());
        r
    }

    #[test]
    fn a_screen_row_is_taken_once_it_is_finished() {
        let mut s = Screen::default();
        assert_eq!(
            s.take(rows(&["GK-BOOTED 2.0.40", "GK-BEG"]), false),
            ["GK-BOOTED 2.0.40"]
        );
        assert!(
            s.take(rows(&["GK-BOOTED 2.0.40", "GK-BEGIN smoke"]), false)
                .is_empty()
        );
        assert_eq!(
            s.take(rows(&["GK-BOOTED 2.0.40", "GK-BEGIN smoke"]), false),
            ["GK-BEGIN smoke"]
        );
        assert!(
            s.take(rows(&["GK-BOOTED 2.0.40", "GK-BEGIN smoke"]), false)
                .is_empty()
        );
        assert_eq!(
            s.take(
                rows(&["GK-BOOTED 2.0.40", "GK-BEGIN smoke", "GK-END smoke pass"]),
                true
            ),
            ["GK-END smoke pass"]
        );
    }

    #[test]
    fn a_scrolled_screen_is_followed() {
        let mut s = Screen::default();
        let full: Vec<String> = (0..25).map(|i| format!("line {i}")).collect();
        let lines = s.take(full.clone(), false);
        assert_eq!(lines.len(), 24);
        let mut moved: Vec<String> = full[2..].to_vec();
        moved.extend(["line 25".to_owned(), "line 26".to_owned()]);
        assert_eq!(s.take(moved, true), ["line 24", "line 25", "line 26"]);
    }

    #[test]
    fn a_cleared_screen_starts_over() {
        let mut s = Screen::default();
        let boot: Vec<String> = (0..25).map(|i| format!("kernel {i}")).collect();
        s.take(boot, false);
        assert_eq!(
            s.take(rows(&["GK-BOOTED 1.2.13", "GK-BEGIN smoke", ""]), false),
            ["GK-BOOTED 1.2.13"]
        );
    }

    #[test]
    fn vga_cells_become_text() {
        let mut bytes = vec![0u8; SCREEN_BYTES];
        for (i, c) in b"GK-END smoke pass".iter().enumerate() {
            bytes[160 + 2 * i] = *c;
            bytes[160 + 2 * i + 1] = 7;
        }
        let r = screen_rows(&bytes);
        assert_eq!(r.len(), 25);
        assert_eq!(r[0], "");
        assert_eq!(r[1], "GK-END smoke pass");
    }

    #[test]
    fn museum_command_lines_name_the_root() {
        let v: Version = "1.2.13".parse().unwrap();
        assert_eq!(
            museum_append("vga", "smoke", &v),
            "no-scroll root=/dev/hdb rw panic=-1 gk.suite=smoke gk.kernel=1.2.13"
        );
        let v: Version = "2.2.26".parse().unwrap();
        assert_eq!(
            museum_append("ttyS0", "smoke", &v),
            "console=ttyS0 root=/dev/ram0 rw panic=-1 noapic gk.suite=smoke gk.kernel=2.2.26"
        );
    }

    #[test]
    fn an_excused_check_does_not_fail_the_suite() {
        let lines = [
            "GK-BOOTED 2.6.0",
            "GK-BEGIN smoke",
            "GK-CHECK pid-1 pass",
            "GK-CHECK sysfs fail",
            "GK-END smoke fail 1",
        ];
        let mut o = read(&lines);
        assert!(!o.passed());
        o.excused = excused(&"2.6.0".parse().unwrap());
        assert!(o.passed());
        assert!(excused(&"2.6.10".parse().unwrap()).is_empty());
        let mut other = read(&[
            "GK-BOOTED 2.6.0",
            "GK-BEGIN smoke",
            "GK-CHECK pid-1 fail",
            "GK-CHECK sysfs fail",
            "GK-END smoke fail 2",
        ]);
        other.excused = vec!["sysfs".into()];
        assert!(!other.passed());
    }

    #[test]
    fn the_first_2_6_kernels_boot_from_a_disk() {
        let repo = gk_model::repo::Repo::load(std::path::Path::new("../..")).unwrap();
        let p = repo.platforms.get("i386").unwrap();
        let joined = |v: &str| {
            qemu_command(p, &v.parse().unwrap(), "smoke", 0x206)
                .unwrap()
                .join(" ")
        };
        let early = joined("2.6.5");
        assert!(early.contains("-drive file=/boot/root.img,format=raw,if=ide,index=0,snapshot=on"));
        assert!(early.contains(" root=/dev/hda rw init=/init"));
        assert!(!early.contains("-initrd"));
        let later = joined("2.6.6");
        assert!(later.contains("-initrd /boot/initramfs.cpio"));
        assert!(!later.contains("root="));
    }

    #[test]
    fn x86_64_before_2_6_16_runs_without_the_hypertransport_hole() {
        let repo = gk_model::repo::Repo::load(std::path::Path::new("../..")).unwrap();
        let p = repo.platforms.get("x86_64").unwrap();
        let cpu = |v: &str| {
            let cmd = qemu_command(p, &v.parse().unwrap(), "smoke", 0x206).unwrap();
            let at = cmd.iter().position(|a| a == "-cpu").unwrap();
            cmd[at + 1].clone()
        };
        assert_eq!(cpu("2.6.15"), "qemu64,vendor=GenuineIntel");
        assert_eq!(cpu("2.6.16"), "qemu64");
    }

    #[test]
    fn the_setup_header_gives_the_protocol() {
        let mut image = vec![0u8; 0x400];
        assert_eq!(protocol(&image), 0);
        image[0x202..0x208].copy_from_slice(b"HdrS\x02\x02");
        assert_eq!(protocol(&image), 0x202);
        assert!(needs_loader(protocol(&image)));
        image[0x206] = 3;
        assert!(!needs_loader(protocol(&image)));
        assert_eq!(protocol(&image[..0x100]), 0);
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
    fn a_marker_cut_in_a_word_is_joined_again() {
        let o = read(&[
            "GK-CHECK cpus pass",
            "GK-END smoke pas[    9.558465] md: stopping all md devices.",
            "[    9.6] WARNING: CPU: 0 PID: 1 at lib/x.c:3 f+0x1/0x2",
            "s 0",
            "[   12.346192] ACPI: Preparing to enter system sleep state S5",
        ]);
        assert_eq!(o.status.as_deref(), Some("pass"));
        assert_eq!(o.checks.len(), 1);
        assert_eq!(o.splats.len(), 1);
        assert!(o.passed());
        let o = read(&[
            "GK-CHECK cpus pa[    4.560537] md: stopping all md devices.",
            "ss",
            "GK-END smoke[    8.626687] md: stopping all md devices.",
            " pass 0",
        ]);
        assert_eq!(o.checks.len(), 1);
        assert_eq!(o.status.as_deref(), Some("pass"));
        assert!(o.passed());
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
