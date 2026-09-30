//! `eras.toml`: kernel eras, each with its era GCC, host environments and configuration fragment (spec 03.3).

use crate::version::Version;
use crate::{Step, step_for};
use serde::{Deserialize, Serialize};

/// The file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eras {
    /// Every era, oldest first.
    #[serde(rename = "era", default)]
    pub eras: Vec<Era>,
}

/// One era.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Era {
    /// The name, as in `M14`.
    pub name: String,
    /// The first kernel of the era. The era runs up to the next era's first kernel.
    pub from: Version,
    /// The eras of the rucc-kernel plan this one maps to, as in `["E11"]`.
    #[serde(default)]
    pub plan_eras: Vec<String>,
    /// The era GCC e(K), as a column id in `gccs.toml`.
    pub gcc: String,
    /// A second reference GCC, where the era has one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub second_gcc: String,
    /// The host environment by kernel version, as names in `hosts.toml`.
    pub hosts: Vec<Step>,
    /// Why a boundary moved, with the cell that showed it (spec 03.3).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

impl Eras {
    /// The era a kernel belongs to.
    #[must_use]
    pub fn of(&self, kernel: &Version) -> Option<&Era> {
        self.eras
            .iter()
            .filter(|e| e.from <= *kernel)
            .max_by(|a, b| a.from.cmp(&b.from))
    }

    /// The host environment a kernel is built in.
    #[must_use]
    pub fn host_for(&self, kernel: &Version) -> Option<&str> {
        step_for(&self.of(kernel)?.hosts, kernel)
    }
}

impl Era {
    /// The configuration fragment of the era, relative to the repository.
    #[must_use]
    pub fn fragment(&self) -> String {
        format!("configs/fragment.{}", self.name)
    }
}
