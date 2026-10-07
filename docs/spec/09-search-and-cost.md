# Search strategy and cost

The full product of the axes is too large to run every week, and most of it is not interesting. A kernel from 2009 will not build with GCC 2.95 or with GCC 16, and running both to confirm it costs as much as a cell that teaches something. This document fixes how `gk` chooses which cells to run, how it checks the choice did not hide anything, and what the whole thing costs.

## 9.1 The size of the full product

Counts as of 1 October 2026, from documents 03, 04 and 06:

| Axis | Members | Note |
|---|---|---|
| kernels | about 250 | museum about 25, releases 2.6.0 to 7.2 about 126, last points about 90, current 8 plus 5 CIP heads, overlapping |
| upstream GCC columns | 36 | the last point of every release series, 1.42 to 16.2 (04.2) |
| extra GCC point columns | 20 | first points of each major from 5 on, and every point release the kernel ever blacklisted |
| distribution GCC columns | about 20 | one per Debian and Ubuntu release that changed a default the kernel notices (04.3) |
| platforms, tier 1 | 3 | x86_64, i386, arm64 |
| platforms, tier 2 | 5 | arm, riscv64, ppc64le, s390x, loongarch64 |
| platforms, tier 3 | the rest | build only |
| configurations | 2 graded | `defconfig+gk` everywhere, `tinyconfig+gk` where it exists (3.17 on) |

Tier 1 alone, upstream last points only, `defconfig` only, is 250 × 36 × 3 = 27,000 cells. About 40% of them are **n/a**: arm64 only exists from 3.7 and needs GCC 4.8 or later, x86_64 needs GCC 3.1 or later, and the museum is i386 only. That leaves about 16,000 real cells for tier 1. A cell that reaches L6 costs a build and three boots.

## 9.2 What a cell costs

The estimates were for one build job with 8 CPUs on gpc, and KVM boots. G0 measured the modern row (open question 3) and the older rows are still estimates until G1 runs a row in each of those eras:

| Kernel era | `defconfig` build | Boot to L6, one run | L7 KUnit run |
|---|---|---|---|
| 0.x to 2.2 | under 1 minute | 15 s (TCG, `isapc`) | none |
| 2.4, 2.6.0 to 2.6.39 | 1 to 3 minutes | 20 s | none |
| 3.x, 4.x | 3 to 5 minutes | 20 s | none before 5.5 |
| 5.x, 6.x, 7.x | 12 to 13 minutes on 6 cores, measured | 13 to 15 s on x86_64 under TCG, measured | about 5 minutes per run on x86_64 under TCG, measured |

A cell that fails fails fast: `make` without `-k` stops at the first error, and a GCC that is too new for an old kernel usually trips within the first hundred units. A cell that fails at L1 costs seconds, because the accept probe (9.4) runs before any build.

The measurement is the dense row of 7.2.8 on x86_64 with `defconfig+gk`, every GCC column from 8.5.0 to 16.2.0, all ten at L4, run on 1 October 2026 on server2, a 6 core AMD EPYC VPS with `make -j6`, and published in `matrix/matrix.json`. The build step alone took 722 to 765 seconds for seven of the columns. GCC 8.5.0, 9.5.0 and 11.5.0 took 1131, 1208 and 938 seconds. Other jobs were building on the same machine when the row started, and nothing in those three cells explains them, so they most likely measure the load rather than the compiler. The G1 cells, run on the same machine under heavier load, took two to five times as long, which shows how much the load moves these numbers (open question 3). A whole cell, with the fetch, the configuration and the checks around the build, adds about 45 seconds. A kernel build scales close to linearly with cores at this size, so the same build on 8 CPUs of gpc should take about 9 or 10 minutes, which is a little more than twice the old estimate.

The average real cell was estimated at about 5 minutes of an 8 CPU job, so the 16,000 tier 1 cells are about 1,300 job-hours. On gpc with four concurrent jobs, that is about two weeks. That is acceptable for a first full sweep, once. It is too much for a weekly one, which is what 9.3 is for. The rest of this document still uses that average. For modern kernels it is about half of what G0 measured, so the real cost is higher, by how much depends on the older eras, which build much faster and which G1 measures.

## 9.3 Frontier search

The working set W(K, P) of 02.6 is believed to be an interval of GCC columns. If it is, the matrix row for (K, P) is fully described by its two edges, and finding an edge in an ordered list of 36 columns takes about 5 cells by binary search instead of 36.

The search per (K, P), on the upstream last-point columns ordered by release date:

1. **Start at the era GCC** e(K), from `kernels.toml`. If the era cell is not **works**, the row is anomalous: it is run dense (every column) and flagged, because the belief failed at its most likely point.
2. **Run the accept probe** (9.4) on every column. Columns that fail L1 are **fails** without a build. The probe is exact, because it runs the kernel's own checks.
3. **Gallop outward** from e(K) toward older columns: e−1, e−2, e−4, e−8, until a cell is not **works** or the probe's boundary is reached. Then binary search between the last **works** and the first failure. Do the same toward newer columns.
4. **The edge cells run to completion.** The last **works** and the first failure on each side get the full ladder, the `-k` run of 9.5, and every applicable twin, because they are the cells that explain the edge.
5. **Sample the interior.** Two interior columns, chosen at random with a seed recorded in the run manifest, run the full ladder. A sampled interior cell that is not **works** is a hole. A hole turns the whole row dense for that run, and becomes a finding (document 08.5). The point columns that a signature pins by exact release, as GCC 14.2 for objtool, are run inside every working set too, since the last points step over them. One of those that does not work is a hole the signature already explains, so it is kept and does not turn the row dense (13, question 1).
6. **Extend to the point columns.** For each edge found, the point releases between the edge column and its neighbour are searched the same way, so that an edge is reported as "4.9.4 works, 5.1.0 fails" rather than "4.9 works, 5 fails".

