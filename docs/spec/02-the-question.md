# The question, stated so a machine can answer it

The question behind this repository is simple: which GCC builds which Linux kernel, on which platform, into a kernel that runs? A person can answer it for one pair in an afternoon, and nobody has answered it for all of them. This document makes the question precise enough that `gk`, the tool in `tamnd/gcc-kernel`, answers it without anyone reading a log. It defines the cell, the outcome ladder a cell climbs, the rules that keep an answer honest, and what does not count.

## 2.1 The cell

A **cell** is one attempt to build and run one kernel with one toolchain. It has six coordinates, and every one of them is a name in a committed file:

| Coordinate | Symbol | Example | Defined in |
|---|---|---|---|
| kernel tree | K | `2.6.32.71` | `kernels.toml` (document 03) |
| compiler | G | `gcc-4.4.7` | `gccs.toml` (document 04) |
| binutils | B | `binutils-2.20.1` | `binutils.toml` (document 04) |
| platform | P | `x86_64/q35` | `platforms.toml` (document 06) |
| configuration | C | `defconfig+gk` | `configs/` (document 07) |
| host environment | H | `gk-host-squeeze` | `hosts.toml` (document 05) |

Two more inputs are fixed per run rather than per cell, and recorded in the run manifest: the QEMU build and the `gk` commit.

A cell's identity is the SHA-256 of the canonical JSON of its six coordinates and the digests they resolve to: the tarball hash of K, the toolchain bundle digest of G and B, the container digest of H, the hash of the merged configuration input, and for a cell that boots, the digest of the `gk-boot` container and of the initramfs. A cell that stops at L4 has no boot coordinates, so its identity does not change when the rig does. Two cells with the same identity are the same experiment, and `gk` runs an experiment once unless told to repeat it. A change to any input, including a rebuilt toolchain whose bytes differ, gives a new identity and a new cell.

The toolchain pair (G, B) is written together as T when the pairing rule of document 04.5 picked B. Most of the matrix is K × T × P with C and H determined by K. The binutils sweep of document 04.6 is the exception, and it varies B alone.

## 2.2 The outcome ladder

Each cell climbs a ladder and stops at the first rung it fails. The rung it reached, and the reason it stopped, is the result.

| Rung | Name | Passes when |
|---|---|---|
| L0 | fetched | the tree's SHA-256 matches the pin, and it unpacks |
| L1 | accepted | the kernel's own compiler checks accept G: no `#error` from a `compiler-gcc*.h`, no refusal from `scripts/min-tool-version.sh` or `cc-version.sh`, no missing `compiler-gccN.h` |
| L2 | configured | the configuration step finishes, and every option the test fragment asked for is set in `.config` |
| L3 | compiled | every translation unit the configuration selects compiles and assembles, with the kernel's own `-Werror` settings untouched |
| L4 | linked | `vmlinux` links and the boot image for P is produced, and on kernels that run them, `objtool` and `modpost` finish without an error |
| L5 | booted | the kernel reaches `gk-init` under QEMU and prints the boot marker within the time budget |
| L6 | smoke | every check of the era's smoke suite passes, and the machine powers itself off |
| L7 | tested | every KUnit suite and in-kernel self test that the fragment builds passes, where the kernel has them |
| L8 | clean | the build log has no `objtool` warning and the console has no splat: `WARNING:`, `BUG:`, `Oops`, `general protection fault`, `KASAN:`, `UBSAN:`, `RCU stall`, `soft lockup`, `hung task` |

"Compiles and runs correctly" in the user's words means L6. L7 and L8 are graded where they exist, and a kernel that has neither KUnit nor objtool (everything before 4.6 for objtool and 5.5 for KUnit) passes them vacuously, marked as such.

Three rungs have a lenient twin, run only when the strict rung fails and only to classify the failure:

| Twin | Differs from the strict rung by |
|---|---|
| L3w | `KCFLAGS=-Wno-error`, and `CONFIG_WERROR=n` where the tree has it. A cell that fails L3 and passes L3w failed on a warning the kernel made fatal, not on an error the compiler made |
| L4o | `OBJTOOL_ARGS` unchanged but objtool failures made non-fatal where the tree offers a switch for it (`CONFIG_OBJTOOL_WERROR=n`), which only exists from 6.x |
| L5k | the boot repeated with `-cpu` reduced to the model the kernel's era targets (document 06.4). A cell that fails L5 and passes L5k failed on the emulated machine, not on the compiler |

A twin never raises the strict result. The matrix cell stays at the strict rung, and the twin is reported next to it as the reason, for example `L2 → L3w`: fails to compile with the tree's own flags, compiles when warnings are not errors.

## 2.3 Verdicts

The ladder gives each cell one of five verdicts, which are what the matrix shows:

| Verdict | Meaning | Cell colour in `reports/` |
|---|---|---|
| **works** | reached L6, and L7 and L8 where the kernel has them | green |
| **runs** | reached L6 and failed L7 or L8: it boots and runs, but a test or the log says something is wrong | yellow |
| **builds** | reached L4 and failed L5 or L6: it links and does not run | orange |
| **fails** | stopped at L1, L2, L3 or L4 | red, with the twin's letter when a twin passed |
| **n/a** | the cell cannot exist: G has no back end for P's architecture, or P's architecture is not in K | grey |

A sixth state, **not run**, is the absence of a result, never a verdict. The reports distinguish it from **n/a** everywhere.

## 2.4 Repetition and flakiness

A build is deterministic enough to run once. A boot is not: timing, the TCG scheduler and entropy make a boot fail now and then for reasons that have nothing to do with the compiler. The rule:

