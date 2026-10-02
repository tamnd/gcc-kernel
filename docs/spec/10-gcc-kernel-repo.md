# The `tamnd/gcc-kernel` repository

`gcc-kernel` pins kernel trees, GCC and binutils releases, host environments and platforms, builds kernels with every pairing the search of document 09 chooses, boots them, and publishes the matrix. It has the shape of `rucc-kernel`, deliberately: the same language, the same file conventions, the same shim schema. Whoever knows one should find their way around the other, and the tools of one should read the other's build directories.

## 10.1 What it may and may not contain

- **No compiler code, and no rucc.** Nothing here compiles C except the toolchains it pins, and rucc appears nowhere: not in a container, not in a workflow, not as a dependency. This is what makes it an independent reference.
- **No kernel patches and no copied kernel files.** Trees are fetched by hash. The fixes that make an old kernel build with a new GCC are named in `signatures.toml` by commit hash and never applied.
- **No toolchain patches.** Upstream GCC and binutils are built from release tarballs by hash. The distribution flavor uses the distribution's own packages, by hash.
- **No hidden configuration.** Everything that changes a cell is in a committed file: kernels, toolchains, hosts, platforms, configs, signatures, the search seed.
- **No results in git beyond the summary.** The matrix, the reports and the signature catalog are committed. Logs and consoles live in release assets and the result store (10.6).

## 10.2 Layout

```
gcc-kernel/
  Cargo.toml                 workspace, Rust 1.98, edition 2024, Apache-2.0
  rust-toolchain.toml        the same pin as rucc-kernel
  crates/
    gk/                      the CLI
    gk-cc/                   the shim: records every compiler call, changes nothing (10.4)
    gk-model/                kernels, toolchains, hosts, platforms, cells: parsing and identity
    gk-forge/                toolchain build recipes and bundle packing (04.4)
    gk-run/                  container runner, build driver, twins
    gk-boot/                 QEMU rig, serial framing, boot budgets
    gk-tap/                  KUnit TAP parser (the same parser as rucc-kernel's, copied)
    gk-sig/                  failure signatures and classification (08)
    gk-search/               frontier search and sampling (09)
    gk-report/               matrix.json, markdown and HTML reports
  gk-init/
    init.c                   PID 1 for 2.6 on, built once per platform by the era GCC
    init-museum.S            PID 1 for 1.x to 2.4, no libc, int $0x80
    init-0x.S                the 0.x boot check (07.7)
  kernels.toml               every tree: version, url, sha256, era, sets (03)
  sets.toml                  the rules that expand into kernels.toml
  eras.toml                  kernel eras: era GCC, era distribution, host environment (03.3, 05)
  gccs.toml                  every GCC column: version, flavor, tarball, sha256, targets, configure options (04)
  binutils.toml              every binutils: version, tarball, sha256, pairing date (04.5)
  hosts.toml                 host environments and forge containers, by digest (05)
  platforms.toml             arch, QEMU machine per era, cpu, console, budgets, tiers (06)
  qemu.toml                  the pinned QEMU builds (06.5)
  configs/
    fragment.<era>           the test fragment per era (07.2)
  signatures.toml            failure signatures with fix commits (08)
  exclusions.toml            cells excluded with a reason and evidence
  provision/
    hosts/<name>/Dockerfile  host environments
    forge/<name>/Dockerfile  forge environments for toolchains
    machines/                gpc, server1..3 setup: KVM, disk layout, runner service
  matrix/
    matrix.json              the published matrix, one record per cell with a verdict (10.7)
    frontiers.json           min, max, holes per kernel and platform
    ranges.json              kernel range per GCC column and platform
  reports/
    matrix-<platform>.md     the heat map tables
    gcc-<version>.md         one page per GCC column
    kernel-<version>.md      one page per kernel
    new-gcc-<version>.md     the release report for a newly added GCC (10.8)
    sweep.md                 the last sweep: counts, holes, flakes, changes
  .github/workflows/
    pr.yml                   harness tests, style, the accept probe on changed pins
    forge.yml                builds toolchain bundles and containers, manual and on new pins
    watch.yml                daily: new kernel tags, new GCC and binutils releases, opens pins PRs
    sweep.yml                dispatches searched sweeps to the self-hosted machines
    publish.yml              regenerates matrix/ and reports/ from the result store
```

## 10.3 Commands

