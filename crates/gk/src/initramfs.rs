//! `gk init`: the init program of spec 07.4 and the initramfs it is booted from.
//!
//! `provision/init/init.c` is compiled once per platform with the bundle and host that `platforms.toml` pins under `init`, in the host container with no network, and packed as `/init` into an uncompressed newc archive. The archive is written here rather than by `cpio`, so its bytes depend on the program alone: every entry has owner 0, time 0 and an inode number given by its place in the list. Its digest is what a booted cell records.
//!
//! The result lives in `<cache>/init/<platform>`, with an `init.json` that says what it was built from. It is built again only when the source, the bundle or the image change.

use crate::{fetch, forge, net};
use gk_model::Version;
use gk_model::platforms::Platform;
use gk_model::repo::Repo;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The flags every build of init uses. No C library, no start files, nothing that needs a loader, and nothing that varies between machines.
const FLAGS: [&str; 13] = [
    "-static",
    "-nostdlib",
    "-ffreestanding",
    "-fno-stack-protector",
    "-fno-pic",
    "-no-pie",
    "-fno-asynchronous-unwind-tables",
    "-O2",
    "-Wall",
    "-Wextra",
    "-s",
    "-Wl,--build-id=none",
    "-Wl,-z,max-page-size=4096",
];

/// What `gk init` writes next to each initramfs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// The platform.
    pub platform: String,
    /// `sha256:` and the SHA-256 of `init.c`.
    pub source: String,
    /// The GCC column that built it.
    pub gcc: String,
    /// The digest of that bundle.
    pub bundle: String,
    /// The image the compiler ran in.
    pub image: String,
    /// The flags, the fixed ones and the platform's own.
    pub flags: Vec<String>,
    /// `sha256:` and the SHA-256 of the init program.
    pub init: String,
    /// `sha256:` and the SHA-256 of the initramfs.
    pub digest: String,
    /// The gk version that built it.
    pub gk: String,
}

/// Where a platform's initramfs lives.
#[must_use]
pub fn dir(platform: &str) -> PathBuf {
    fetch::cache_dir().join("init").join(platform)
}

/// The initramfs of a platform, built if it is missing or stale. Returns its manifest and path.
pub fn for_platform(repo: &Repo, p: &Platform) -> Result<(Manifest, PathBuf), String> {
    let pin = p
        .init
        .as_ref()
        .ok_or_else(|| format!("platforms.toml has no init for {}", p.name))?;
    let triple = pin.triple.as_deref().unwrap_or(&p.triple);
    let source_path = repo.root.join("provision/init/init.c");
    let source = std::fs::read(&source_path)
        .map_err(|e| format!("reading {}: {e}", source_path.display()))?;
    let bundle = forge::manifest(&pin.gcc, triple)?;
    let image = forge::image_for(repo, &pin.host)?;
    let mut flags: Vec<String> = FLAGS.iter().map(|f| (*f).to_owned()).collect();
    flags.extend(pin.flags.iter().cloned());
    let dir = dir(&p.name);
    let archive = dir.join("initramfs.cpio");
    let manifest_path = dir.join("init.json");
    let wanted = |m: &Manifest| {
        m.source == format!("sha256:{}", net::sha256_bytes(&source))
            && m.bundle == bundle.digest
            && m.image == image
            && m.flags == flags
    };
    if let Ok(text) = std::fs::read_to_string(&manifest_path)
        && let Ok(m) = serde_json::from_str::<Manifest>(&text)
        && wanted(&m)
        && archive.is_file()
        && net::sha256_file(&archive).is_ok_and(|d| format!("sha256:{d}") == m.digest)
    {
        return Ok((m, archive));
    }

    std::fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let bundle_dir = forge::unpacked(&bundle)?;
    let mount = format!("/opt/gk/t/{}", bundle.id);
    let program = dir.join("init");
    let _ = std::fs::remove_file(&program);
    let out = Command::new("docker")
        .args(["run", "--rm", "--network=none"])
        .arg("-v")
        .arg(format!(
            "{}:/init:ro",
            source_path.parent().unwrap_or(Path::new(".")).display()
        ))
        .arg("-v")
        .arg(format!("{}:{mount}:ro", bundle_dir.display()))
        .arg("-v")
        .arg(format!("{}:/out", dir.display()))
        .args(["-w", "/init"])
        .arg(&image)
        .arg(format!("{mount}/bin/{triple}-gcc"))
        .args(&flags)
        .args(["-o", "/out/init", "init.c"])
        .output()
        .map_err(|e| format!("running docker: {e}"))?;
    let said = String::from_utf8_lossy(&out.stderr).trim().to_owned();
    if !out.status.success() {
        return Err(format!("compiling init for {} failed:\n{said}", p.name));
    }
    if !said.is_empty() {
        return Err(format!(
            "init for {} compiles with warnings, which it must not:\n{said}",
            p.name
        ));
    }
    let init =
        std::fs::read(&program).map_err(|e| format!("reading {}: {e}", program.display()))?;
    if !is_static_elf(&init) {
        return Err(format!("{} is not a static ELF program", program.display()));
    }
    let bytes = initramfs(&init);
    let partial = archive.with_extension("part");
    std::fs::write(&partial, &bytes).map_err(|e| format!("writing {}: {e}", partial.display()))?;
    std::fs::rename(&partial, &archive)
        .map_err(|e| format!("renaming {}: {e}", partial.display()))?;
    let m = Manifest {
        platform: p.name.clone(),
        source: format!("sha256:{}", net::sha256_bytes(&source)),
        gcc: pin.gcc.clone(),
        bundle: bundle.digest.clone(),
        image,
        flags,
        init: format!("sha256:{}", net::sha256_bytes(&init)),
        digest: format!("sha256:{}", net::sha256_bytes(&bytes)),
        gk: env!("CARGO_PKG_VERSION").to_owned(),
    };
    let text = serde_json::to_string_pretty(&m).map_err(|e| e.to_string())? + "\n";
    std::fs::write(&manifest_path, text)
        .map_err(|e| format!("writing {}: {e}", manifest_path.display()))?;
    Ok((m, archive))
}

