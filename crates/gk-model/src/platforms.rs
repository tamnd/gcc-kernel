//! `platforms.toml`: the architectures, with the QEMU machine, CPU and console each one boots with at each kernel age (spec 06).

use crate::version::Version;
use crate::{Step, step_for};
use serde::{Deserialize, Serialize};

/// The file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Platforms {
    /// Every platform.
    #[serde(rename = "platform", default)]
    pub platforms: Vec<Platform>,
}

/// One platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Platform {
    /// The name, as in `x86_64`.
    pub name: String,
    /// The tier, 1 to 4.
    pub tier: u8,
    /// The GNU triple of its toolchain bundles.
    pub triple: String,
    /// kbuild's `ARCH` by kernel version.
    pub arch: Vec<Step>,
    /// The first kernel graded on it.
    pub first_kernel: Version,
    /// The first GCC with a back end that builds its kernel. Older columns are n/a.
    pub first_gcc: Version,
    /// The defconfig target by kernel version.
    pub defconfig: Vec<Step>,
    /// The boot image kbuild writes, by kernel version, relative to `arch/<ARCH>/boot`.
    pub image: Vec<Step>,
    /// The QEMU binary.
    pub qemu: String,
    /// The QEMU machine by kernel version.
    pub machine: Vec<Step>,
    /// The QEMU CPU by kernel version.
    pub cpu: Vec<Step>,
    /// The console device by kernel version.
    pub console: Vec<Step>,
    /// How long a build and a boot may take before the cell is stopped.
    pub budget: Budget,
}

/// Time limits for one cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Budget {
    /// Minutes for configure and build together.
    pub build_minutes: u32,
    /// Seconds from QEMU start to the boot marker.
    pub boot_seconds: u32,
}

impl Platforms {
    /// A platform by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Platform> {
        self.platforms.iter().find(|p| p.name == name)
    }
}

impl Platform {
    /// kbuild's `ARCH` for a kernel.
    #[must_use]
    pub fn arch_for(&self, kernel: &Version) -> Option<&str> {
        step_for(&self.arch, kernel)
    }

    /// The defconfig target for a kernel.
    #[must_use]
    pub fn defconfig_for(&self, kernel: &Version) -> Option<&str> {
        step_for(&self.defconfig, kernel)
    }

    /// The boot image for a kernel.
    #[must_use]
    pub fn image_for(&self, kernel: &Version) -> Option<&str> {
        step_for(&self.image, kernel)
    }

    /// The QEMU machine for a kernel.
    #[must_use]
    pub fn machine_for(&self, kernel: &Version) -> Option<&str> {
        step_for(&self.machine, kernel)
    }

    /// The QEMU CPU for a kernel.
    #[must_use]
    pub fn cpu_for(&self, kernel: &Version) -> Option<&str> {
        step_for(&self.cpu, kernel)
    }

    /// The console for a kernel.
    #[must_use]
    pub fn console_for(&self, kernel: &Version) -> Option<&str> {
        step_for(&self.console, kernel)
    }

    /// Whether a cell of this platform with this kernel and GCC can exist at all. When it cannot, the cell is n/a (spec 02.3).
    #[must_use]
    pub fn applies(&self, kernel: &Version, gcc: &Version) -> bool {
        *kernel >= self.first_kernel && *gcc >= self.first_gcc
    }
}
