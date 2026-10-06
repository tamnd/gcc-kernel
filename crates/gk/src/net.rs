//! Downloads, through curl, and hashing.
//!
//! curl is on every machine that runs `gk`, and using it keeps an HTTP and TLS stack out of the dependency tree.

use sha2::{Digest, Sha256, Sha512};
use std::io::Read;
use std::path::Path;
use std::process::Command;

/// The GNU master site, and the mirror that is tried when it cannot be reached.
const GNU: &str = "https://ftp.gnu.org/gnu/";
const GNU_MIRROR: &str = "https://mirrors.kernel.org/gnu/";

/// The URLs to try for `url`, in order: itself, and for a file on the GNU master site the same path on the mirror. A file from the mirror is checked against its signature or its pin like any other, so the mirror is trusted no more than the master site.
fn sources(url: &str) -> Vec<String> {
    let mut out = vec![url.to_owned()];
    if let Some(path) = url.strip_prefix(GNU) {
        out.push(format!("{GNU_MIRROR}{path}"));
    }
    out
}

/// Download a URL to a file, failing on any HTTP error. The file appears only when the download is complete.
pub fn download(url: &str, to: &Path) -> Result<(), String> {
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }
    let partial = to.with_extension("part");
    for source in sources(url) {
        let status = Command::new("curl")
            .args(["-fsSL", "--retry", "3", "--connect-timeout", "30", "-o"])
            .arg(&partial)
            .arg(&source)
            .status()
            .map_err(|e| format!("running curl: {e}"))?;
        if status.success() {
            return std::fs::rename(&partial, to)
                .map_err(|e| format!("moving {}: {e}", partial.display()));
        }
        let _ = std::fs::remove_file(&partial);
    }
    Err(format!("downloading {url} failed"))
}

/// Download a URL and return its text. Only the URL itself is tried, since the directory listings `gk pins` reads are in the master site's format, which the mirror does not keep.
pub fn fetch_text(url: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "--connect-timeout", "30", url])
        .output()
        .map_err(|e| format!("running curl: {e}"))?;
    if !out.status.success() {
        return Err(format!("downloading {url} failed"));
    }
    String::from_utf8(out.stdout).map_err(|_| format!("{url} is not text"))
}

/// The lower case hex SHA-256 of a file.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    hash_file::<Sha256>(path)
}

/// The lower case hex SHA-256 of bytes in memory.
#[must_use]
pub fn sha256_bytes(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// The lower case hex SHA-512 of a file, which is what GCC's `prerequisites.sha512` lists.
pub fn sha512_file(path: &Path) -> Result<String, String> {
    hash_file::<Sha512>(path)
}

fn hash_file<D: Digest>(path: &Path) -> Result<String, String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("opening {}: {e}", path.display()))?;
    let mut hasher = D::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gnu_file_falls_back_to_the_mirror() {
        assert_eq!(
            sources("https://ftp.gnu.org/gnu/binutils/binutils-2.37.tar.xz.sig"),
            [
                "https://ftp.gnu.org/gnu/binutils/binutils-2.37.tar.xz.sig",
                "https://mirrors.kernel.org/gnu/binutils/binutils-2.37.tar.xz.sig"
            ]
        );
        assert_eq!(
            sources("https://cdn.kernel.org/pub/linux/kernel/v6.x/sha256sums.asc"),
            ["https://cdn.kernel.org/pub/linux/kernel/v6.x/sha256sums.asc"]
        );
    }
}