/// Whether a kernel has no devtmpfs, which came in 2.6.32. Its archive then carries `/dev/null` as a node of its own, because the smoke suite writes to it.
#[must_use]
pub fn static_dev(version: &Version) -> bool {
    version.series(3) < [2, 6, 32].to_vec()
}

/// The initramfs a kernel boots from: the platform's, or for a kernel with no devtmpfs the same program in an archive that also has `/dev/null`. The manifest's digest is the digest of the archive returned, so a 3.x or later cell keeps the identity it had.
pub fn for_kernel(
    repo: &Repo,
    p: &Platform,
    version: &Version,
) -> Result<(Manifest, PathBuf), String> {
    let (m, archive) = for_platform(repo, p)?;
    if !static_dev(version) {
        return Ok((m, archive));
    }
    let program = dir(&p.name).join("init");
    let init =
        std::fs::read(&program).map_err(|e| format!("reading {}: {e}", program.display()))?;
    if format!("sha256:{}", net::sha256_bytes(&init)) != m.init {
        return Err(format!(
            "{} is not the program init.json names; run gk init {}",
            program.display(),
            p.name
        ));
    }
    let bytes = archive_of(&init, true);
    let path = dir(&p.name).join("initramfs-static-dev.cpio");
    let partial = path.with_extension("part");
    std::fs::write(&partial, &bytes).map_err(|e| format!("writing {}: {e}", partial.display()))?;
    std::fs::rename(&partial, &path).map_err(|e| format!("renaming {}: {e}", partial.display()))?;
    Ok((
        Manifest {
            digest: format!("sha256:{}", net::sha256_bytes(&bytes)),
            ..m
        },
        path,
    ))
}

