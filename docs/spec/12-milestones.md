# Milestones

Seven milestones, G0 to G6. The order follows what rucc needs first, not the chronology of the kernel. The GCC 16 column and the Current set come first, because they are rucc's target. The museum comes last, because it only serves as reference. Each milestone has a scope, an exit criterion that a `gk` command checks, and a cost in engineer-weeks for one engineer who knows rucc-kernel's code. The costs are estimates made before anything has run. G0's measurements replace them.

## Summary

| | Scope | Cost | Serves |
|---|---|---|---|
| G0 | the instrument: repository, model, shim, modern forge, one dense row | 3 weeks | everything |
| G1 | the current stripe on tier 1, boots and KUnit, the GCC 16 column | 3 weeks | rucc-kernel K1 to K5 |
| G2 | GCC 4.6 to 7.5, releases 3.0 to 7.x, the longterm stripe, the failure catalog | 4 weeks | the kernel plan's era table, open question 10 |
| G3 | the legacy forge (GCC 1.x to 4.5), 2.6.x, 2.4, the museum, the 0.x experiment | 5 weeks | rucc-kernel K10 and K11, open question 4 |
| G4 | tier 2 platforms | 3 weeks | rucc-kernel K6 and K7 |
| G5 | distribution columns and the binutils sweep | 2 weeks | persona accuracy |
| G6 | steady state: watch, new GCC reports, HTML | 1 week, then upkeep | everyone |

Total about 21 weeks, and G3 is the least certain. G4 and G5 can run in parallel with G3 on a second engineer.

## G0: the instrument

**Scope.**
- The repository with the layout of 10.2. `gk-model` with cell identity. `gk-cc` copied from `rk-cc` with the two added `build.json` fields.
- `kernels.toml` with the Current set and the releases 5.0 to 7.x. `gccs.toml` with the upstream last points 8.5 to 16.2, and 16.1.
- Host containers `gk-host-bookworm` and `gk-host-trixie`. Forge container `gk-forge-modern`. Toolchain bundles for `x86_64-linux-gnu` and `aarch64-linux-gnu` for 8.5 to 16.2, with binutils paired by 04.5.
- `gk forge`, `gk forge verify`, `gk probe`, `gk cell` to L4, and the result store.

**Exit criterion.** `gk search 7.2.8 --platform x86_64 --dense` runs every column from 8.5 to 16.2 to L4, and the verdicts are in `matrix.json`. The build time of each cell is recorded, and document 09's estimates for modern kernels are replaced by measurements.

## G1: the current stripe

**Scope.**
- `gk-boot`, `gk-init` for 2.6 on, the fragments for eras M13 and M14 (03.3, the kernel plan's E10 and E11), boots to L6, KUnit to L7, splat detection for L8, the three-run rule.
- The i386 platform and its bundles. The current stripe of 09.6, dense, on x86_64, i386 and arm64, `defconfig` and `tinyconfig`.
- The GCC 16 column, dense over releases 5.0 to 7.x.
- The warning census (11.5) and the config differential across columns (11.3) on the Current set.
- `gk publish` with the markdown heat maps.

**Exit criterion.**
1. The current stripe is complete, and every cell has a verdict or a reason for **n/a**.
2. rucc-kernel runs `rk config-diff` with a gcc-kernel GCC 16 cell directory as one side, unmodified, and gets a table. That proves the data contract of 10.4.
3. `reports/new-gcc-16.2.md` exists, written as if 16.2 were new, as a rehearsal of 10.8.

## G2: the modern history

**Scope.**
- Forges for GCC 4.6 to 7.5 (jessie and stretch forges, 04.4). Host containers from wheezy to bullseye.
- Kernels from 3.0 to 4.20 and the last points of every longterm branch in that range.
- The frontier search of 09.3, interior sampling, and the `-k` run.
- The failure catalog, `signatures.toml`, seeded with every entry of document 08.4, and `gk classify`, `gk triage`, `gk explain`.
- `gk bisect-kernel` with the full history clone.
- `reports/eras.md` for rucc-kernel (11.2).

**Exit criterion.**
1. Every kernel from 3.0 on has frontiers on x86_64 and arm64, and the longterm stripe is dense.
2. Every edge cell's first error is classified, or listed by `gk triage` with an issue. Unclassified edges are under 5%.
3. `reports/eras.md` states, for each era of the kernel plan, whether the era GCC works for every version in it, and the kernel plan's open question 10 is answered in its own document with a link.

## G3: the legacy history

**Scope.**
- The legacy forge chain of 04.4: GCC 4.0 to 4.5 in etch and lenny, 3.x in sarge and etch, 2.95.3 and egcs in potato and woody, 2.7.2.3 in hamm or slink, 2.5.8 and 1.x experiments, i386 host binaries.
- Kernels 2.6.0 to 2.6.39 and their last points, 2.4, 2.2, 2.0, 1.2, 1.0.
- `gk-init` for the museum, the initrd and floppy boot paths of 07.6, `isapc`.
- The 0.x experiment of 07.7, time-boxed to one week.

**Exit criterion.**
1. Every release from 2.6.0 has frontiers on x86_64 (from 2.6.0, where the x86_64 port exists in the tree) and i386.
2. Every museum kernel has a verdict with its era GCC, and either a frontier or a written reason why the toolchain for it could not be built.
3. The 0.x experiment has a written result: which of 0.01, 0.11, 0.12 and 0.99.15 built unpatched and booted, with what, or why not. The kernel plan's open question 4 is answered with a link.

**Risk.** This is where the unknowns are: building GCC 2.x and 1.x without patches in a container that still runs, booting 1.0 on current QEMU, and whether Linus's `-mstring-insns` GCC for 0.01 can be reconstructed at all. The milestone is allowed to end with **unbuildable** records instead of cells, as long as each has its log.

## G4: tier 2 platforms

**Scope.** arm, riscv64, ppc64le, s390x and loongarch64: bundles for every GCC that has the target (04.2), QEMU machines per era (06), fragments, and the searched sweep over the kernels that have each architecture. Tier 3 platforms get build-only cells on the Current set with GCC 16.2.

**Exit criterion.** Every tier 2 platform has frontiers for every kernel that has it, and the Current set with GCC 16.2 builds or has a classified failure on every tier 3 platform.

## G5: distributions and binutils

**Scope.**
- The distribution flavor columns of 04.3: Debian and Ubuntu GCC packages that changed a default the kernel notices, run on the kernels around each change.
- The binutils sweep of 04.6.
- `reports/persona-surface.md` (11.3).

**Exit criterion.** The known distribution breakages (default PIE, default stack protector, default `-fcf-protection`) are each reproduced by a cell and each has a signature. The binutils sweep has run for the longterm stripe on x86_64.

## G6: steady state

**Scope.** `watch.yml` opens pin pull requests for new kernel tags, GCC and binutils releases. `sweep.yml` runs incrementally. The new GCC report of 10.8 is produced automatically. The HTML rendering is published.

**Exit criterion.** A kernel release and a GCC release happen with nobody touching the repository, and both show up in `matrix.json`, with their report, within a week.

**Upkeep.** About half a day a week: triage of unclassified failures, new signatures, and the occasional host container that breaks because an archive moved.
