//! Signature checks with gpg, against the keys committed in `keys/`.
//!
//! Each key file gets its own gpg home under the cache, so nothing in the user's own keyring is trusted and nothing is added to it. A signature counts when gpg reports `VALIDSIG`, which it does for a good signature by any key in the home whatever the trust settings say.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A gpg home holding the keys of one committed key file.
pub struct Keyring {
    home: PathBuf,
}

impl Keyring {
    /// Make or refresh the home for `keys` under `cache`.
    pub fn open(keys: &Path, cache: &Path) -> Result<Self, String> {
        let stem = keys.file_stem().and_then(|s| s.to_str()).unwrap_or("keys");
        let home = cache.join("gnupg").join(stem);
        std::fs::create_dir_all(&home).map_err(|e| format!("creating {}: {e}", home.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o700));
        }
        let out = Command::new("gpg")
            .env("GNUPGHOME", &home)
            .args(["--batch", "--quiet", "--import"])
            .arg(keys)
            .output()
            .map_err(|e| format!("running gpg: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "importing {}: {}",
                keys.display(),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(Keyring { home })
    }

    /// Check a detached signature over a file, and return the fingerprint that signed it.
    pub fn verify_detached(&self, sig: &Path, file: &Path) -> Result<String, String> {
        let status = self.home.join("status");
        let out = Command::new("gpg")
            .env("GNUPGHOME", &self.home)
            .args(["--batch", "--status-file"])
            .arg(&status)
            .arg("--verify")
            .arg(sig)
            .arg(file)
            .output()
            .map_err(|e| format!("running gpg: {e}"))?;
        valid(&status, out.status.success())
            .ok_or_else(|| format!("{} has no good signature from keys/", file.display()))
    }

    /// Check a clear signed file and return the text that was signed, with the fingerprint that signed it. Only the signed text is returned, so lines added around the signature are never read.
    pub fn verify_clear(&self, file: &Path) -> Result<(String, String), String> {
        let status = self.home.join("status");
        let out = Command::new("gpg")
            .env("GNUPGHOME", &self.home)
            .args(["--batch", "--status-file"])
            .arg(&status)
            .args(["--output", "-", "--decrypt"])
            .arg(file)
            .output()
            .map_err(|e| format!("running gpg: {e}"))?;
        let fpr = valid(&status, out.status.success())
            .ok_or_else(|| format!("{} has no good signature from keys/", file.display()))?;
        let text =
            String::from_utf8(out.stdout).map_err(|_| format!("{} is not text", file.display()))?;
        Ok((text, fpr))
    }
}

fn valid(status: &Path, ok: bool) -> Option<String> {
    let text = std::fs::read_to_string(status).ok()?;
    let fpr = text
        .lines()
        .find_map(|l| l.strip_prefix("[GNUPG:] VALIDSIG "))?
        .split(' ')
        .next()?
        .to_owned();
    ok.then_some(fpr)
}
