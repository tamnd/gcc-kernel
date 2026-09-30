//! Downloads, through curl, and hashing.
//!
//! curl is on every machine that runs `gk`, and using it keeps an HTTP and TLS stack out of the dependency tree.

use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use std::process::Command;

/// Download a URL to a file, failing on any HTTP error. The file appears only when the download is complete.
pub fn download(url: &str, to: &Path) -> Result<(), String> {
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }
    let partial = to.with_extension("part");
    let status = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-o"])
        .arg(&partial)
        .arg(url)
        .status()
        .map_err(|e| format!("running curl: {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_file(&partial);
        return Err(format!("downloading {url} failed"));
    }
    std::fs::rename(&partial, to).map_err(|e| format!("moving {}: {e}", partial.display()))
}

/// Download a URL and return its text.
pub fn fetch_text(url: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--retry", "3", url])
        .output()
        .map_err(|e| format!("running curl: {e}"))?;
    if !out.status.success() {
        return Err(format!("downloading {url} failed"));
    }
    String::from_utf8(out.stdout).map_err(|_| format!("{url} is not text"))
}

/// The lower case hex SHA-256 of a file.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("opening {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
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
