//! `hosts.toml`: the host environments kernels are built in, the forge containers toolchains are built in, and the QEMU container cells boot in (spec 05, 04.4 and 07.4).

use serde::{Deserialize, Serialize};

/// The file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hosts {
    /// Every host and forge.
    #[serde(rename = "host", default)]
    pub hosts: Vec<Host>,
}

/// One container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Host {
    /// The name, as in `gk-host-trixie`.
    pub name: String,
    /// `host` for a kernel build host, `forge` for a toolchain forge, `boot` for the QEMU container.
    pub kind: String,
    /// The base image, as in `debian:trixie`.
    pub base: String,
    /// The architectures of the base image.
    pub arches: Vec<String>,
    /// The host GCC the distribution ships, as its archive lists it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub gcc: String,
    /// The distribution's binutils.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub binutils: String,
    /// The distribution's make.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub make: String,
    /// The distribution's QEMU, for the boot container.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub qemu: String,
    /// The pushed image, by digest, as in `ghcr.io/tamnd/gcc-kernel/gk-host-trixie@sha256:...`. Empty until the image has been built.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub image: String,
}

impl Hosts {
    /// A host by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Host> {
        self.hosts.iter().find(|h| h.name == name)
    }
}

impl Host {
    /// The Dockerfile that builds it, relative to the repository.
    #[must_use]
    pub fn dockerfile(&self) -> String {
        let dir = match self.kind.as_str() {
            "forge" => "forge",
            "boot" => "boot",
            _ => "hosts",
        };
        format!("provision/{dir}/{}/Dockerfile", self.name)
    }

    /// Whether the image has been built and pinned.
    #[must_use]
    pub fn is_built(&self) -> bool {
        self.image.contains("@sha256:")
    }
}