/// Whether the bytes are an ELF executable with no program interpreter, which is what the kernel can run as `/init` with nothing else in the archive.
#[must_use]
pub fn is_static_elf(bytes: &[u8]) -> bool {
    if bytes.len() < 52 || &bytes[..4] != b"\x7fELF" {
        return false;
    }
    let wide = bytes[4] == 2;
    let little = bytes[5] == 1;
    let half = |at: usize| -> usize {
        let b = [bytes[at], bytes[at + 1]];
        usize::from(if little {
            u16::from_le_bytes(b)
        } else {
            u16::from_be_bytes(b)
        })
    };
    let word = |at: usize, wide: bool| -> usize {
        if wide {
            let mut b = [0; 8];
            b.copy_from_slice(&bytes[at..at + 8]);
            usize::try_from(if little {
                u64::from_le_bytes(b)
            } else {
                u64::from_be_bytes(b)
            })
            .unwrap_or(usize::MAX)
        } else {
            let mut b = [0; 4];
            b.copy_from_slice(&bytes[at..at + 4]);
            usize::try_from(if little {
                u32::from_le_bytes(b)
            } else {
                u32::from_be_bytes(b)
            })
            .unwrap_or(usize::MAX)
        }
    };
    if wide && bytes.len() < 64 {
        return false;
    }
    // ET_EXEC only: a static PIE would need relocating, and init is built with -no-pie.
    if half(16) != 2 {
        return false;
    }
    let (phoff, phentsize, phnum) = if wide {
        (word(32, true), half(54), half(56))
    } else {
        (word(28, false), half(42), half(44))
    };
    (0..phnum).all(|i| {
        let at = phoff.saturating_add(i.saturating_mul(phentsize));
        // A program header we cannot read means the file is not what we built.
        at.checked_add(4)
            .is_some_and(|end| end <= bytes.len() && word(at, false) & 0xffff_ffff != 3)
    })
}

/// One file in the archive.
struct Entry<'a> {
    name: &'a str,
    mode: u32,
    rdev: (u32, u32),
    data: &'a [u8],
}

/// Append one newc entry. The fields are inode, mode, uid, gid, links, mtime, size, the two device numbers of the file system, the two of the device itself, the name's length with its NUL, and a checksum that newc leaves at zero.
fn push_entry(out: &mut Vec<u8>, ino: u32, entry: &Entry<'_>) {
    let name_size = entry.name.len() + 1;
    let fields = [
        ino,
        entry.mode,
        0,
        0,
        if entry.mode & 0o170_000 == 0o040_000 {
            2
        } else {
            1
        },
        0,
        u32::try_from(entry.data.len()).unwrap_or(u32::MAX),
        0,
        0,
        entry.rdev.0,
        entry.rdev.1,
        u32::try_from(name_size).unwrap_or(u32::MAX),
        0,
    ];
    out.extend_from_slice(b"070701");
    for field in fields {
        out.extend_from_slice(format!("{field:08X}").as_bytes());
    }
    out.extend_from_slice(entry.name.as_bytes());
    out.push(0);
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
    out.extend_from_slice(entry.data);
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
}

/// The initramfs for an init program, as bytes: the mount points init uses, a console device so the kernel has somewhere to send init's output before devtmpfs is mounted, and `/init` itself.
#[must_use]
pub fn initramfs(init: &[u8]) -> Vec<u8> {
    archive_of(init, false)
}

/// The archive, with `/dev/null` as well when `dev_null` is set.
fn archive_of(init: &[u8], dev_null: bool) -> Vec<u8> {
    let dir = |name| Entry {
        name,
        mode: 0o040_755,
        rdev: (0, 0),
        data: &[],
    };
    let node = |name, rdev| Entry {
        name,
        mode: 0o020_600,
        rdev,
        data: &[],
    };
    let mut entries = vec![dir("dev"), node("dev/console", (5, 1))];
    if dev_null {
        entries.push(Entry {
            mode: 0o020_666,
            ..node("dev/null", (1, 3))
        });
    }
    entries.extend([
        dir("proc"),
        dir("sys"),
        dir("tmp"),
        Entry {
            name: "init",
            mode: 0o100_755,
            rdev: (0, 0),
            data: init,
        },
        Entry {
            name: "TRAILER!!!",
            mode: 0,
            rdev: (0, 0),
            data: &[],
        },
    ]);
    let mut out = Vec::with_capacity(init.len() + 1024);
    for (ino, entry) in (1..).zip(&entries) {
        let ino = if entry.name == "TRAILER!!!" { 0 } else { ino };
        push_entry(&mut out, ino, entry);
    }
    // The kernel reads the archive in 512 byte blocks, as cpio writes it.
    while !out.len().is_multiple_of(512) {
        out.push(0);
    }
    out
}

