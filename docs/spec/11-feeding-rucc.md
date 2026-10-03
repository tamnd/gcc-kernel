# What rucc and rucc-kernel take from the matrix

gcc-kernel is independent: it never runs rucc and would be worth having if rucc did not exist. It exists now because rucc needs it. rucc claims to be compatible with GCC 16, and the kernel plan ([rucc-kernel's plan](https://github.com/tamnd/rucc-kernel/blob/main/docs/plan/00-README.md)) grades rucc against a period GCC per era. Both claims rest on facts about GCC that nobody has measured: which GCC builds which kernel, what changes in the kernel's build when the GCC changes, and where the era boundaries really are. This document lists what flows from gcc-kernel to rucc and rucc-kernel, in what form, and which open questions of the kernel plan it settles.

The direction is one way. rucc-kernel reads `matrix.json` and gcc-kernel build directories at a pinned sweep. gcc-kernel reads nothing from rucc-kernel.

## 11.1 The GCC 16 column is rucc's target on recent kernels

The goal the user stated for rucc is to compile recent kernels. rucc's own claim is compatibility with the latest GCC, which is GCC 16.2 today. The GCC 16 column of the matrix, run dense on every platform (09.6), turns that claim into a checkable set:

**The GCC 16 set** S16(P) is every kernel whose cell with GCC 16.2 on platform P is **works**.

For every kernel in S16(P), a rucc build with the GCC 16 persona is held to the GCC 16 build. That is a second reference next to the era GCC of the kernel plan's section 4.2, and the kernel plan already names GCC 16.2 as a second reference on Current. gcc-kernel widens it from Current to every kernel GCC 16 can build, which the dense column measures. Three consequences:

1. **rucc-kernel gets a GCC 16 baseline for free.** The cell directories of the GCC 16 column are rucc-kernel build directories by the shared schema of 10.4. `rk test --reference <gk cell dir>` works without rebuilding the reference, and `rk config-diff`, `rk flags-diff` and `rk sections-diff` do too.
2. **Kernels outside S16 are not rucc's problem under the GCC 16 persona.** If GCC 16 fails to build 4.9.337, rucc with the GCC 16 persona may fail too, and the failure is expected. The kernel plan grades those kernels under their era persona instead. gcc-kernel's classification says why GCC 16 fails (for example, the gnu23 default of GCC 15 meeting `bool` in a tree without `-std`), and rucc must fail on the same kernels for the same reason, or have a written reason not to.
3. **"Compatible with GCC 16" gets a kernel-sized test.** Where the GCC 16 cell fails and the rucc cell under the GCC 16 persona passes, rucc accepted something GCC 16 rejects. That is a compatibility bug in rucc's claim, even though the kernel ran, and it is reported as one.

## 11.2 Era boundaries from evidence

The kernel plan's rule 4 in section 4.2 says an era boundary moves only with evidence, and its open question 10 asks which 3.x and 4.x stable branches a later GCC can build. The frontier file answers both:

| Kernel plan item | Answered by |
|---|---|
| is the era GCC in `personas.toml` actually a working GCC for every version in the era? | `frontiers.json`: e(K) ∈ W(K, P) for every K in the era |
| where should an era split? | the kernels where W(K, P) stops containing the era GCC |
| can a later era's GCC build this version's last point (open question 10)? | the row for the last point, and W's maximum |
| does a single persona fit a range of versions? | the intersection of W over the range. An empty intersection means the era must split |
| which binutils goes with the era? | the binutils sweep of 04.6 |

`gk` writes a proposed `personas.toml` diff for rucc-kernel as a report (`reports/eras.md`), never as a pull request to rucc-kernel. A person decides, because the persona also has to match what the era's distributions shipped.

The persona version itself is not chosen by the matrix. The kernel plan picks the GCC a distribution shipped, and that stays. What the matrix adds is the check that the choice works, and the fallback choice when it does not: the working column nearest to the distribution's.

## 11.3 What the kernel notices about a GCC

A persona is only correct if the kernel sees the same thing when it probes rucc as when it probes the GCC being impersonated. gcc-kernel measures what the kernel sees, per GCC column:

- **The configuration differential across columns.** `gk config-diff K G1 G2` on the dense stripes gives, for each kernel, every `.config` line that changes between consecutive GCC columns. That is the list of `CC_HAS_*`, `CC_VERSION_TEXT`, `GCC_VERSION` and dependent symbols that a persona decides. rucc-kernel's `config-divergences.toml` for a persona must be a subset of the lines that differ between neighbouring real GCCs, and never contain a line that no GCC change ever touched. On the Current set, `gk publish` writes the same comparison for every kernel as `reports/config-differential.md`, gathered per step between neighbouring columns.
- **The flags differential across columns.** `gk flags-diff` gives the per-unit command lines that change with the GCC version, before Kconfig probes existed (4.18). For older kernels that is the only record of what `cc-option` decided.
- **The version gates.** `gk gates K` lists every place a tree tests the compiler's version: `GCC_VERSION >= 40600`, `__GNUC__ >= 4`, `cc-ifversion`, `gcc-min-version`, `$(call cc-option,...)`, Kconfig `depends on GCC_VERSION >= 110000`. Joined with the columns, it says which gates flip at which GCC release. That is exactly the list of behaviours rucc's persona must switch at the same version, and the kernel plan's section 4.2 rule 3 needs it.

The output for rucc is `reports/persona-surface.md`: per GCC release, the gates that flip, the Kconfig symbols that change, and the flags kbuild starts or stops passing, over all kernels. It is the table rucc's `-fgnuc-version` has to honour, with evidence.

## 11.4 The demand census per GCC

The kernel plan's `rk demands` counts which compiler features a kernel build uses. gcc-kernel computes the same census from the gcc-kernel build directories, for every GCC column on the Current set, and adds a dimension: the features a kernel uses only when the GCC has them. `asm goto` with outputs, `__counted_by`, named address spaces, `-fzero-call-used-regs`, `-fstrict-flex-arrays=3`, `-ftrivial-auto-var-init`, `-mfunction-return=thunk-extern`, `-fmin-function-alignment` are all turned on by probes. The census per column tells rucc what a persona of that version will be asked for, before a single rucc build runs. Under the GCC 16 persona, rucc will be asked for everything the GCC 16 cells use.

## 11.5 The warning census

From 5.15, `CONFIG_WERROR` exists and `allmodconfig` turns it on through `COMPILE_TEST`, and many directories have added `-Werror` of their own before that. A compiler that warns where GCC is silent breaks those builds. gcc-kernel records the warnings of every Current cell by option and unit (`warnings.jsonl`) and publishes the count per GCC column in `reports/warning-census.md`, written by `gk publish`, with the column that first gives each warning option and the units built with `-Werror` that warn. For rucc this gives two rules, checked by rucc-kernel:
1. under a persona, rucc must not emit a warning on a unit where that GCC version emits none, when the unit builds with `-Werror`;
2. the warning options the kernel disables per GCC version (`-Wno-*` from `cc-disable-warning`) must be accepted by rucc under that persona.

The census across columns also documents which GCC release introduced each warning that broke kernels, which is the other half of the failure catalog.

## 11.6 Known failure shapes

The failure catalog (document 08) is the list of every way a real GCC has failed to build or run a kernel, with the kernel commit that fixed it. Two uses in rucc:
- **rucc must reproduce the refusals it claims.** Under the persona of GCC N, a kernel that GCC N refuses at L1 must be refused by rucc the same way, because the kernel's own checks do it. A rucc build that gets past L1 where GCC N stops means the persona leaks.
- **rucc must not reproduce the miscompilations.** The catalog's miscompile class (for example the GCC 4.1.0 `__weak` miscompile, the arm 4.8.0 to 4.8.2 blacklist and the 4.9.0 `load_balance()` miscompile) are bugs, and rucc under those personas must build a kernel that runs. The kernel's own blacklists refuse most of those versions by number, so the kernel plan never picks them as personas. The catalog makes that an explicit list rather than a coincidence.

Each catalog entry that names a compiler behaviour becomes a candidate test in `rucc-corpus` or `rucc-compat`, written from the description, never by copying kernel code, as the kernel plan's section 14.3 requires.

## 11.7 Toolchains and host environments

The kernel plan's era containers (`provision/eras/` in rucc-kernel, section 4.4 of the plan) and gcc-kernel's host environments (document 05) solve the same problem. gcc-kernel's are a superset: a Debian host for every era from bo (1997) to trixie, and the upstream toolchain bundles built from source for every GCC release. rucc-kernel may pin gcc-kernel's images by digest in its `toolchains.toml` instead of building its own. That is a pinned data dependency, the same kind as a kernel tarball, and it does not make rucc-kernel run gcc-kernel code. The decision belongs to rucc-kernel (open question 7 here).

The toolchain bundles also give rucc-kernel what its plan's section 4.4 wanted and did not have: a real GCC 2.7.2.3, 2.95.3 and 3.x for the museum eras, built from source and checked, and a GCC 16.2 cross toolchain for every tier 1 and tier 2 platform.

## 11.8 Open questions of the kernel plan this settles

| Kernel plan question | What gcc-kernel provides | When |
|---|---|---|
| 4, what the museum really includes, a.out and 0.x | the museum rows with their era toolchains: whether 1.0 builds with GCC 2.5 or 2.7 at all, whether it boots under QEMU, and whether 0.01 can be built unpatched (document 07.7) | G3 |
| 7, disk, CI time and machines | measured build and boot times for every era on gpc, from real cells | G1 |
| 10, which 3.x and 4.x stable branches a later GCC builds | the longterm stripe | G2 |
| era boundaries in `personas.toml` | `reports/eras.md` | G2 |

## 11.9 What gcc-kernel does not tell rucc

It does not say whether rucc is right. A green GCC 16 cell with a red rucc cell is rucc-kernel's finding, not gcc-kernel's, and the bisection (`rk mixed`) happens there. gcc-kernel also does not rank rucc's work. The kernel plan's demand census does. gcc-kernel provides the measured ground under both.
