//! `gk hosts check`: run every host and forge container and compare its tools with `hosts.toml`.

use gk_model::repo::Repo;
use std::process::Command;

/// What a container reports, one tool per line, in the order gcc, binutils, make, qemu: the first version number on the first line of each tool's `--version`, which is the one the tool calls its own in every release from hamm on. `-dumpversion` is no use for this, since wheezy's says `4.7` for 4.7.2. A tool the image lacks prints `-`.
const PROBE: &str = "v() { sed -n '1s/^[^0-9]*\\([0-9][0-9.]*[0-9]\\).*/\\1/p' | grep . || echo -; }; \
gcc --version 2>/dev/null | v; \
ld --version 2>/dev/null | v; \
make --version 2>/dev/null | v; \
qemu-system-x86_64 --version 2>/dev/null | v";

/// Check every host that has an image or a Dockerfile. Returns whether they all agree.
pub fn check(repo: &Repo) -> Result<bool, String> {
    let mut ok = true;
    let mut checked = 0;
    for host in &repo.hosts.hosts {
        if !host.is_built() && !repo.root.join(host.dockerfile()).is_file() {
            continue;
        }
        let image = crate::forge::image_for(repo, &host.name)?;
        let out = Command::new("docker")
            .args(["run", "--rm", "--network=none", &image, "sh", "-c", PROBE])
            .output()
            .map_err(|e| format!("running docker: {e}"))?;
        if !out.status.success() {
            println!("{}: the container did not run", host.name);
            ok = false;
            continue;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let got: Vec<&str> = text.lines().map(str::trim).collect();
        let want = [&host.gcc, &host.binutils, &host.make, &host.qemu];
        let mut bad = Vec::new();
        for (i, tool) in ["gcc", "binutils", "make", "qemu"].iter().enumerate() {
            let have = got.get(i).copied().unwrap_or("-");
            if !want[i].is_empty() && want[i] != have {
                bad.push(format!("{tool} is {have}, hosts.toml says {}", want[i]));
            }
        }
        checked += 1;
        if bad.is_empty() {
            println!("{}: ok ({image})", host.name);
        } else {
            println!("{}: {}", host.name, bad.join(", "));
            ok = false;
        }
    }
    println!("{checked} containers checked");
    Ok(ok)
}