/// Run `gk init`: build the initramfs of every platform named, or of every platform that has an init pin, and print their digests.
pub fn run(repo: &Repo, only: &[String]) -> Result<(), String> {
    let platforms: Vec<&Platform> = repo
        .platforms
        .platforms
        .iter()
        .filter(|p| p.init.is_some())
        .filter(|p| only.is_empty() || only.contains(&p.name))
        .collect();
    if let Some(missing) = only
        .iter()
        .find(|n| !platforms.iter().any(|p| &&p.name == n))
    {
        return Err(format!("{missing} has no init in platforms.toml"));
    }
    for p in platforms {
        let (m, path) = for_platform(repo, p)?;
        println!("{:<8} {} {}", p.name, m.digest, path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(entry: &[u8], n: usize) -> u32 {
        let at = 6 + n * 8;
        u32::from_str_radix(std::str::from_utf8(&entry[at..at + 8]).unwrap(), 16).unwrap()
    }

    #[test]
    fn the_archive_depends_on_init_alone() {
        let a = initramfs(b"\x7fELF one");
        assert_eq!(a, initramfs(b"\x7fELF one"));
        assert_ne!(a, initramfs(b"\x7fELF two"));
        assert!(a.len().is_multiple_of(512));
        assert!(a.starts_with(b"070701"));
    }

    #[test]
    fn entries_are_aligned_and_named() {
        let init = b"abcde";
        let a = initramfs(init);
        let mut at = 0;
        let mut names = Vec::new();
        loop {
            let entry = &a[at..];
            assert_eq!(&entry[..6], b"070701", "entry at {at}");
            let size = field(entry, 6) as usize;
            let name_size = field(entry, 11) as usize;
            let name = std::str::from_utf8(&entry[110..110 + name_size - 1]).unwrap();
            names.push(name.to_owned());
            let data = (110 + name_size).next_multiple_of(4);
            if name == "init" {
                assert_eq!(&entry[data..data + size], init);
                assert_eq!(field(entry, 1), 0o100_755);
            }
            if name == "dev/console" {
                assert_eq!((field(entry, 9), field(entry, 10)), (5, 1));
            }
            if name == "TRAILER!!!" {
                break;
            }
            at += (data + size).next_multiple_of(4);
        }
        assert_eq!(
            names,
            [
                "dev",
                "dev/console",
                "proc",
                "sys",
                "tmp",
                "init",
                "TRAILER!!!"
            ]
        );
    }

    #[test]
    fn kernels_before_devtmpfs_get_a_dev_null() {
        let init = b"abcde";
        let plain = initramfs(init);
        let old = archive_of(init, true);
        assert_eq!(archive_of(init, false), plain);
        let at = old.windows(9).position(|w| w == b"dev/null\0").unwrap();
        let entry = &old[at - 110..];
        assert_eq!(field(entry, 1), 0o020_666);
        assert_eq!((field(entry, 9), field(entry, 10)), (1, 3));
        assert!(!plain.windows(8).any(|w| w == b"dev/null"));
        assert!(static_dev(&"2.6.0".parse().unwrap()));
        assert!(static_dev(&"2.6.31.14".parse().unwrap()));
        assert!(!static_dev(&"2.6.32".parse().unwrap()));
        assert!(!static_dev(&"3.0".parse().unwrap()));
    }

    fn elf64(kind: u16, interp: bool) -> Vec<u8> {
        let mut b = vec![0u8; 64 + 2 * 56];
        b[..4].copy_from_slice(b"\x7fELF");
        b[4] = 2;
        b[5] = 1;
        b[16..18].copy_from_slice(&kind.to_le_bytes());
        b[32..40].copy_from_slice(&64u64.to_le_bytes());
        b[54..56].copy_from_slice(&56u16.to_le_bytes());
        b[56..58].copy_from_slice(&2u16.to_le_bytes());
        b[64..68].copy_from_slice(&1u32.to_le_bytes());
        b[120..124].copy_from_slice(&(if interp { 3u32 } else { 4 }).to_le_bytes());
        b
    }

    #[test]
    fn only_static_executables_pass() {
        assert!(is_static_elf(&elf64(2, false)));
        assert!(!is_static_elf(&elf64(2, true)));
        assert!(!is_static_elf(&elf64(3, false)));
        assert!(!is_static_elf(b"#!/bin/sh\nexec /bin/true\n"));
        let mut cut = elf64(2, false);
        cut.truncate(100);
        assert!(!is_static_elf(&cut));
    }
}
