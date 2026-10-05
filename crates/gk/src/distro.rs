//! `gk report distributions`: the distribution columns of spec 04.3 against upstream.
//!
//! A distribution column is a GCC as a distribution shipped it, run in that distribution's own image with its own binutils. Its defaults are where it differs from the upstream release it was built from, so each of its cells is set beside the upstream column of the same major series on the same kernel, platform and configuration. The report ends with the changed defaults of `signatures.toml` and the cells that reproduce each one.

use crate::publish::Entry;
use crate::report::Cell;
use gk_model::Version;
use gk_model::repo::Repo;
use std::fmt::Write as _;

/// The newest cell of `gcc` on a kernel, platform and configuration.
fn newest<'a>(cells: &'a [Cell], gcc: &str, kernel: &str, platform: &str, config: &str) -> Option<&'a Entry> {
    cells
        .iter()
        .map(|c| &c.entry)
        .filter(|e| e.gcc == gcc && e.kernel == kernel && e.platform == platform && e.config == config)
        .max_by(|a, b| a.started.cmp(&b.started))
}

/// A verdict with the rung it got to, as `fails at L2`.
fn verdict(e: &Entry) -> String {
    if e.rung.is_empty() {
        e.verdict.clone()
    } else {
        format!("{} at {}", e.verdict, e.rung)
    }
}

/// The report, from the cells given.
#[must_use]
pub fn report(repo: &Repo, cells: &[Cell]) -> String {
    let mut out = String::new();
    out.push_str("# Distribution columns\n\n");
    out.push_str("A distribution column is a GCC as a distribution shipped it, run in that distribution's own image with its own binutils (spec 04.3). What sets it apart from the upstream release it was built from is its defaults, such as PIE, the stack protector and `-fcf-protection`, and this report shows which kernels notice them. Each row is the newest cell of the column, beside the newest cell of the upstream release it was built from, or of the newest upstream release of the same major series when the matrix lacks that one, on the same kernel, platform and configuration.\n\n");
    out.push_str("Written by `gk report distributions` from the result store. `gk cell K <column> --platform P` runs a row.\n");
    for g in repo.gccs.gccs.iter().filter(|g| g.flavor != "upstream") {
        let _ = write!(out, "\n## {}, {} {} {}\n\n", g.id, g.flavor, g.package, g.version);
        if !g.why.is_empty() {
            let _ = write!(out, "{}\n\n", g.why);
        }
        // The release it was built from when the matrix has it, or else the newest of the same major series.
        let series = || repo.gccs.gccs.iter().filter(|u| u.flavor == "upstream" && u.version.parts().first() == g.version.parts().first());
        let upstream = series().find(|u| u.version == g.version).or_else(|| series().max_by(|a, b| a.version.cmp(&b.version)));
        let mut rows: Vec<&Entry> = Vec::new();
        for c in cells.iter().filter(|c| c.entry.gcc == g.id) {
            let e = &c.entry;
            if rows.iter().any(|r| r.kernel == e.kernel && r.platform == e.platform && r.config == e.config) {
                continue;
            }
            if let Some(n) = newest(cells, &g.id, &e.kernel, &e.platform, &e.config) {
                rows.push(n);
            }
        }
        if rows.is_empty() {
            out.push_str("No cell has run yet.\n");
            continue;
        }
        rows.sort_by(|a, b| {
            let key = |e: &Entry| (e.platform.clone(), e.config.clone(), e.kernel.parse::<Version>().ok());
            key(a).cmp(&key(b))
        });
        let up = upstream.map_or("upstream", |u| u.id.as_str());
        let _ = writeln!(out, "| Kernel | Platform | Configuration | Verdict | Class | {up} |");
        out.push_str("|---|---|---|---|---|---|\n");
        for e in rows {
            let class = if e.class.is_empty() {
                String::new()
            } else {
                format!("`{}`", e.class)
            };
            let theirs = upstream
                .and_then(|u| newest(cells, &u.id, &e.kernel, &e.platform, &e.config))
                .map_or_else(|| "not run".to_owned(), verdict);
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {class} | {theirs} |",
                e.kernel,
                e.platform,
                e.config,
                verdict(e)
            );
        }
    }
    out.push_str("\n## Changed defaults\n\nThe signatures of kind `default-change`, and the cells of any column classified under each.\n\n| Class | Flavors | GCC | Fixed by | Reproduced on |\n|---|---|---|---|---|\n");
    for s in repo.signatures.signatures.iter().filter(|s| s.kind == "default-change") {
        let fixes: Vec<String> = s
            .fixed_by
            .iter()
            .map(|f| {
                if f.first_tag.is_empty() {
                    format!("`{}`", f.commit)
                } else {
                    format!("`{}` ({})", f.commit, f.first_tag)
                }
            })
            .collect();
        let mut seen: Vec<String> = cells
            .iter()
            .filter(|c| c.entry.class == s.class)
            .map(|c| format!("{} {}", c.entry.kernel, c.entry.gcc))
            .collect();
        seen.sort();
        seen.dedup();
        let reproduced = if seen.is_empty() {
            "not yet".to_owned()
        } else {
            seen.join(", ")
        };
        let flavors = if s.flavor.is_empty() {
            "any".to_owned()
        } else {
            s.flavor.join(", ")
        };
        let _ = writeln!(
            out,
            "| `{}` | {flavors} | {} | {} | {reproduced} |",
            s.class,
            s.gcc,
            fixes.join(", ")
        );
    }
    out
}
