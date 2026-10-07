# The kernel axis

The rows of the matrix are kernel trees. This document fixes which trees are rows, where they come from, how each is tied to an era with an era GCC and a host, and the full history clone used to bisect edges.

## 3.1 Sets

A tree is a row because it belongs to one or more sets. `sets.toml` holds the rules, in the same form as rucc-kernel's `sets.toml` (`rk sets` and `gk pins` apply them to kernel.org's `releases.json` and the archive listings), and `kernels.toml` holds the resulting pins.

| Set | Rule | Size, October 2026 | Search (09) |
|---|---|---|---|
| `current` | every line kernel.org maintains, at its latest point: stable and each longterm | about 7 | dense on tier 1 and 2 |
| `cip` | the latest release of each CIP super long term line (`-cip` tags from the CIP kernel tree) | about 4 **(which lines are still maintained is checked at G0)** | dense on tier 1 |
| `releases` | every mainline release from 2.6.0: 2.6.0 to 2.6.39, 3.0 to 3.19, 4.0 to 4.20, 5.0 to 5.19, 6.0 to 6.19, 7.0 to the newest | about 125 | frontier search |
| `last-points` | the last point release of every release line that had stable updates, from 2.6.11 on (when the `-stable` process began), and the last `2.6.x` of the four digit numbering | about 110 | frontier search |
| `longterm-stripe` | the last point of every line that was ever longterm | about 25 | dense on tier 1 |
| `museum` | 0.01, 0.11, 0.12, 0.95, 0.96c, 0.97, 0.98, 0.99.15, 1.0, 1.1.95, 1.2.13, 1.3.100, 2.0.40, 2.1.132, 2.2.26, 2.3.99-pre9, 2.4.37.11, 2.5.75 | 18 | the era diagonal first, then frontiers where toolchains exist |
| `rc` | a release candidate, on demand, for a finding that needs it | 0 by default | as asked |

The union is about 250 trees, which is the row count used in document 09. A tree in several sets is one row with several set tags.

**Why 2.6.0 is the floor of `releases`.** Before 2.6, odd minor versions were development series with a hundred releases each, and even ones had point releases that mostly fixed bugs. The museum takes the last of each series instead, which is where each one was most finished. Anyone who needs a particular 1.3.x or 2.1.x can run `gk cell` on it; it is not swept.

**Why last points.** A release like 4.9 was the base of a longterm line that ended at 4.9.337, six years later, with hundreds of backported fixes, including the GCC fixes of 01.4. The last point is the tree people kept building, often with much newer GCCs than the release had. The difference between the release's row and its last point's row is the measured effect of stable backports on compiler compatibility, which is one of the questions document 02 asks.

## 3.2 Sources and hashes

| Trees | Source | Check |
|---|---|---|
| 2.6.0 on | `cdn.kernel.org/pub/linux/kernel/v<N>.x/` tarballs, `.tar.xz` where it exists | the `sha256sums.asc` of the directory, signed. `gk fetch` checks the signature once and pins the SHA-256 in `kernels.toml` |
| 1.0 to 2.5 | the same, under `v1.0` to `v2.5`, `.tar.gz` or `.tar.bz2` | `.tar.sign` where present, then the pinned SHA-256 |
| 0.x | kernel.org's `Historic/` directory, which has 0.01 and later | `sha256sums.asc` there, and the pinned hash. 0.01 is `24454f830cdb571e2c4ad15481119c43b3cafd48dd869a9b2945d1036d1dc68d` |
| CIP | the CIP kernel git tree, a tag, archived with `git archive` | the tag's commit id pinned in `kernels.toml` alongside the archive's SHA-256 |
| rc | kernel.org snapshot tarballs from git | commit id pinned |

Tarballs are fetched once into the store (10.6) and never refetched, so a change of a tarball on a mirror is noticed as a hash failure, not absorbed. A tree is unpacked fresh into scratch for every cell.

## 3.3 Eras and the era GCC