| Command | Does |
|---|---|
| `gk pins` | expand `sets.toml` against kernel.org, list new GCC and binutils releases from the GNU mirrors, print the diff; `--write` updates the pin files |
| `gk fetch [--kernel K] [--gcc G]` | download and check trees and toolchain tarballs into `~/.cache/gk` (or `GK_CACHE`) |
| `gk forge G [--target T]` | build one toolchain bundle in its forge container (04.4) and print its digest |
| `gk forge verify` | run every bundle's self check: version banners, `-dumpversion`, a trivial kernel-style object for each target |
| `gk hosts check` | run every host container and check its tool versions against `hosts.toml` |
| `gk probe K G --platform P` | the accept probe of 09.4 |
| `gk cell K G --platform P [--config C] [--twins]` | run one cell up the ladder and print its verdict, writing the cell directory |
| `gk boot DIR [--runs 3]` | boot an existing build directory |
| `gk search K --platform P` | the frontier search of 09.3 for one row |
| `gk column G --platform P` | the same search for one column, over kernels |
| `gk sweep --set S [--tier N]` | search every row of a set, scheduling jobs across machines |
| `gk classify DIR` | run the signatures over a failed cell and print its classes |
| `gk triage` | list failed cells whose first error matched no signature |
| `gk explain K G --platform P` | the edge in words: which unit failed, the signature, the kernel commit that fixed it and the first release that has it |
| `gk bisect-kernel K1 K2 G --platform P` | bisect the kernel history between two versions for the commit that made G work or stop working, using the full history clone (03.5) |
| `gk config-diff K G1 G2 --platform P` | the `.config` differences between two GCC columns on one kernel (11.3) |
| `gk flags-diff K G1 G2 --platform P` | the per-unit command line differences between two columns |
| `gk gates K` | the compiler version gates in a tree: every `GCC_VERSION`, `__GNUC__`, `cc-ifversion`, `gcc-min-version`, `cc-option` and Kconfig `GCC_VERSION` test (11.4) |
| `gk warnings K G --platform P` | the warning census of one cell (11.5) |
| `gk repro CELL` | print the exact commands to reproduce a cell by hand with plain `docker` and `make`, without `gk` |
| `gk publish` | write `matrix/` and `reports/` from the result store |
| `gk report new-gcc G` | write `reports/new-gcc-<version>.md` from the result store (10.8) |

Every command that writes a result records the `gk` commit and refuses to record a graded result from a dirty tree unless `--ungraded` is given.

## 10.4 The shim and the build directory

`gk-cc` is `rk-cc` under another name. It is copied from rucc-kernel at G0, not shared through a crate, for the reason the kernel plan gives in its section 14.6: two copies are cheaper than an early abstraction. Placed in front of `CC`, it runs the real compiler with the arguments unchanged and appends a line per call to `compile.jsonl`: cwd, argv, inputs, outputs, exit status, wall and CPU time, peak RSS, the first 64 KiB of stderr, which is where the warning census of 11.5 comes from, and whether the call was a probe.

The schema of `compile.jsonl`, `build.json` and `summary.md` is the rucc-kernel schema, with two added fields in `build.json`: `cell` (the identity of 02.1) and `rung` (the rung reached). That is the data contract between the two repositories. `rk config-diff`, `rk flags-diff`, `rk sections-diff`, `rk symvers-diff`, `rk vec-audit` and `rk frames` can read a gcc-kernel build directory as the reference side of a comparison without change, which is how rucc-kernel uses this repository (document 11).

A cell directory holds:

```
cells/<cell-id>/
  cell.json          coordinates, digests, verdict, rung, twins, classes, timings, machine
  build.json         the rucc-kernel schema, plus cell and rung
  compile.jsonl      kept for edge cells and holes, dropped otherwise after classification
  errors.jsonl       every failing unit with its first error and its signature classes
  warnings.jsonl     the warning census, for columns the census covers (11.5)
  .config            the configuration kbuild built, under the name rk reads
  make.log.zst       kept for edge cells and holes
  boot-<n>.log       one console per boot run
  kunit.json         parsed TAP, where L7 ran
```

## 10.5 File formats

`kernels.toml` follows rucc-kernel's `pins.toml`, with era and sets:

```toml
[[kernel]]
version = "2.6.32.71"
url = "https://cdn.kernel.org/pub/linux/kernel/v2.6/longterm/v2.6.32/linux-2.6.32.71.tar.xz"
sha256 = "..."
era = "M6"
sets = ["last-points", "longterm-stripe"]
arches = ["x86", "arm", "powerpc", "s390", "mips"]
```

`gccs.toml` has one entry per column:

