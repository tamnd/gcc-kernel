//! Cells and their identity (spec 02.1).
//!
//! A cell's identity is the SHA-256 of the canonical JSON of its six coordinates and the digests they resolve to. Two cells with the same identity are the same experiment. A rebuilt toolchain whose bytes differ, a host image rebuilt from the same Dockerfile, or an edited fragment each give a new identity.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A named input and the digest it resolved to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Named {
    /// The name in the pin file, as in `gcc-16.2.0`.
    pub name: String,
    /// The digest of what the name resolved to, as in `sha256:...`.
    pub digest: String,
}

/// The six coordinates of a cell with their digests, and the QEMU container and initramfs when the cell boots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coordinates {
    /// K, with the tarball hash.
    pub kernel: Named,
    /// G, with the toolchain bundle digest.
    pub gcc: Named,
    /// B, with its digest inside the bundle.
    pub binutils: Named,
    /// P, as in `x86_64`.
    pub platform: String,
    /// C, with the hash of the merged configuration input.
    pub config: Named,
    /// H, with the container digest.
    pub host: Named,
    /// The boot container's digest, empty for cells that do not boot.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub qemu: String,
    /// The initramfs digest, empty for cells that do not boot.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub initramfs: String,
}

impl Coordinates {
    /// The canonical JSON the identity is the hash of: fields in the order above, no spaces.
    #[must_use]
    pub fn canonical(&self) -> String {
        serde_json::to_string(self).expect("plain strings always serialize")
    }

    /// The cell's identity, as `sha256:` and 64 hex digits.
    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "sha256:{}",
            hex(&Sha256::digest(self.canonical().as_bytes()))
        )
    }

    /// A short form of the identity for directory names and logs: the first 16 hex digits.
    #[must_use]
    pub fn short_id(&self) -> String {
        self.identity()["sha256:".len().."sha256:".len() + 16].to_owned()
    }
}

/// Lower case hex.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str, digest: &str) -> Named {
        Named {
            name: name.into(),
            digest: digest.into(),
        }
    }

    fn cell() -> Coordinates {
        Coordinates {
            kernel: named("7.2.8", "sha256:12e8"),
            gcc: named("gcc-16.2.0", "sha256:aa"),
            binutils: named("binutils-2.45", "sha256:bb"),
            platform: "x86_64".into(),
            config: named("defconfig+gk", "sha256:cc"),
            host: named("gk-host-trixie", "sha256:dd"),
            qemu: String::new(),
            initramfs: String::new(),
        }
    }

    #[test]
    fn identity_is_stable() {
        let c = cell();
        assert_eq!(c.identity(), cell().identity());
        assert_eq!(c.identity().len(), "sha256:".len() + 64);
        assert_eq!(c.short_id().len(), 16);
        assert!(c.canonical().starts_with(r#"{"kernel":{"name":"7.2.8""#));
    }

    #[test]
    fn any_changed_digest_is_a_new_cell() {
        let mut other = cell();
        other.gcc.digest = "sha256:ab".into();
        assert_ne!(other.identity(), cell().identity());
        let mut booted = cell();
        booted.qemu = "sha256:ee".into();
        assert_ne!(booted.identity(), cell().identity());
        let mut other_init = booted.clone();
        other_init.initramfs = "sha256:ff".into();
        assert_ne!(other_init.identity(), booted.identity());
    }

    #[test]
    fn a_cell_that_does_not_boot_keeps_its_old_identity() {
        assert!(!cell().canonical().contains("qemu"));
        assert!(!cell().canonical().contains("initramfs"));
    }
}
