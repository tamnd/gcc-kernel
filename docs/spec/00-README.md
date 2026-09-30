# The gcc-kernel spec

This folder is the specification for gcc-kernel. The repository measures, for every Linux kernel release from 0.01 to today and every GCC release from 1.x to 16.2, on every platform QEMU can boot, whether that GCC builds the kernel into one that boots and passes its tests. The product is a published matrix with every edge explained: the first GCC that works, the last one that does, why the ones outside fail, and the kernel commit that changed it.

## Why it exists

The goal is rucc, the C compiler written in Rust that aims at full compatibility with GCC 16, and rucc-kernel, the harness that builds kernels with it ([rucc-kernel's plan](https://github.com/tamnd/rucc-kernel/blob/main/docs/plan/00-README.md)). Both lean on facts about GCC that nobody has measured:
- rucc claims GCC 16 compatibility. Which kernels does GCC 16 itself build and boot? Where GCC 16 fails, rucc under the GCC 16 persona should fail the same way.
- rucc-kernel grades each kernel era against a period GCC persona. Does that GCC actually work for every version in the era, and where should the era boundaries be?
- A persona has to look like the real GCC to the kernel's probes. What changes in a kernel's `.config`, flags and version gates when the GCC changes?

gcc-kernel answers these with real GCCs only. It never runs rucc, and data flows one way, into rucc-kernel (document 11). The recent kernels are the point, because that is where rucc is going. The old ones are the reference: they show how the kernel and GCC have moved together, which is what a persona has to reproduce.

## Settled decisions

1. **A cell is (K, G, B, P, C, H)**, identified by the hash of its inputs, graded on a ladder from L0 fetched to L8 clean. "Compiles and runs correctly" is L6 (02).
2. **Nothing is patched.** Kernels are unmodified tarballs and toolchains are built from release tarballs. What cannot be built unpatched is recorded as unbuildable (02.5, 04.4).
3. **The kernel picks the host.** `HOSTCC` and the host tools come from a Debian container of the kernel's era. The GCC under test comes in through `CROSS_COMPILE` as a static bundle. A failure along a row is the toolchain's (05).
4. **Frontier search, not a dense sweep.** Rows are searched outward from the era GCC for their edges. Interior samples look for holes, and chosen stripes (Current, longterm, GCC 16, the era diagonal) are dense (09).
5. **GCC 16 first.** The GCC 16 column and the Current set come first. The museum comes last (12).
6. **Every failure gets a name.** First errors are classified by signatures with the fixing kernel commit, confirmed by bisection (08).
7. **Shared schema with rucc-kernel.** `gk-cc` is rucc-kernel's `rk-cc`, and a gcc-kernel cell directory is a rucc-kernel build directory, so rucc-kernel's diff tools read gcc-kernel cells unchanged (10.4).

## Reading order

| Doc | Title | What it holds |
|---|---|---|
| 01 | [Research, October 2026](01-research-2026.md) | GCC release history, the kernel's minimum GCC over time, blacklists, breakages per GCC release with fixing commits, early kernels, Debian hosts, prior art |
| 02 | [The question](02-the-question.md) | the cell, the ladder, verdicts, rules, frontiers |
| 03 | [The kernel axis](03-kernel-axis.md) | sets of trees, sources, eras and the era GCC, the full history clone |
| 04 | [The GCC axis](04-gcc-axis.md) | columns, distribution flavors, the forge, binutils pairing and sweep |
| 05 | [Host environments](05-host-environments.md) | why the kernel picks the host, the host images, how a toolchain runs in any host |
| 06 | [Platforms](06-platforms.md) | tiers, QEMU machines and CPUs by kernel age, QEMU pins |
| 07 | [Building and booting a cell](07-build-and-boot.md) | configuration, build, boot rig, `gk-init`, suites, the 0.x experiment |
| 08 | [Outcomes and signatures](08-outcomes-and-signatures.md) | first errors, the signature catalog and its seed, holes |
| 09 | [Search and cost](09-search-and-cost.md) | matrix size, the frontier search, dense stripes, machines, cost |
| 10 | [The gcc-kernel repository](10-gcc-kernel-repo.md) | layout, `gk` commands, file formats, result store, publication, CI |
| 11 | [Feeding rucc](11-feeding-rucc.md) | the GCC 16 set, era evidence, the persona surface, what rucc-kernel reads |
| 12 | [Milestones](12-milestones.md) | G0 to G6, about 21 engineer-weeks |
| 13 | [Open questions](13-open-questions.md) | what is not decided yet, with proposed answers |

Someone who only wants to know what the repository will do should read 02, 09 and 11. Someone building it should read 05, 07, 10 and 12. Document 01 is the evidence and can be read as needed.

## Relation to rucc-kernel

The [rucc-kernel plan](https://github.com/tamnd/rucc-kernel/tree/main/docs/plan) is what rucc-kernel implements. This spec refers to its eras (E0 to E11), personas and open questions by number.

## Conventions

Kernel versions are written without a `v` except as git tags. GCC versions are the release numbers of gcc.gnu.org. "First tag" is the first mainline tag containing a commit. Facts not checked against a primary source are marked **(unverified)**, and the milestone that needs them checks them.