Every tree belongs to an era in `eras.toml`. An era is a range of kernel versions with:
- **an era GCC** e(K): the GCC the kernel was built with when it was current;
- **an era binutils**;
- **a host environment** H (05.2);
- **a configuration fragment** `configs/fragment.<era>` (07.2);
- **the kernel plan era** it maps to (E0 to E11 of [the kernel plan's document 04](https://github.com/tamnd/rucc-kernel/blob/main/docs/plan/04-versions-and-personas.md)).

The era GCC is the kernel plan's persona version where one exists, and the era host's distribution GCC otherwise. That keeps the two repositories agreeing on what "the GCC of this kernel" means, and it makes e(K) the thing the matrix checks for rucc-kernel (11.2): if e(K) is not in W(K, P), the kernel plan's persona is wrong for that kernel.

| Era | Kernels | Kernel plan era | e(K) | Host |
|---|---|---|---|---|
| X0 | 0.01 to 0.12 | none | 1.40 | `gk-host-minix`, `gk-host-linux012` |
| X1 | 0.95 to 0.99.15 | none | 2.3.3 to 2.5.8 **(chosen at G3)** | `gk-host-bo` |
| M0 | 1.0 to 1.2.13 | E0 | 2.5.8 | `gk-host-bo` |
| M1 | 1.3.x, 2.0.x | E0, E1 | 2.7.2.3 | `gk-host-hamm` |
| M2 | 2.1.x, 2.2.x | E1 | 2.7.2.3 | `gk-host-slink` |
| M3 | 2.3.x, 2.4.x | E2 | 2.95.3 | `gk-host-woody` |
| M4 | 2.5.x, 2.6.0 to 2.6.15 | E3 | 3.3.6 (decided at G3, see below) | `gk-host-sarge` |
| M5 | 2.6.16 to 2.6.25 | E4 | 4.1.2 | `gk-host-etch` |
| M6 | 2.6.26 to 2.6.39 | E4 | 4.3.5 | `gk-host-lenny`, `gk-host-squeeze` |
| M7 | 3.0 to 3.17 | E5 | 4.7.2 | `gk-host-squeeze`, `gk-host-wheezy`, `gk-host-jessie` |
| M8 | 3.18 to 4.1 | E6 | 4.9.2 | `gk-host-jessie` |
| M9 | 4.2 to 4.17 | E7 | 6.3.0 | `gk-host-jessie`, `gk-host-stretch` |
| M10 | 4.18 to 5.4 | E8 | 8.3.0 | `gk-host-stretch`, `gk-host-buster` |
| M11 | 5.5 to 5.11 | E8 | 10.2.1 | `gk-host-buster`, `gk-host-bullseye` |
| M12 | 5.12 to 5.17 | E9 | 10.2.1 | `gk-host-bullseye` |
| M13 | 5.18 to 6.14 | E10 | 12.2.0 | `gk-host-bullseye`, `gk-host-bookworm`, `gk-host-trixie` |
| M14 | 6.15 to current | E11 | 14.2.0, and 16.2 as a second reference | `gk-host-trixie` |

Two numberings exist on purpose. The M eras are finer than the E eras because they follow host boundaries (05.2), and a host boundary can fall inside a GCC era. Where an M era spans two hosts, the host split follows the table of 05.2, and `eras.toml` records it per kernel range. The era GCC is not the host's GCC in every case: in M4 the host is sarge with GCC 3.3.5, and the era GCC is 3.3.6. That does not matter, because the era GCC is a column run through `CROSS_COMPILE` like any other (05.3).

**The M4 era GCC** was left open until G3, between the kernel plan's 3.4.6 and 2.95.3 for the releases before 2.6.12. The forge settled it. On i386, 3.3.6 builds every release from 2.6.0 to 2.6.15, while 3.4.6 fails 2.6.0 to 2.6.3 on conflicting types for `smp_send_reschedule`, and 2.95.3 and 3.0.4 stop on `.incbin`, which their binutils do not have. On x86_64 both 3.3.6 and 3.4.6 fail 2.6.0 to 2.6.10 on a macro redefinition in `entry.S` with the binutils they pair with, and 3.2.3 builds most of them, so the x86_64 rows of that span get 3.2.3 as their best GCC through the frontier search rather than through the era. 2.6.6 on x86_64 fails with every GCC, on `PCI_PROBE_MMCONF` in `mmconfig.c`, which is a kernel bug. M4 uses 3.3.6.

**The era diagonal** is the set of cells (K, e(K), P) for every K. It is run first in every sweep (09.6), because it tests three things at once: that the host works for the kernel, that the configuration fragment is right, and that the kernel plan's persona builds and boots the kernel. An era diagonal cell that fails is investigated before its row is searched. The failure is either a host or fragment bug in gcc-kernel, or a real finding that the persona is wrong.

**Moving a boundary.** Like the kernel plan's rule, a boundary moves only with evidence: an era diagonal cell that fails where the neighbouring era's GCC or host works. `eras.toml` records the reason and the cell.

## 3.4 Configurations per kernel

Covered in 07.2. The configuration is a function of the kernel's era and the platform, never of the column, so that a row shares one configuration recipe.

## 3.5 The full history clone

Bisecting an edge needs every commit, not only releases. `gk bisect-kernel K1 K2 G --platform P` uses a clone that joins:
- Linus's tree from v2.6.12-rc2 (the start of git);
- the `history.git` import of BitKeeper history, 2.4.0 to 2.6.12-rc2, whose root commit is bb441db;
- mpe's `linux-fullhistory`, which grafts the two and adds the pre-2.4 releases as commits.

The joined clone lives on gpc (about 6 GB), updated weekly. The bisection runs cells of the form (commit, G, P) with the row's configuration and host, to the rung where the edge is, and reports the first commit on which the cell's verdict changes. An edge at L1 is a refusal, so its bisection runs only the accept probe on each commit, which takes seconds instead of a build. An edge at L3 whose first failing unit is known can be bisected with `--unit` and that object, and then each commit is configured as its cell would be and builds `prepare` and the object alone, which takes minutes instead of an hour. Commits before git are release-sized, so bisections into the museum stop at release granularity, which the report says. A commit whose cell failed for the machine's reasons, over its budget, out of disk, or a build that stopped with no make error in its log, is skipped rather than counted as a miss, since one false miss sends the bisection down the wrong half and names the wrong commit.

Bisections have two uses:
1. **Confirming `fixed-by`.** Every signature's fix commit (08.2) is confirmed by bisecting between a failing and a working kernel with the signature's GCC. A mismatch with the claimed commit is recorded and the bisected commit wins.
2. **Explaining unknown edges.** An edge without a signature is bisected. The commit found, with its message, is usually enough to write the signature.

Stable branches are bisected on the branch itself (`linux-X.Y.y` from the stable tree), since a backport has a different commit id from the mainline fix. The signature records both.

## 3.6 New kernels

`watch.yml` (10.9) polls `releases.json` daily. A new mainline release, point release or longterm line changes `kernels.toml` by a pin pull request. Once merged, the incremental sweep (09.7) runs the new row: era diagonal, GCC 16 column, the current stripe's columns, and a frontier search. A new point release of a longterm line replaces its predecessor in `current` and in `last-points` and keeps the predecessor's results, so the history of a line's compatibility across its point releases is kept too.
