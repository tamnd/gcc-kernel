//! `.config` files and configuration fragments.
//!
//! Copied from rucc-kernel's `rk`, the `config-diff` half without `config-divergences.toml`, which belongs to rucc. `gk config-diff` compares GCC columns, where every difference is data rather than something to explain. A fragment is laid over a `.config` the way `merge_config.sh -m` does it, and `olddefconfig` settles it afterwards.

use std::collections::{BTreeMap, BTreeSet};
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

/// One symbol that differs. A side with no value did not have the symbol at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Difference {
    /// The symbol, without `CONFIG_`.
    pub symbol: String,
    /// The first build's value.
    pub from: Option<String>,
    /// The second build's value.
    pub to: Option<String>,
}

/// Every symbol whose value differs, in symbol order.
#[must_use]
pub fn diff(from: &Config, to: &Config) -> Vec<Difference> {
    let symbols: BTreeSet<&String> = from.keys().chain(to.keys()).collect();
    symbols
        .into_iter()
        .filter(|s| from.get(*s) != to.get(*s))
        .map(|s| Difference {
            symbol: s.clone(),
            from: from.get(s).cloned(),
            to: to.get(s).cloned(),
        })
        .collect()
}

/// The differences as a markdown table with the two sides named.
#[must_use]
pub fn table(differences: &[Difference], from: &str, to: &str) -> String {
    let side = |v: Option<&String>| v.map_or("(absent)", String::as_str).to_owned();
    let mut s = format!("| symbol | {from} | {to} |\n|---|---|---|\n");
    for d in differences {
        let _ = writeln!(
            s,
            "| {} | {} | {} |",
            d.symbol,
            side(d.from.as_ref()),
            side(d.to.as_ref())
        );
    }
    s
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
            let least = if path.ends_with("tiny.gk") { 1 } else { 10 };
            assert!(
                fragment.len() >= least,
                "{} is nearly empty",
                path.display()
            );
            assert!(fragment.iter().all(|(_, v)| v == "y" || v == "n"));
        }
    }

    #[test]
    fn two_columns_differ_where_a_probe_answered_differently() {
        let a = parse("CONFIG_GCC_VERSION=140400\nCONFIG_CC_HAS_ASM_GOTO_OUTPUT=y\nCONFIG_X86=y\n");
        let b = parse(
            "CONFIG_GCC_VERSION=160200\n# CONFIG_CC_HAS_ASM_GOTO_OUTPUT is not set\nCONFIG_X86=y\nCONFIG_CC_HAS_COUNTED_BY=y\n",
        );
        let d = diff(&a, &b);
        let symbols: Vec<&str> = d.iter().map(|d| d.symbol.as_str()).collect();
        assert_eq!(
            symbols,
            ["CC_HAS_ASM_GOTO_OUTPUT", "CC_HAS_COUNTED_BY", "GCC_VERSION"]
        );
        assert_eq!(d[1].from, None);
        let t = table(&d, "14.4.0", "16.2.0");
        assert!(t.contains("| CC_HAS_COUNTED_BY | (absent) | y |"));
        assert!(t.contains("| CC_HAS_ASM_GOTO_OUTPUT | y | n |"));
    }
}
