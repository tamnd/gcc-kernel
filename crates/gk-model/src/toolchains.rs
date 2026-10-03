//! `gccs.toml` and `binutils.toml`: the columns of the matrix and the binutils each one is paired with (spec 04).

use crate::version::Version;
use serde::{Deserialize, Serialize};

/// `gccs.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gccs {
    /// Every column, oldest first.
    #[serde(rename = "gcc", default)]
    pub gccs: Vec<Gcc>,
}

/// One GCC column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Gcc {
    /// The column's name, as in `gcc-16.2.0`.
    pub id: String,
    /// The release.
    pub version: Version,
    /// `upstream` for a release tarball, or the distribution for a distribution package.
    pub flavor: String,
    /// The release date, `YYYY-MM-DD`, which the binutils pairing starts from. `gk pins` fills it in from the GNU mirror when it is left empty.
    #[serde(default)]
    pub released: String,
    /// The release tarball, for upstream columns.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// Its SHA-256.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sha256: String,
    /// Why the mirror has no detached signature for the tarball, when it has none. A tarball with a reason here is held to its SHA-256 pin alone, and every other one has to carry a good signature from `keys/gnu.asc` as well.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unsigned: String,
    /// The forge container that builds it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub forge: String,
    /// The GNU triples its bundle is built for.
    #[serde(default)]
    pub targets: Vec<String>,
    /// `auto` for the pairing rule of spec 04.5, or a binutils id.
    #[serde(default = "auto")]
    pub binutils: String,
    /// Why the column exists: `last-point`, `point` or `refinement`.
    pub columns: Vec<String>,
    /// The reason for a point column, in words.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub why: String,
}

fn auto() -> String {
    "auto".to_owned()
}

/// `binutils.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllBinutils {
    /// Every release, oldest first.
    #[serde(rename = "binutils", default)]
    pub releases: Vec<Binutils>,
}

/// One binutils release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Binutils {
    /// The name, as in `binutils-2.45`.
    pub id: String,
    /// The release.
    pub version: Version,
    /// The release date, `YYYY-MM-DD`.
    pub released: String,
    /// The release tarball.
    pub url: String,
    /// Its SHA-256.
    pub sha256: String,
    /// Why the mirror has no detached signature for the tarball, when it has none. A tarball with a reason here is held to its SHA-256 pin alone, and every other one has to carry a good signature from `keys/gnu.asc` as well.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unsigned: String,
}

impl Gccs {
    /// A column by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Gcc> {
        self.gccs.iter().find(|g| g.id == id)
    }
}

impl AllBinutils {
    /// A release by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Binutils> {
        self.releases.iter().find(|b| b.id == id)
    }

    /// The binutils a GCC is paired with by rules 1, 2 and 4 of spec 04.5: the newest release on or before the GCC's release date plus 90 days, raised to the kernel's own minimum when there is one.
    ///
    /// Rule 3, moving forward for a target the chosen binutils lacks, is the forge's to apply, because only building it shows that.
    #[must_use]
    pub fn pair(&self, gcc: &Gcc, kernel_minimum: Option<&Version>) -> Option<&Binutils> {
        if gcc.binutils != "auto" {
            return self.get(&gcc.binutils);
        }
        let limit = add_days(&gcc.released, 90)?;
        let by_time = self
            .releases
            .iter()
            .filter(|b| b.released.as_str() <= limit.as_str())
            .max_by(|a, b| a.version.cmp(&b.version))?;
        match kernel_minimum {
            Some(min) if by_time.version < *min => self
                .releases
                .iter()
                .filter(|b| b.version >= *min)
                .min_by(|a, b| a.version.cmp(&b.version)),
            _ => Some(by_time),
        }
    }
}

/// A `YYYY-MM-DD` date moved forward by some days, or `None` when it is not a date.
#[must_use]
pub fn add_days(date: &str, days: i64) -> Option<String> {
    let mut it = date.splitn(3, '-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let (y2, m2, d2) = civil_from_days(days_from_civil(y, m, d) + days);
    Some(format!("{y2:04}-{m2:02}-{d2:02}"))
}

// Howard Hinnant's day count algorithms, which need no calendar crate.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gcc(released: &str) -> Gcc {
        Gcc {
            id: "gcc-x".into(),
            version: "7.3.0".parse().unwrap(),
            flavor: "upstream".into(),
            released: released.into(),
            url: String::new(),
            sha256: String::new(),
            unsigned: String::new(),
            forge: String::new(),
            targets: vec![],
            binutils: "auto".into(),
            columns: vec!["point".into()],
            why: String::new(),
        }
    }

    fn all() -> AllBinutils {
        let b = |v: &str, d: &str| Binutils {
            id: format!("binutils-{v}"),
            version: v.parse().unwrap(),
            released: d.into(),
            url: String::new(),
            sha256: String::new(),
            unsigned: String::new(),
        };
        AllBinutils {
            releases: vec![
                b("2.29.1", "2017-09-25"),
                b("2.30", "2018-01-27"),
                b("2.31.1", "2018-07-18"),
                b("2.36.1", "2021-02-06"),
            ],
        }
    }

    #[test]
    fn days_are_added_across_months_and_leap_years() {
        assert_eq!(add_days("2018-01-25", 90).unwrap(), "2018-04-25");
        assert_eq!(add_days("2024-02-28", 1).unwrap(), "2024-02-29");
        assert_eq!(add_days("2023-12-31", 1).unwrap(), "2024-01-01");
        assert!(add_days("2024-13-01", 1).is_none());
    }

    #[test]
    fn a_2018_gcc_is_kept_off_binutils_2_31() {
        // GCC 7.3 came out on 2018-01-25, and 2.31 would break x86_64 kernels before 4.16 (spec 04.5).
        let all = all();
        assert_eq!(
            all.pair(&gcc("2018-01-25"), None).unwrap().id,
            "binutils-2.30"
        );
    }

    #[test]
    fn the_kernel_minimum_raises_the_pair() {
        let all = all();
        let min: Version = "2.30".parse().unwrap();
        assert_eq!(
            all.pair(&gcc("2017-08-14"), None).unwrap().id,
            "binutils-2.29.1"
        );
        assert_eq!(
            all.pair(&gcc("2017-08-14"), Some(&min)).unwrap().id,
            "binutils-2.30"
        );
    }

    #[test]
    fn an_explicit_pair_wins() {
        let mut g = gcc("2018-01-25");
        g.binutils = "binutils-2.36.1".into();
        assert_eq!(all().pair(&g, None).unwrap().id, "binutils-2.36.1");
    }
}