1. **Cells at L4 and below are run once.** Every build is also run a second time at a sample rate of 2%, and the two `vmlinux` files are compared. A difference is recorded as a nondeterminism finding against the toolchain, which is itself interesting data about old GCC versions.
2. **Cells that reach L5 are run three times.** The verdict is the lowest rung reached in any of the three. A cell whose three runs disagree is marked **flaky**, keeps the lowest verdict, and its three consoles are kept.
3. **A flaky cell is not re-run until it passes.** It is re-run only when its identity changes.

## 2.5 The rules

**Unmodified kernels.** The tree is the pinned tarball, byte for byte, checked by SHA-256 on every use. Nothing is patched. Nothing in it is regenerated by us. The harness never writes into the tree, except for kbuild's own output under `O=` where the kernel supports `O=` (2.6 onward), and the in-tree objects of kernels that do not (2.4 and earlier), which are built in a scratch copy that is thrown away.

**Unmodified toolchains.** Every GCC and binutils of the upstream flavor is built from the release tarball on `ftp.gnu.org` or `gcc.gnu.org`, checked by hash, with nothing patched. Only configure options change, and they are fixed per release line in `gccs.toml`. A release that cannot be built unpatched in any of the forge environments of document 04.4 is recorded as **unbuildable**, with the log, and it has no column. The distribution flavor of document 04.3 uses the distribution's own binaries, which carry the distribution's patches, and that is the point of it.

**Configuration only through documented interfaces.** Allowed:
- a configuration target the kernel ships (`defconfig`, `tinyconfig`, `allnoconfig`, `allmodconfig`, `<board>_defconfig`, `oldconfig` fed with defaults);
- an in-tree fragment;
- the published test fragment of the era (document 07.2), which only turns things on;
- the make variables the kernel documents: `ARCH`, `CROSS_COMPILE`, `CC`, `LD`, `AS`, `HOSTCC`, `O=`, `KBUILD_BUILD_TIMESTAMP`, `KBUILD_BUILD_USER`, `KBUILD_BUILD_HOST`, and `V=1` for logs.

`KCFLAGS` is empty on every strict rung. The only place it is set is the L3w twin, which says so.

**The kernel's era picks the host, the cell picks the compiler.** `HOSTCC`, `make`, `perl`, `bison`, `flex`, `bc`, `python` and `openssl` come from the host environment H, which is fixed by K (document 05). Only `CC` and the binutils change across the row of the matrix. So a failure in a row is a failure of the target toolchain, not of a newer perl or a newer host compiler building `scripts/`. The opposite experiment, where `HOSTCC` is the compiler under test too, is the native mode of document 05.5. It is recorded, and it is not the matrix.

**One cell, one sandbox.** Every build runs in a fresh container from the pinned digest, with the tree and the toolchain bundle mounted read-only, no network, and a fixed `SOURCE_DATE_EPOCH` and `KBUILD_BUILD_*`. The build gets a fixed number of CPUs and a memory cap, both recorded, so that an out-of-memory kill is a reproducible result rather than a scheduling accident.

## 2.6 Frontiers

The matrix answers pair by pair, but the useful answers are ranges. For a kernel K on platform P, define:

| Name | Definition |
|---|---|
| **working set** W(K, P) | the set of upstream GCC columns whose cell is **works** |
| **oldest working GCC** min(K, P) | the oldest column in W |
| **newest working GCC** max(K, P) | the newest column in W |
| **era GCC** e(K) | the GCC the kernel was developed and released with (document 03.3) |
| **holes** | columns strictly between min and max that are not **works** |

The belief, from how kernels and compilers change, is that W is an interval: a kernel refuses GCCs that are too old, breaks on GCCs that are too new, and works in between. The belief is tested, not assumed. Document 09 samples the interior, and every hole found is a finding in its own right: a GCC release that miscompiled or refused a kernel its neighbours handle. The kernel's own refusals of GCC 3.0 and 3.1 (from 2.6.29), 4.1.0 and 4.1.1 (the `__weak` miscompile), and 4.8.0 to 4.8.2 on arm (PR 58854) are the known holes, together with the unblacklisted 4.9.0 and 4.9.1 scheduler miscompile (PR 61801) that the kernel worked around instead (document 01.3).

Symmetrically, for a GCC column G, the **kernel range** of G on P is the set of kernel versions G builds and runs. That is the view a compiler writer wants, and it is the one rucc reads (document 11).

## 2.7 What does not count

These are not results, and `gk` refuses to record them as such:
- a cell built from a tree or toolchain whose hash does not match its pin;
- a cell that set `KCFLAGS`, `KCPPFLAGS`, `EXTRA_CFLAGS` or any make variable outside the list in 2.5, except the L3w twin;
- a cell whose `.config` was edited after the configuration step;
- a boot with a looser timeout than the platform's budget, or a retried boot counted as a pass;
- a cell built in a host environment other than the one `hosts.toml` assigns to K, unless it is labeled native mode or a host sweep;
- a cell run with a `gk` commit that has uncommitted changes, unless it is labeled ungraded.

## 2.8 What this repository is not

It is not a compiler test. It never runs rucc, and nothing in it depends on rucc. It records what real GCC does, so that a compiler that claims to be GCC can be held to it. It is not a patch collection either: the kernel commits that fixed a GCC incompatibility are named by hash in the failure catalog (document 08), and never applied. And it is not a benchmark. Performance of the built kernels is out of scope, because the question is whether a pair works, and rucc-kernel already owns performance.
