//! What the GNU mirror publishes: the directory listings of GCC and binutils releases, with their dates.
//!
//! The GNU mirror has no hash lists. Each tarball has a detached signature, which `gk fetch` checks, and `gk pins` downloads a tarball once to take its SHA-256.

/// The GCC release directory.
pub const GCC_URL: &str = "https://ftp.gnu.org/gnu/gcc";

/// The binutils release directory.
pub const BINUTILS_URL: &str = "https://ftp.gnu.org/gnu/binutils";

/// One entry of a directory listing: the link target without a trailing slash, and the date next to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The name, as in `gcc-16.2.0` or `binutils-2.47.tar.xz`.
    pub name: String,
    /// The date the listing shows, `YYYY-MM-DD`.
    pub date: String,
}

/// Parse an Apache directory listing. Each row has an `href` and then a date, and rows without a date, such as the parent link, are skipped.
#[must_use]
pub fn parse_listing(html: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    for row in html.split("href=\"").skip(1) {
        let Some((target, rest)) = row.split_once('"') else {
            continue;
        };
        let rest = rest.split("href=\"").next().unwrap_or_default();
        if let Some(date) = first_date(rest) {
            out.push(Entry {
                name: target.trim_end_matches('/').to_owned(),
                date,
            });
        }
    }
    out
}

fn first_date(s: &str) -> Option<String> {
    let b = s.as_bytes();
    (0..b.len().saturating_sub(9)).find_map(|i| {
        let w = &b[i..i + 10];
        let shape = w.iter().enumerate().all(|(j, c)| {
            if j == 4 || j == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        });
        shape.then(|| s[i..i + 10].to_owned())
    })
}

/// The GCC release directories, as version and date.
#[must_use]
pub fn gcc_releases(listing: &[Entry]) -> Vec<(String, String)> {
    listing
        .iter()
        .filter_map(|e| {
            let v = e.name.strip_prefix("gcc-")?;
            v.bytes()
                .all(|c| c.is_ascii_digit() || c == b'.')
                .then(|| (v.to_owned(), e.date.clone()))
        })
        .collect()
}

/// The binutils `.tar.xz` releases, as version and date.
#[must_use]
pub fn binutils_releases(listing: &[Entry]) -> Vec<(String, String)> {
    listing
        .iter()
        .filter_map(|e| {
            let v = e.name.strip_prefix("binutils-")?.strip_suffix(".tar.xz")?;
            v.bytes()
                .all(|c| c.is_ascii_digit() || c == b'.')
                .then(|| (v.to_owned(), e.date.clone()))
        })
        .collect()
}

/// The URL of a GCC release tarball.
#[must_use]
pub fn gcc_url(version: &str) -> String {
    format!("{GCC_URL}/gcc-{version}/gcc-{version}.tar.xz")
}

/// The URL of a binutils release tarball.
#[must_use]
pub fn binutils_url(version: &str) -> String {
    format!("{BINUTILS_URL}/binutils-{version}.tar.xz")
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTING: &str = r#"<tr><td><a href="/gnu/">Parent Directory</a></td><td>&nbsp;</td></tr>
<tr><td valign="top"><img src="/icons/folder.gif" alt="[DIR]"></td><td><a href="gcc-16.1.0/">gcc-16.1.0/</a></td><td align="right">2026-04-30 05:30  </td><td align="right">  - </td></tr>
<tr><td valign="top"><img src="/icons/folder.gif" alt="[DIR]"></td><td><a href="gcc-16.2.0/">gcc-16.2.0/</a></td><td align="right">2026-08-07 04:40  </td><td align="right">  - </td></tr>
<tr><td><a href="binutils-2.47.tar.xz">binutils-2.47.tar.xz</a></td><td align="right">2026-07-20 10:01  </td></tr>
<tr><td><a href="binutils-2.47.tar.xz.sig">binutils-2.47.tar.xz.sig</a></td><td align="right">2026-07-20 10:01  </td></tr>"#;

    #[test]
    fn listings_read_as_names_and_dates() {
        let entries = parse_listing(LISTING);
        assert_eq!(entries.len(), 4);
        assert_eq!(
            gcc_releases(&entries),
            [
                ("16.1.0".to_owned(), "2026-04-30".to_owned()),
                ("16.2.0".to_owned(), "2026-08-07".to_owned())
            ]
        );
        assert_eq!(
            binutils_releases(&entries),
            [("2.47".to_owned(), "2026-07-20".to_owned())]
        );
    }
}