Per (K, P), that is about 10 builds and about 6 boot sets, against 36 and 36 for the dense row. It cuts the tier 1 sweep to roughly a third, about 450 job-hours.

## 9.4 The accept probe

L1 is decided without building. The kernel's compiler checks are a handful of files that run the compiler on tiny inputs:
- the `#error` lines in `include/linux/compiler-gcc*.h` and `compiler.h`, which fire under `-E`;
- the `#include` of `compiler-gcc<__GNUC__>.h`, which fails under `-E` when the file is missing;
- `scripts/min-tool-version.sh` together with `scripts/cc-version.sh` (5.13 on), and `scripts/gcc-version.sh` before it;
- the Kconfig `CC_IS_GCC` and version checks (4.18 on), which `make olddefconfig` runs.

`gk probe` runs the kernel's own configuration step with `CC` set to the column's compiler, then preprocesses `init/main.c` with the flags kbuild chose (read from its `.cmd` file on the era GCC's cell). That covers every refusal mechanism the kernel has had, costs 2 to 20 seconds, and never guesses: a column the probe passes may still fail later, but a column it fails is certainly **fails** at L1.

## 9.5 The `-k` run

For an edge failure, one number says more than the first error: how many units fail. `gk` re-runs the failing build of each edge cell with `make -k`, collects every failing unit and its first error, and classifies them with the signatures of document 08. The report then says "GCC 10.5 on 5.4.0: 3 units fail, all `-fno-common`" instead of "fails". The unit count over the columns outside the working set also measures how far outside it a GCC is, which is what a porter wants to know.

## 9.6 Dense stripes

Some rows and columns are run in full, every sweep, as a check on the search and because they are the most read:

| Stripe | Content | Why |
|---|---|---|
| the longterm stripe | the last point of every longterm branch ever, every upstream column, tier 1 | these are the kernels people still build with new compilers, and CIP still maintains some |
| the current stripe | the Current set, every upstream column from 8.1, all tier 1 and tier 2 platforms, `defconfig` and `tinyconfig` | the rows rucc cares about most (document 11) |
| the GCC 16 column | every kernel, GCC 16.2, every platform | the column rucc claims to match |
| the era diagonal | every kernel with its era GCC and its era distribution GCC | the reference rucc-kernel grades against |

The dense stripes are about 4,000 cells per sweep, run whole. Their results are also used to test the interval belief: a dense row with a hole would have been missed by the search in 5 of 7 cases, and the sweep report says how many holes the stripes found that the search would not have.

## 9.7 Incremental runs

A cell's identity (02.1) only changes when an input changes. So after the first sweep:
- a new kernel release adds one row, searched as in 9.3, the day it appears;
- a new GCC release adds one column, run dense on the current stripe the day it appears, and searched against every row within the week. A new GCC is the event this repository exists for, and its report is the most useful thing it publishes (10.8);
- a new QEMU pin re-runs the boots of the dense stripes, and every row whose boot result changes is re-searched;
- a rebuilt container or toolchain with a new digest re-runs what used it.

Nothing is re-run because time passed.

## 9.8 Machines

The machines are the ones in the kernel plan's section 13.6, shared with rucc-kernel. gcc-kernel runs at lower priority: it yields to rucc-kernel's nightly run and never runs on server3 while a benchmark does.

| Machine | Role here |
|---|---|
| gpc, 32 cores | the sweep worker: toolchain forging, builds, KVM boots of x86_64 and i386 |
| server3, 8 cores | overflow builds and boots when idle |
| server2, 6 cores and server1, 4 cores | TCG boots of arm64, arm, riscv64, ppc64le, s390x, loongarch64 |
| GitHub `ubuntu-24.04` runners | the harness's own tests, and the accept probe on a new GCC or kernel |
| GitHub `ubuntu-24.04-arm` runners | arm64 native builds, to check that a cross build and a native build of the same cell give the same bytes |

## 9.9 Cost of the whole thing

| Item | Once | Per week after |
|---|---|---|
| forging the toolchains (04.4): about 55 GCC columns × 8 targets, plus binutils | about 60 job-hours of gpc | only new releases |
| host and forge containers | about 10 hours | none |
| first full sweep, searched, tier 1 | about 450 job-hours | |
| dense stripes | about 330 job-hours | about 30, incremental |
| tier 2 searched | about 400 job-hours, TCG | about 20 |
| new kernel releases and point releases | | about 10 |

Storage:

| Item | Size |
|---|---|
| kernel tarball cache | about 17 GB, shared with rucc-kernel's cache by content hash |
| toolchain bundles, compressed | about 15 GB |
| host and forge containers | about 25 GB |
| scratch per concurrent build | 2 to 4 GB, deleted after the cell |
| kept results: JSON, classified errors, consoles of boot cells | about 2 KB per failing build cell, 50 KB per boot cell, about 3 GB in all |
| kept full logs | edge cells and holes only, compressed, about 5 GB |

Engineering cost is in document 12.
