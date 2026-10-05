//! `gk publish --html`: the static site of spec 10.7, the same heat maps as `reports/` as plain HTML with no script, for GitHub Pages.
//!
//! The site is an index with the verdict counts and one page per platform. Each square's title says the cell's rung, class and first error, so hovering over a red square says why it is red. `matrix.json` is copied next to the pages so the site is the whole product on its own.

use crate::publish::{Entry, Matrix};
use gk_model::Version;
use gk_model::repo::Repo;
use std::fmt::Write as _;
use std::path::Path;

const STYLE: &str = "body{font:14px/1.4 system-ui,sans-serif;margin:2em;color:#222}
table{border-collapse:collapse;margin:1em 0}
th,td{border:1px solid #ddd;padding:2px 4px;text-align:center;white-space:nowrap}
th{background:#f4f4f4;font-weight:normal}
td.k{text-align:left}
td.works{background:#3fb950}
td.museum{background:#a6e3b0}
td.runs{background:#e3b341}
td.builds{background:#f0883e}
td.fails{background:#f85149}
td.na{color:#999}
td.flaky{outline:2px dashed #000;outline-offset:-3px}
.wrap{overflow-x:auto}
code{background:#f4f4f4;padding:0 3px}";

/// Escape text for HTML.
fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

fn page(title: &str, body: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{}</title>\n<style>\n{STYLE}\n</style>\n</head>\n<body>\n{body}</body>\n</html>\n",
        esc(title)
    )
}

/// What a square says when hovered.
fn title(e: &Entry) -> String {
    let mut t = format!(
        "{} {} {}: {} at {}",
        e.kernel, e.gcc, e.config, e.verdict, e.rung
    );
    if !e.class.is_empty() {
        let _ = write!(t, ", {}", e.class);
    }
    if !e.first_error.is_empty() {
        let _ = write!(t, "\n{}", e.first_error);
    }
    if e.flaky {
        t.push_str("\nflaky: the boots disagreed");
    }
    t
}

/// The page of one platform, or `None` when it has no cells.
fn platform_page(repo: &Repo, m: &Matrix, platform: &str) -> Option<String> {
    let p = repo.platforms.get(platform)?;
    let cells: Vec<&Entry> = m.cells.iter().filter(|e| e.platform == platform).collect();
    if cells.is_empty() {
        return None;
    }
    let mut gccs: Vec<(&str, &Version)> = repo
        .gccs
        .gccs
        .iter()
        .filter(|g| g.flavor == "upstream" && g.targets.contains(&p.triple))
        .map(|g| (g.id.as_str(), &g.version))
        .collect();
    gccs.sort_by(|a, b| a.1.cmp(b.1));
    let mut configs: Vec<&str> = cells.iter().map(|e| e.config.as_str()).collect();
    configs.sort_unstable();
    configs.dedup();
    let mut body = format!(
        "<p><a href=\"index.html\">gcc-kernel</a></p>\n<h1>{}</h1>\n<p>A row per kernel and a column per GCC. Green works, light green works on the smaller museum suite of a kernel before 2.6, yellow runs, orange builds, red fails, a dot is n/a, and an empty square has not run yet. A dashed square is flaky. Hover over a square for its rung, class and first error.</p>\n",
        esc(platform)
    );
    for config in configs {
        let mut kernels: Vec<Version> = cells
            .iter()
            .filter(|e| e.config == config)
            .filter_map(|e| e.kernel.parse().ok())
            .collect();
        kernels.sort();
        kernels.dedup();
        let _ = write!(
            body,
            "<h2>{}</h2>\n<div class=\"wrap\"><table>\n<tr><th>Kernel</th>",
            esc(config)
        );
        for (_, v) in &gccs {
            let _ = write!(body, "<th>{}</th>", esc(v.as_str()));
        }
        body.push_str("</tr>\n");
        for k in &kernels {
            let _ = write!(body, "<tr><td class=\"k\">{}</td>", esc(k.as_str()));
            for (id, _) in &gccs {
                let cell = cells
                    .iter()
                    .filter(|e| {
                        e.config == config
                            && e.gcc == *id
                            && e.kernel.parse::<Version>().ok().as_ref() == Some(k)
                    })
                    .max_by_key(|e| e.started);
                match cell {
                    Some(e) => {
                        let class = match e.verdict.as_str() {
                            "works" if crate::publish::museum_works(e) => "museum",
                            v @ ("works" | "runs" | "builds" | "fails") => v,
                            _ => "na",
                        };
                        let flaky = if e.flaky { " flaky" } else { "" };
                        let mark = if class == "na" { "·" } else { "" };
                        let _ = write!(
                            body,
                            "<td class=\"{class}{flaky}\" title=\"{}\">{mark}</td>",
                            esc(&title(e))
                        );
                    }
                    None => body.push_str("<td></td>"),
                }
            }
            body.push_str("</tr>\n");
        }
        body.push_str("</table></div>\n");
    }
    Some(page(&format!("gcc-kernel: {platform}"), &body))
}

fn index(repo: &Repo, m: &Matrix, platforms: &[&str], date: &str) -> String {
    let mut body = String::from(
        "<h1>gcc-kernel</h1>\n<p>Which GCC releases build and boot which Linux kernels, cell by cell. Each cell is one kernel, one GCC, one platform and one configuration, built in a pinned container and booted in QEMU. The source and the method are at <a href=\"https://github.com/tamnd/gcc-kernel\">github.com/tamnd/gcc-kernel</a>, and the data is <a href=\"matrix.json\">matrix.json</a>.</p>\n",
    );
    let _ = writeln!(
        body,
        "<p>{} cells, published {}.</p>",
        m.cells.len(),
        esc(date)
    );
    body.push_str("<table>\n<tr><th>Platform</th><th>works</th><th>runs</th><th>builds</th><th>fails</th><th>n/a</th></tr>\n");
    for p in platforms {
        let count = |v: &str| {
            m.cells
                .iter()
                .filter(|e| e.platform == *p && e.verdict == v)
                .count()
        };
        let _ = writeln!(
            body,
            "<tr><td class=\"k\"><a href=\"{p}.html\">{}</a></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            esc(p),
            count("works"),
            count("runs"),
            count("builds"),
            count("fails"),
            count("n/a")
        );
    }
    body.push_str("</table>\n<p>The reports, in markdown:</p>\n<ul>\n");
    let reports = repo.root.join("reports");
    let mut names: Vec<String> = std::fs::read_dir(&reports)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| Path::new(n).extension().is_some_and(|x| x == "md"))
        .collect();
    names.sort();
    for n in names {
        let _ = writeln!(
            body,
            "<li><a href=\"https://github.com/tamnd/gcc-kernel/blob/main/reports/{0}\">{0}</a></li>",
            esc(&n)
        );
    }
    body.push_str("</ul>\n");
    page("gcc-kernel", &body)
}

/// Write the site into `dir`. Returns the files written.
pub fn write(repo: &Repo, m: &Matrix, dir: &Path, date: &str) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let mut written = Vec::new();
    let mut platforms = Vec::new();
    let mut put = |name: String, text: String| {
        std::fs::write(dir.join(&name), text).map_err(|e| format!("writing {name}: {e}"))?;
        written.push(dir.join(name).display().to_string());
        Ok::<(), String>(())
    };
    for p in &repo.platforms.platforms {
        if let Some(text) = platform_page(repo, m, &p.name) {
            put(format!("{}.html", p.name), text)?;
            platforms.push(p.name.as_str());
        }
    }
    put("index.html".into(), index(repo, m, &platforms, date))?;
    put(
        "matrix.json".into(),
        serde_json::to_string(m).map_err(|e| e.to_string())? + "\n",
    )?;
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(kernel: &str, gcc: &str, verdict: &str) -> Entry {
        serde_json::from_value(serde_json::json!({
            "cell": "", "kernel": kernel, "gcc": gcc, "binutils": "", "platform": "x86_64",
            "config": "defconfig+gk", "host": "", "rung": "L3", "verdict": verdict, "runs": 1,
            "gk": "", "date": "", "seconds": 0.0, "machine": "",
            "first-error": "fs/x.c:1: error: <bad> & \"worse\""
        }))
        .unwrap()
    }

    #[test]
    fn a_platform_page_has_a_square_per_cell_and_escapes_the_error() {
        let repo = Repo::load(Path::new("../..")).unwrap();
        let m = Matrix {
            schema: 1,
            cells: vec![
                cell("7.2.8", "gcc-16.2.0", "fails"),
                cell("7.2.8", "gcc-14.2.0", "works"),
            ],
        };
        let html = platform_page(&repo, &m, "x86_64").unwrap();
        assert_eq!(html.matches("<td class=\"fails\"").count(), 1);
        assert_eq!(html.matches("<td class=\"works\"").count(), 1);
        assert!(html.contains("&lt;bad&gt; &amp; &quot;worse&quot;"));
        assert!(!html.contains("<script"));
        assert!(platform_page(&repo, &m, "arm64").is_none());
        let index = index(&repo, &m, &["x86_64"], "2026-10-03");
        assert!(index.contains(
            "<a href=\"x86_64.html\">x86_64</a></td><td>1</td><td>0</td><td>0</td><td>1</td>"
        ));
    }
}
