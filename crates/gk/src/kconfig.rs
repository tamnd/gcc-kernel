//! `.config` files and configuration fragments.
//!
//! Copied from rucc-kernel's `rk` with the `config-diff` half left out, which arrives with `gk config-diff`. A fragment is laid over a `.config` the way `merge_config.sh -m` does it, and `olddefconfig` settles it afterwards.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

/// A `.config`, as symbol to value. `# CONFIG_X is not set` is the value `n`.
pub type Config = BTreeMap<String, String>;

/// Read a `.config`.
#[must_use]
pub fn parse(text: &str) -> Config {
    let mut config = Config::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("# CONFIG_") {
            if let Some(symbol) = rest.strip_suffix(" is not set") {
                config.insert(symbol.to_string(), "n".to_string());
            }
        } else if let Some((symbol, value)) = line
            .strip_prefix("CONFIG_")
            .and_then(|rest| rest.split_once('='))
        {
            config.insert(symbol.to_string(), value.to_string());
        }
    }
    config
}

/// Read a `.config` from a file, or from a build directory holding one.
pub fn load(path: &Path) -> Result<Config, String> {
    let file = if path.is_dir() {
        path.join(".config")
    } else {
        path.to_path_buf()
    };
    let text =
        std::fs::read_to_string(&file).map_err(|e| format!("reading {}: {e}", file.display()))?;
    Ok(parse(&text))
}

/// A configuration fragment: symbols and the values it asks for, in order. Values are cut at a
/// `#` that follows whitespace, so that a line may carry a comment, which a `.config` may not.
#[must_use]
pub fn parse_fragment(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if let Some(symbol) = line
                .strip_prefix("# CONFIG_")
                .and_then(|r| r.strip_suffix(" is not set"))
            {
                return Some((symbol.to_string(), "n".to_string()));
            }
            let (symbol, value) = line.strip_prefix("CONFIG_")?.split_once('=')?;
            let value = value
                .find(" #")
                .or_else(|| value.find("\t#"))
                .map_or(value, |at| &value[..at]);
            Some((symbol.to_string(), value.trim().to_string()))
        })
        .collect()
}

/// A `.config` with a fragment laid over it, as `merge_config.sh -m` does: every line that sets
/// a symbol the fragment names is dropped, and the fragment's lines go at the end. Running
/// `olddefconfig` afterwards settles dependencies.
#[must_use]
pub fn merge(config: &str, fragment: &[(String, String)]) -> String {
    let named = |line: &str| {
        let line = line.trim();
        let symbol = line
            .strip_prefix("# CONFIG_")
            .and_then(|r| r.strip_suffix(" is not set"))
            .or_else(|| {
                line.strip_prefix("CONFIG_")
                    .and_then(|r| r.split_once('='))
                    .map(|(s, _)| s)
            });
        symbol.is_some_and(|s| fragment.iter().any(|(f, _)| f == s))
    };
    let mut out: String = config
        .lines()
        .filter(|l| !named(l))
        .flat_map(|l| [l, "\n"])
        .collect();
    for (symbol, value) in fragment {
        if value == "n" {
            let _ = writeln!(out, "# CONFIG_{symbol} is not set");
        } else {
            let _ = writeln!(out, "CONFIG_{symbol}={value}");
        }
    }
    out
}

/// The fragment's requests that did not take effect, usually because the symbol does not exist
/// in this version or on this architecture, or depends on something that is off. A symbol asked
/// to be `n` that is missing counts as taken.
#[must_use]
pub fn missed(config: &Config, fragment: &[(String, String)]) -> Vec<String> {
    fragment
        .iter()
        .filter(|(symbol, value)| match config.get(symbol) {
            Some(v) => v != value,
            None => value != "n",
        })
        .map(|(symbol, _)| symbol.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEW: &str = "\
CONFIG_CC_VERSION_TEXT=\"x86_64-linux-gnu-gcc (GCC) 16.2.0\"
CONFIG_CC_IS_GCC=y
CONFIG_GCC_VERSION=160200
CONFIG_CC_HAS_ASM_GOTO_OUTPUT=y
# CONFIG_KASAN is not set
CONFIG_STACKPROTECTOR=y
";

    const OLD: &str = "\
CONFIG_CC_VERSION_TEXT=\"x86_64-linux-gnu-gcc (GCC) 8.5.0\"
CONFIG_CC_IS_GCC=y
CONFIG_GCC_VERSION=80500
# CONFIG_KASAN is not set
# CONFIG_STACKPROTECTOR is not set
";

    #[test]
    fn set_unset_and_missing_symbols_are_read() {
        let config = parse(NEW);
        assert_eq!(config["KASAN"], "n");
        assert_eq!(config["GCC_VERSION"], "160200");
        assert_eq!(config.len(), 6);
    }

    #[test]
    fn a_fragment_replaces_what_it_names_and_reports_what_did_not_stick() {
        let fragment = parse_fragment(
            "# console\nCONFIG_STACKPROTECTOR=y   # on\nCONFIG_SERIAL_AMBA_PL011=y\n# CONFIG_KASAN is not set\n",
        );
        assert_eq!(fragment.len(), 3);
        assert_eq!(fragment[0], ("STACKPROTECTOR".to_string(), "y".to_string()));
        let merged = merge(OLD, &fragment);
        assert!(!merged.contains("# CONFIG_STACKPROTECTOR is not set"));
        assert!(merged.ends_with("CONFIG_SERIAL_AMBA_PL011=y\n# CONFIG_KASAN is not set\n"));
        let settled = parse(OLD);
        assert_eq!(
            missed(&settled, &fragment),
            ["STACKPROTECTOR", "SERIAL_AMBA_PL011"]
        );
    }

    #[test]
    fn the_committed_fragments_read() {
        for entry in std::fs::read_dir("../../configs").unwrap() {
            let path = entry.unwrap().path();
            let fragment = parse_fragment(&std::fs::read_to_string(&path).unwrap());
            assert!(fragment.len() > 10, "{} is nearly empty", path.display());
            assert!(fragment.iter().all(|(_, v)| v == "y" || v == "n"));
        }
    }
}
