//! What the shim needs to know: which compiler to run and where to write.
//!
//! `gk cell` copies `gk-cc` into the build's `bin` directory and writes `gk-cc.toml` next to the copy. The shim reads that file from the directory it was run from, and environment variables of the same meaning override it. The file is what makes the shim work when kbuild calls it from a sub make whose environment has been cleaned, which happens in the boot and vDSO Makefiles.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The name of the file next to the shim.
pub const FILE_NAME: &str = "gk-cc.toml";

/// The shim's settings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ShimConfig {
    /// The real compiler, as an absolute path. Overridden by `GK_REAL_CC`.
    pub real: PathBuf,
    /// The `compile.jsonl` to append to. Overridden by `GK_COMPILE_LOG`.
    pub log: PathBuf,
    /// Whether to compile everything twice and compare. Overridden by `GK_TWICE`.
    #[serde(default)]
    pub twice: bool,
}

impl ShimConfig {
    /// Read the file next to the shim, if any, then apply the environment.
    ///
    /// Returns `None` when neither says which compiler to run, which is the one setting without a sensible default.
    #[must_use]
    pub fn load(shim_dir: Option<&Path>, env: &dyn Fn(&str) -> Option<String>) -> Option<Self> {
        let mut config = shim_dir
            .map(|dir| dir.join(FILE_NAME))
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|text| toml::from_str::<Self>(&text).ok())
            .unwrap_or_default();
        if let Some(real) = env("GK_REAL_CC").filter(|v| !v.is_empty()) {
            config.real = PathBuf::from(real);
        }
        if let Some(log) = env("GK_COMPILE_LOG").filter(|v| !v.is_empty()) {
            config.log = PathBuf::from(log);
        }
        if let Some(value) = env("GK_TWICE") {
            config.twice = truthy(&value);
        }
        (!config.real.as_os_str().is_empty()).then_some(config)
    }

    /// The file's text.
    #[must_use]
    pub fn to_toml(&self) -> String {
        toml::to_string(self).unwrap_or_default()
    }
}

fn truthy(value: &str) -> bool {
    matches!(value.trim(), "1" | "true" | "yes" | "on")
}

/// The environment variables that can change what a compiler does, recorded when set.
///
/// `PATH` is here because it decides which assembler and linker GCC finds. The locale variables are here because they change the language of diagnostics, which kbuild probes sometimes grep. `KBUILD_` variables are recorded as well.
pub const RECORDED_ENV: &[&str] = &[
    "PATH",
    "CPATH",
    "C_INCLUDE_PATH",
    "LIBRARY_PATH",
    "COMPILER_PATH",
    "GCC_EXEC_PREFIX",
    "SOURCE_DATE_EPOCH",
    "DEPENDENCIES_OUTPUT",
    "SUNPRO_DEPENDENCIES",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "LC_MESSAGES",
    "ARCH",
    "CROSS_COMPILE",
    "LLVM",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    #[test]
    fn with_nothing_to_go_on_there_is_no_config() {
        assert!(ShimConfig::load(None, &env_of(&[])).is_none());
    }

    #[test]
    fn the_environment_is_enough_on_its_own() {
        let config = ShimConfig::load(
            None,
            &env_of(&[("GK_REAL_CC", "/usr/bin/gcc-16"), ("GK_TWICE", "1")]),
        )
        .unwrap();
        assert_eq!(config.real, Path::new("/usr/bin/gcc-16"));
        assert!(config.twice);
    }

    #[test]
    fn the_file_is_read_and_the_environment_wins() {
        let dir = std::env::temp_dir().join(format!("gk-config-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let written = ShimConfig {
            real: "/opt/gk/gcc-16.2.0/bin/x86_64-linux-gnu-gcc".into(),
            log: "/b/compile.jsonl".into(),
            twice: true,
        };
        std::fs::write(dir.join(FILE_NAME), written.to_toml()).unwrap();
        let read = ShimConfig::load(Some(&dir), &env_of(&[])).unwrap();
        assert_eq!(read, written);
        let overridden = ShimConfig::load(Some(&dir), &env_of(&[("GK_TWICE", "0")])).unwrap();
        assert!(!overridden.twice);
        std::fs::remove_dir_all(&dir).ok();
    }
}
