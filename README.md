# gcc-kernel

Which GCC builds which Linux kernel? This repository answers that by trying it. It pins every kernel release from 0.01 to today, every GCC release it can build from source, and every platform QEMU can boot, then builds each pairing it picks, boots the result, runs the kernel's own tests, and publishes a matrix. Every edge of that matrix comes with an explanation: the first GCC that works for a kernel, the last one that does, why the ones outside fail, and the kernel commit that fixed it.

It exists because of [rucc](https://github.com/tamnd/rucc), a C compiler in Rust that aims to be compatible with GCC 16, and [rucc-kernel](https://github.com/tamnd/rucc-kernel), the harness that builds kernels with it. Both depend on facts about GCC that nobody has written down in one place, such as which kernels GCC 16 itself builds and boots, and whether the GCC a kernel era is graded against really works for every version in that era. gcc-kernel never runs rucc. It only measures real GCCs, so it stays an independent reference, and its results flow one way into rucc-kernel.

Recent kernels come first, because that is where rucc is going. Old kernels are there as reference, to show how the kernel and GCC moved together.

<!-- gk:status:begin -->
## Status

This section is written by `gk publish` from [matrix/matrix.json](matrix/matrix.json) and is replaced every time the results are published. Last published 2026-10-05, with 95 cells in the matrix.

### Progress

The current stripe of G1 is every Current kernel with every upstream GCC column from 8.1 on the tier 1 platforms, with `defconfig+gk` and `tinyconfig+gk`. The GCC 16 column is every release from 5.0 on with the newest GCC 16. A Current kernel that has just moved to a new point release counts the cells of the point before until the new one has run.

```text
x86_64 defconfig+gk      ████████░░░░░░░░░░░░░░░░░░░░░░   24/91    26%
x86_64 tinyconfig+gk     ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    0/91     0%
i386 defconfig+gk        ████░░░░░░░░░░░░░░░░░░░░░░░░░░   12/91    13%
i386 tinyconfig+gk       ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    0/91     0%
arm64 defconfig+gk       ▊░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    2/91     2%
arm64 tinyconfig+gk      ▍░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    1/91     1%
total                    ██▎░░░░░░░░░░░░░░░░░░░░░░░░░░░   39/546    7%
```

```text
x86_64 gcc-16.2.0        ▎░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    1/131    0%
i386 gcc-16.2.0          ▎░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    1/131    0%
arm64 gcc-16.2.0         ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    0/84     0%
total                    ▎░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    2/346    0%
```

### Verdicts

The newest cell at each crossing, over every platform and configuration.

```text
🟩 works      ████████████████████████████████████████ 24
🟨 runs       █████████████████████▋                   13
🟧 builds     ███▍                                     2
🟥 fails                                               0
```

The same cells by the rung they reached (spec 02.2), L8 being clean all the way through KUnit and the splat check.

```text
L0                                                    0
L1                                                    0
L2                                                    0
L3                                                    0
L4                                                    0
L5           ███▍                                     2
L6           █████████████▍                           8
L7           ████████▍                                5
L8           ████████████████████████████████████████ 24
```

The median build time of `defconfig+gk` cells per GCC column, in minutes. The machines are shared, so these move with their load.

```text
8.5.0        ████████████████████████████████████▍    32 min, 4 cells
9.5.0        █████████████████████▊                   19 min, 3 cells
10.5.0       ████████████████████████▋                22 min, 3 cells
11.5.0       █████████████████████████▎               22 min, 3 cells
12.2.0       ██████████████████████████▎              23 min, 4 cells
12.5.0       ██████████████████████████████▍          27 min, 3 cells
13.5.0       ████████████████████████████▍            25 min, 3 cells
14.2.0       ████████████████████████████████████████ 36 min, 4 cells
14.4.0       ███████████████████████████████▎         28 min, 3 cells
15.3.0       █████████████████▏                       15 min, 3 cells
16.1.0       ██████████████████                       16 min, 3 cells
16.2.0       █████████████████▉                       16 min, 2 cells
```

### By GCC column

The newest cell at each crossing again, over every kernel, platform and configuration.

| GCC | Cells | 🟩 | 🟨 | 🟧 | 🟥 | ⚠️ | Works |
|---|--:|--:|--:|--:|--:|--:|---|
| 8.5.0 | 4 | 2 | 2 | 0 | 0 | 1 | `█████░░░░░` 50% |
| 9.5.0 | 3 | 2 | 1 | 0 | 0 | 0 | `██████▊░░░` 66% |
| 10.5.0 | 3 | 3 | 0 | 0 | 0 | 0 | `██████████` 100% |
| 11.5.0 | 3 | 2 | 1 | 0 | 0 | 0 | `██████▊░░░` 66% |
| 12.2.0 | 4 | 2 | 1 | 1 | 0 | 1 | `█████░░░░░` 50% |
| 12.5.0 | 3 | 2 | 1 | 0 | 0 | 0 | `██████▊░░░` 66% |
| 13.5.0 | 3 | 2 | 1 | 0 | 0 | 0 | `██████▊░░░` 66% |
| 14.2.0 | 4 | 2 | 2 | 0 | 0 | 1 | `█████░░░░░` 50% |
| 14.4.0 | 3 | 2 | 1 | 0 | 0 | 0 | `██████▊░░░` 66% |
| 15.3.0 | 3 | 2 | 0 | 1 | 0 | 1 | `██████▊░░░` 66% |
| 16.1.0 | 3 | 2 | 1 | 0 | 0 | 0 | `██████▊░░░` 66% |
| 16.2.0 | 3 | 1 | 2 | 0 | 0 | 1 | `███▍░░░░░░` 33% |

### Latest cells

| Date | Kernel | GCC | Platform | Config | Verdict | Rung | Minutes |
|---|---|---|---|---|---|---|--:|
| 2026-10-03 | 7.2.8 | 16.1.0 | i386 | defconfig+gk | 🟩 works | L8 | 50 |
| 2026-10-03 | 7.2.8 | 14.4.0 | x86_64 | defconfig+gk | 🟩 works | L8 | 76 |
| 2026-10-03 | 7.2.8 | 15.3.0 | i386 | defconfig+gk | 🟩 works | L8 | 48 |
| 2026-10-03 | 7.2.8 | 14.4.0 | i386 | defconfig+gk | 🟩 works | L8 | 50 |
| 2026-10-03 | 7.2.8 | 13.5.0 | x86_64 | defconfig+gk | 🟩 works | L8 | 70 |
| 2026-10-03 | 7.2.8 | 13.5.0 | i386 | defconfig+gk | 🟩 works | L8 | 45 |
| 2026-10-03 | 7.2.8 | 12.5.0 | x86_64 | defconfig+gk | 🟩 works | L8 | 62 |
| 2026-10-03 | 7.2.8 | 12.5.0 | i386 | defconfig+gk | 🟩 works | L8 | 46 |
| 2026-10-03 | 7.2.8 | 12.2.0 | x86_64 | defconfig+gk | 🟧⚠️ builds | L5 | 50 |
| 2026-10-03 | 7.2.8 | 12.2.0 | i386 | defconfig+gk | 🟩 works | L8 | 43 |

### Matrices

One square per cell: 🟩 works, 🟢 works on the smaller museum suite of a kernel before 2.6, 🟨 runs, 🟧 builds, 🟥 fails, · n/a, and blank where the cell has not run yet. ⚠️ marks a flaky cell, whose boots disagreed and which keeps the lowest rung. A table appears once its first cell has run, and then every Current kernel has a row in it. The full heat maps, with warning counts, are in [reports](reports), next to the [warning census](reports/warning-census.md) and the [configuration differential](reports/config-differential.md) of the Current set.

#### x86_64 defconfig+gk, 24 cells

| Kernel | 3.2.3 | 3.3.6 | 3.4.6 | 4.0.4 | 4.1.2 | 4.2.4 | 4.3.5 | 4.3.6 | 4.4.7 | 4.5.4 | 4.6.4 | 4.7.2 | 4.7.4 | 4.8.5 | 4.9.2 | 4.9.4 | 5.5.0 | 6.3.0 | 6.5.0 | 7.5.0 | 8.3.0 | 8.5.0 | 9.5.0 | 10.5.0 | 11.5.0 | 12.2.0 | 12.5.0 | 13.5.0 | 14.2.0 | 14.4.0 | 15.3.0 | 16.1.0 | 16.2.0 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 5.10.271 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 5.15.222 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.1.189 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.6.158 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.12.111 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 🟩 |  |  |  |  |  |  |  |
| 6.12.112 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.18.54 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 🟨 | 🟨 | 🟩 | 🟨 | 🟨 | 🟨 | 🟨 | 🟨⚠️ | 🟨 | 🟧⚠️ | 🟨 |  |
| 6.18.55 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 7.2.8 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 🟩 | 🟩 | 🟩 | 🟩 | 🟧⚠️ | 🟩 | 🟩 | 🟨 | 🟩 | 🟩 | 🟩 | 🟩 |
| 7.2.9 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

#### i386 defconfig+gk, 12 cells

| Kernel | 2.5.8 | 2.6.3 | 2.7.2.3 | 2.8.1 | 2.91.66 | 2.95.3 | 3.0.4 | 3.1.1 | 3.2.3 | 3.3.6 | 3.4.6 | 4.0.4 | 4.1.2 | 4.2.4 | 4.3.5 | 4.3.6 | 4.4.7 | 4.5.4 | 4.6.4 | 4.7.2 | 4.7.4 | 4.8.5 | 4.9.2 | 4.9.4 | 5.5.0 | 6.3.0 | 6.5.0 | 7.5.0 | 8.3.0 | 8.5.0 | 9.5.0 | 10.5.0 | 11.5.0 | 12.2.0 | 12.5.0 | 13.5.0 | 14.2.0 | 14.4.0 | 15.3.0 | 16.1.0 | 16.2.0 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 5.10.271 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 5.15.222 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.1.189 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.6.158 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.12.112 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.18.55 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 7.2.8 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 🟩 | 🟩 | 🟩 | 🟩 | 🟩 | 🟩 | 🟩 | 🟩 | 🟩 | 🟩 | 🟩 | 🟨⚠️ |
| 7.2.9 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

#### arm64 defconfig+gk, 2 cells

| Kernel | 4.8.5 | 4.9.2 | 4.9.4 | 5.5.0 | 6.3.0 | 6.5.0 | 7.5.0 | 8.3.0 | 8.5.0 | 9.5.0 | 10.5.0 | 11.5.0 | 12.2.0 | 12.5.0 | 13.5.0 | 14.2.0 | 14.4.0 | 15.3.0 | 16.1.0 | 16.2.0 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 5.10.271 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 5.15.222 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.1.189 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.6.158 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.12.112 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.18.55 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 7.2.8 |  |  |  |  |  |  |  |  | 🟨⚠️ |  |  |  |  |  |  | 🟩 |  |  |  |  |
| 7.2.9 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

#### arm64 tinyconfig+gk, 1 cell

| Kernel | 4.8.5 | 4.9.2 | 4.9.4 | 5.5.0 | 6.3.0 | 6.5.0 | 7.5.0 | 8.3.0 | 8.5.0 | 9.5.0 | 10.5.0 | 11.5.0 | 12.2.0 | 12.5.0 | 13.5.0 | 14.2.0 | 14.4.0 | 15.3.0 | 16.1.0 | 16.2.0 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 5.10.271 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 5.15.222 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.1.189 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.6.158 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.12.112 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 6.18.55 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| 7.2.8 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 🟨 |
| 7.2.9 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
<!-- gk:status:end -->

## How it works

A cell is one kernel, one GCC, one binutils, one platform, one configuration and one host environment. Each cell climbs a ladder, from L0 (the tree is fetched and checks out) through L4 (it links) and L6 (it boots and passes the smoke suite) to L8 (KUnit passes and the console is clean). The highest rung it reaches gives its verdict: works, runs, builds, fails, or n/a.

The kernel picks the host. `HOSTCC` and the host tools come from a Debian container of the kernel's own era, and the GCC under test comes in as a static toolchain bundle through `CROSS_COMPILE`, so a failure along a row is the toolchain's and not the host's.

The matrix is not swept densely. Each row is searched outward from the GCC the kernel was built with when it was current, until the edges are found, with samples inside to catch holes. A few stripes are run in full: the kernels kernel.org maintains today, the last point of every longterm line, the GCC 16 column, and the era diagonal.

## Before 1.0

The 0.x kernels are outside the matrix, and [reports/museum-0x.md](reports/museum-0x.md) has what happened to them. In short, Linus's own GCC 1.40 of September 1991, the one with `-mstring-insns`, survives on oldlinux, and with it pristine 0.11 and 0.12 build and boot under Bochs:

```
0.01     ..........  needs Minix to build
0.11     ##########  built, booted, smoke rc passed
0.12     ##########  built, booted, smoke rc passed
0.95     #.........  L0, no config target
0.96c    #.........  L0, no config target
0.99.11  ###.......  L2, compiles as C++
0.99.12  ###.......  L2, compiles as C++
0.99.13  ###.......  L2, panic declared twice
0.99.15  ###.......  L2, GNU ld 1.x syntax
```

## The rules

**Nothing is patched.** Kernels are unmodified tarballs checked by hash. GCC and binutils are built from release tarballs. Anything that cannot be built without a patch is recorded as unbuildable, with its log.

**Every failure gets a name.** The first error of every failing cell is matched against a catalog of signatures, each with the kernel commit that fixed it, and that commit is confirmed by bisection.

**No hidden configuration.** Everything that changes a cell is in a committed file: kernels, toolchains, hosts, platforms, configs, signatures and the search seed.

**Same schema as rucc-kernel.** The compiler shim `gk-cc` is rucc-kernel's `rk-cc`, and a gcc-kernel cell directory is a rucc-kernel build directory, so rucc-kernel's diff tools read it unchanged.

## Platforms

Tier 1 is x86_64, i386 and arm64. Tier 2 is arm, riscv64, ppc64le, s390x and loongarch64. Tier 3 gets build only cells on current kernels.

## Milestones

The full plan is in [docs/spec](docs/spec). It is split into seven milestones, G0 to G6. Each one has a GitHub milestone and a tracking issue of the same name.

| | Scope |
|---|---|
| G0 | the instrument: model, shim, modern forge, one dense row |
| G1 | the current stripe on tier 1, boots and KUnit, the GCC 16 column |
| G2 | GCC 4.6 to 7.5, kernels 3.0 to 4.20, the failure catalog, the eras report |
| G3 | the legacy forge, 2.6.x, 2.4 and the museum, the 0.x experiment |
| G4 | tier 2 platforms |
| G5 | distribution GCCs and the binutils sweep |
| G6 | steady state: new kernels and GCCs show up without anyone touching the repo |

## Building

`cargo build --release` builds `gk`. The Rust toolchain is pinned in `rust-toolchain.toml`. Right now `gk` knows `check`, `pins`, `fetch`, `forge`, `hosts check`, `pins changed`, `probe`, `cell`, `store`, `ladder` and `version`. `gk check` reads the pin files at the top of the repository and checks that they agree with each other. `gk pins` applies the rules in `sets.toml` to kernel.org and the GNU mirror and shows how the pins would change, and `gk pins --write` writes them. `gk fetch` downloads pinned tarballs into `~/.cache/gk` and checks them against their pins and the signing keys in `keys/`. `gk forge` builds a static GCC and binutils bundle in a forge container, and `gk forge verify` checks every bundle in every host. `gk hosts check` runs each host and forge container and compares its GCC, binutils and make with `hosts.toml`. `gk probe 7.2.8 gcc-16.2.0 --platform x86_64` asks whether that kernel accepts that GCC at all, by running `defconfig` and preprocessing `init/main.c`. `gk cell` with the same arguments builds the cell up to L4 in its host container, with `gk-cc` logging every compiler call, and writes the cell directory into the result store, under `cells/<identity>` in `$GK_STORE` or `~/.cache/gk/store`. `gk store` lists the cells in it, `gk store show` prints one, `gk store check` finds cells that did not finish, and `gk store pack` writes the tarball a sweep publishes. It needs a static `gk-cc` next to `gk`, which `cargo build --release --target x86_64-unknown-linux-musl -p gk-cc` gives. `gk pins changed BASE` lists the accept probes that the pins changed since a git revision call for, and `pr.yml` runs them on every pull request that touches a pin file. The rest of the commands in [docs/spec/10-gcc-kernel-repo.md](docs/spec/10-gcc-kernel-repo.md) arrive with the milestones that need them. Running cells will need Linux, Docker and QEMU.

## House style

Prose in this repository is plain English with one paragraph per line, no em or en dashes and no horizontal rules. `scripts/style.sh` checks it on every pull request.

## License

Apache-2.0. The kernel trees and toolchains it fetches are under their own licenses and are never stored here.