```toml
[[gcc]]
id = "gcc-4.9.4"
version = "4.9.4"
flavor = "upstream"
released = "2016-08-03"
url = "https://ftp.gnu.org/gnu/gcc/gcc-4.9.4/gcc-4.9.4.tar.bz2"
sha256 = "..."
forge = "gk-forge-jessie"
targets = ["x86_64-linux-gnu", "i686-linux-gnu", "aarch64-linux-gnu", "arm-linux-gnueabi", "powerpc64le-linux-gnu", "s390x-linux-gnu", "mips-linux-gnu"]
binutils = "auto"
configure = "modern"
columns = ["last-point"]

[[gcc]]
id = "debian-stretch-gcc-6"
version = "6.3.0"
flavor = "debian"
package = { dist = "stretch", name = "gcc-6", version = "6.3.0-18+deb9u1" }
host = "gk-host-stretch"
notes = "default PIE: the flavor that broke 4.4 and earlier without -fno-PIE"
```

`platforms.toml` extends rucc-kernel's `rows.toml` with eras, because the same architecture boots on different machines at different ages:

```toml
[[platform]]
name = "x86_64"
tier = 1
arch = { from = "2.6.24", value = "x86", before = "x86_64" }
first-kernel = "2.6.0"
image = "bzImage"
qemu = "qemu-system-x86_64"
machine = [ { from = "2.6.0", value = "pc" }, { from = "3.0", value = "q35" } ]
cpu = [ { from = "2.6.0", value = "qemu64" }, { from = "4.0", value = "max" } ]
console = "ttyS0"
budget = { build-minutes = 60, boot-seconds = 120 }
```

`cell.json` is the result record, and `matrix.json` is the list of them with the heavy fields removed:

```json
{
  "cell": "sha256:...",
  "kernel": "5.4.0", "gcc": "gcc-10.5.0", "binutils": "binutils-2.36.1",
  "platform": "x86_64", "config": "defconfig+gk", "host": "gk-host-buster",
  "rung": "L2", "verdict": "fails", "twins": { "L3w": "fails" },
  "classes": ["fno-common"], "failing-units": 3,
  "first-error": "scripts/dtc/dtc-parser.tab.o: multiple definition of `yylloc'",
  "fixed-by": { "commit": "e33a814e772c", "first-release": "5.6" },
  "runs": 1, "gk": "abcdef0", "date": "2026-10-14"
}
```

## 10.6 The result store

Results are data, not source, so they do not go in the repository's history. The store is a directory tree on gpc, keyed by cell identity, mirrored nightly to a release asset per sweep (`sweep-2026-10-14.tar.zst`). `gk publish` reads it and writes `matrix/` and `reports/`, which are committed. A result can always be traced from `matrix.json` to its cell directory by identity.

## 10.7 The published matrix

`matrix.json` and the three derived files are the product. They are versioned by the sweep date, released as assets, and have a stable schema with a `schema` version field. Two consumers are planned: rucc-kernel (document 11) and the human-readable reports. Anything else can read the JSON.

The markdown reports are heat maps: one table per platform, a row per kernel, a column per GCC, one character per cell (`W` works, `R` runs, `B` builds, `F` fails, `w` when the L3w twin passed, `·` n/a, blank for not run). The per-GCC and per-kernel pages list edges with their explanation from `gk explain`. A static HTML rendering of the same data is published to the repository's GitHub Pages, generated by `gk publish --html`, with no JavaScript framework.

## 10.8 The new GCC report

When `watch.yml` sees a GCC release, it opens a pull request adding the column. When the pull request lands, `sweep.yml` runs the current stripe dense and then the column search. The report `reports/new-gcc-<version>.md` says:
1. which kernels in the Current set build and run, per platform;
2. which kernels the previous release of the same series ran and this one does not, with the classified first errors;
3. new warnings on current kernels, counted by warning option (11.5);
4. `.config` differences against the previous column on the Current set.

`gk report new-gcc G` writes it from whatever cells the store holds. A crossing with no cell is shown as not run, so the report can be written while the sweep is going and again when it is done. The file is named by the major and minor version when the patch level is zero, as in `new-gcc-16.2.md`.

The first of these reports will be for whatever GCC is released after G2. It is also the cheapest early warning rucc gets about what "compatible with the latest GCC" is about to mean.

## 10.9 CI

| Workflow | Runs on | Does |
|---|---|---|
| `pr.yml` | GitHub runners | `cargo test`, clippy, rustfmt, the style check, schema checks of every TOML file, and the accept probe for any pin the pull request changes |
| `forge.yml` | gpc, self-hosted | builds bundles and containers, pushes them to `ghcr.io/tamnd/gcc-kernel`, writes digests into a pull request |
| `watch.yml` | GitHub runners, daily | `gk pins`, and a pull request when anything changed |
| `sweep.yml` | gpc and servers, self-hosted | the sweep or the incremental runs, at low priority |
| `publish.yml` | GitHub runners | `gk publish` from the latest store mirror, commits `matrix/` and `reports/` |

## 10.10 House style

The same as rucc-kernel: plain English, one paragraph per line, no em or en dashes, no horizontal rules, checked by `scripts/style.sh` on every pull request.
