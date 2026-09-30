# Open questions

Each question has a proposed answer, the evidence that settles it, and the milestone by which it must be settled. None of them blocks G0.

**1. Is W(K, P) really an interval?** The frontier search (09.3) assumes it and samples the interior to check. If holes turn out to be common rather than rare (more than about 5% of rows), the search saves little and a dense sweep of tier 1 is cheaper to reason about. Proposed: keep the search, measure the hole rate on the dense stripes at G1 and G2, and switch tier 1 to dense if it is above 5%. Settled by G2.

**2. What counts as "runs correctly" for a museum kernel?** 2.6 and later have the smoke suite and, from 5.5, KUnit. A 1.2 kernel has fork, exec and a pipe. That is a much weaker claim, and the matrix prints the same `W` for both. Proposed: the published matrix marks museum cells with the suite they ran, and the heat map uses `W` only where the full smoke suite ran, `R` otherwise. Settled at G3.

**3. What does a cell actually cost?** Document 09's estimates are made before anything has run. G0 measures builds of modern kernels, G1 boots and KUnit, G3 the legacy builds. If the tier 1 sweep comes to more than about twice the estimate, tier 2 moves to Current and the longterm stripe only. Settled by G1 for modern kernels, G3 for the rest.

**4. Can Linus's `-mstring-insns` GCC be found or rebuilt?** 0.01 to 0.12 need it. The oldlinux archives hold GCC 1.40 binaries for Linux 0.1x, which may be the patched build, and the patch itself may be recoverable from them. Proposed: the 0.x experiment of 07.7, time-boxed to a week. A patched GCC 1.40 rebuilt from a recovered patch would violate "unmodified toolchains" (02.5), so it would produce an ungraded report, not cells. Settled at G3.

**5. How far back can the forge build unpatched?** Building GCC 2.x in an i386 Debian container of its time should work, since that is how Debian built it. GCC 1.x in an emulated Linux 0.12 might not. Anything that needs a patch to build is **unbuildable** by rule, and the matrix loses those columns. Proposed: accept the loss and document it. Revisit only if a whole era has no buildable column. Settled at G3.

**6. Distribution columns beyond Debian and Ubuntu.** Fedora, Arch and SUSE ship GCC with their own defaults, and Arch ships new GCC majors within weeks, so Arch users hit new breakages first. Proposed: no columns for them until a finding shows a kernel break specific to that distribution's GCC that the upstream column does not show. Settled at G5.

**7. Should rucc-kernel use gcc-kernel's host images and toolchain bundles?** The kernel plan's era containers and gcc-kernel's hosts overlap. Sharing them saves rucc-kernel a forge and makes both repositories use the same period GCC byte for byte. It also makes rucc-kernel depend on a registry it does not own. Proposed: rucc-kernel pins gcc-kernel images and bundles by digest in its `toolchains.toml`, as data, and keeps its own Dockerfiles as a fallback. The decision is rucc-kernel's. Settled when rucc-kernel reaches its museum milestones (K10, K11).

**8. KVM or TCG for x86 boots?** KVM is much faster and the budget depends on it (09.2), but it exposes the machine's CPU to old kernels. Proposed: KVM from 2.6.25 on after the G1 comparison of 06.6, TCG before. If the comparison shows any verdict difference on the era diagonal, TCG everywhere and accept the cost. Settled at G1.

**9. `CONFIG_WERROR` and arch `-Werror`.** The matrix builds with `CONFIG_WERROR=n`, but several architectures and directories have their own `-Werror` (sparc, powerpc's `PPC_WERROR`, alpha, mips, some drivers, objtool). A new warning therefore fails some platforms and not others with the same GCC. That is a real finding, and the L3w twin shows it, but it could also make tier 3 look worse than it is. Proposed: report it as is, with the class `new-warning-werror`, and never turn off an architecture's own `-Werror` (that would change the kernel, which 02.5 forbids). Settled by G4.

**10. `allmodconfig` on the Current set.** It exercises far more code than `defconfig`, and it is where most new-GCC breakages first show. It is also about ten times as expensive. Proposed: `allmodconfig` build-only on the Current set, the GCC 16 column and the newest two last points, x86_64 and arm64 only. Settled at G1, from its measured cost.

**11. Clang.** The question the user asked is about GCC, and rucc aims at GCC compatibility. A clang axis would reuse everything here: host environments, the ladder, the boot rig and the catalog. Proposed: out of scope. The design keeps `flavor` open, so a `clang` flavor column could be added later without changing the model. Not scheduled.

**12. Rust.** `CONFIG_RUST=n` everywhere. Kernel Rust needs its own rustc and bindgen versions, which is a third axis. Proposed: out of scope, like clang. Not scheduled.

**13. Should every log be kept?** Document 09.9 keeps full `make` logs and `compile.jsonl` only for edge cells and holes, about 5 GB. That is enough to explain every edge, but a later question about an interior cell (for example, a new warning census over an old sweep) then needs a rebuild. Keeping everything is a few hundred GB per full sweep, compressed **(estimate, measured at G1)**. Proposed: keep everything for the Current set and the GCC 16 column, edges and holes elsewhere, and rebuild on demand, which identity makes exact. Settled at G2.
