//! The result store of spec 10.6: a directory tree keyed by cell identity.
//!
//! A cell lives at `cells/<hex>`, where `<hex>` is its identity without the `sha256:` prefix, and is complete once `cell.json` is there, because `gk cell` writes that file last. The store is wherever `GK_STORE` points, or `store` in the cache. It lives on server3 for now, as gpc is not up often enough to hold it. `gk store pack` writes the tarball that a sweep publishes as a release asset.

use crate::cell::CellRecord;
use crate::fetch;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The store's root: `GK_STORE`, or `store` in the cache.
#[must_use]
pub fn root() -> PathBuf {
    std::env::var_os("GK_STORE").map_or_else(|| fetch::cache_dir().join("store"), PathBuf::from)
}

/// Where a cell's directory goes.
#[must_use]
pub fn cell_dir(identity: &str) -> PathBuf {
    root()
        .join("cells")
        .join(identity.strip_prefix("sha256:").unwrap_or(identity))
}

/// Every complete cell, oldest first, with its directory.
pub fn cells() -> Result<Vec<(PathBuf, CellRecord)>, String> {
    let dir = root().join("cells");
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("reading {}: {e}", dir.display())),
    };
    let mut out = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|e| format!("reading {}: {e}", dir.display()))?
            .path();
        let Ok(text) = std::fs::read_to_string(path.join("cell.json")) else {
            continue;
        };
        let record: CellRecord = serde_json::from_str(&text)
            .map_err(|e| format!("{}/cell.json: {e}", path.display()))?;
        out.push((path, record));
    }
    out.sort_by_key(|(_, r)| r.started);
    Ok(out)
}

/// The cell whose identity starts with `prefix`, which may carry the `sha256:` prefix.
pub fn find(prefix: &str) -> Result<(PathBuf, CellRecord), String> {
    let want = prefix.strip_prefix("sha256:").unwrap_or(prefix);
    let mut hits: Vec<_> = cells()?
        .into_iter()
        .filter(|(_, r)| r.cell["sha256:".len()..].starts_with(want))
        .collect();
    match hits.len() {
        0 => Err(format!("no cell in the store starts with {want}")),
        1 => Ok(hits.remove(0)),
        n => Err(format!(
            "{n} cells start with {want}; give more of the identity"
        )),
    }
}

/// What is wrong with the store: a directory whose name is not the identity in its `cell.json`, or one that has no `cell.json`, which is a cell that died before it finished.
pub fn check() -> Result<Vec<String>, String> {
    let dir = root().join("cells");
    let mut problems = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(problems);
    };
    for entry in entries {
        let path = entry
            .map_err(|e| format!("reading {}: {e}", dir.display()))?
            .path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match std::fs::read_to_string(path.join("cell.json")) {
            Err(_) => problems.push(format!("{name}: no cell.json, so the cell did not finish")),
            Ok(text) => match serde_json::from_str::<CellRecord>(&text) {
                Err(e) => problems.push(format!("{name}: cell.json does not read: {e}")),
                Ok(r) if r.cell.strip_prefix("sha256:") != Some(name.as_str()) => {
                    problems.push(format!("{name}: cell.json says it is {}", r.cell));
                }
                Ok(r) if r.coordinates.identity() != r.cell => {
                    problems.push(format!(
                        "{name}: its coordinates hash to {}",
                        r.coordinates.identity()
                    ));
                }
                Ok(_) => {}
            },
        }
    }
    problems.sort();
    Ok(problems)
}

/// Write the complete cells into `out` as a zstd tarball, the form a sweep publishes.
pub fn pack(out: &Path) -> Result<usize, String> {
    let cells = cells()?;
    let names: Vec<String> = cells
        .iter()
        .filter_map(|(p, _)| {
            p.file_name()
                .map(|n| format!("cells/{}", n.to_string_lossy()))
        })
        .collect();
    let list = std::env::temp_dir().join(format!("gk-pack-{}.txt", std::process::id()));
    std::fs::write(&list, names.join("\n") + "\n")
        .map_err(|e| format!("writing {}: {e}", list.display()))?;
    let status = Command::new("tar")
        .arg("-C")
        .arg(root())
        .args(["--sort=name", "--owner=0", "--group=0", "--numeric-owner"])
        // Two threads: at level 19 each one holds hundreds of megabytes, and with one per core the packing machine ran out of memory.
        .args(["--use-compress-program", "zstd -19 -T2"])
        .arg("-cf")
        .arg(out)
        .arg("-T")
        .arg(&list)
        .status();
    let _ = std::fs::remove_file(&list);
    match status {
        Ok(s) if s.success() => Ok(cells.len()),
        Ok(s) => Err(format!("tar exited with {s}")),
        Err(e) => Err(format!("running tar: {e}")),
    }
}
