# Platforms

A platform is an architecture, the QEMU machine that boots it, and the details that change with the kernel's age: the kbuild `ARCH` name, the machine model, the CPU model, the console, and the image format. This document fixes the platforms, their tiers, and what a boot looks like on each. The table facts are from document 01.5 and carry its verification marks.

## 6.1 Tiers

| Tier | Platforms | Ladder | Search |
|---|---|---|---|
| 1 | x86_64, i386, arm64 | full, L0 to L8 | every kernel, frontier search, dense stripes |
| 2 | arm, riscv64, ppc64le, s390x, loongarch64 | full, L0 to L8 where the kernel has the tests | every kernel that has the architecture, frontier search |
| 3 | mips (malta), ppc (e500), ppc64 big endian (pseries), sparc64, m68k, alpha, parisc, sh4, openrisc, xtensa, microblaze | L0 to L6 | Current set and the longterm stripe, GCC 16.2 and the era GCC only |
| 4 | arc, csky, hexagon, nios2, and every removed architecture (ia64, avr32, blackfin, cris, frv, m32r, metag, mn10300, score, tile, h8300, unicore32, nds32) | L0 to L4, build only | on demand |

Tier 1 is where rucc's targets are: x86-64 now, i386 because x86-64 needs it and the museum is i386, and arm64 because rucc has an arm64 back end. Tier 2 is where rucc goes next or where the kernel is widely built. Tier 3 and 4 are there because the question is about GCC and the kernel, not only about rucc, and because a GCC change that breaks one of them often breaks a tier 1 platform a release later.

## 6.2 Tier 1 and tier 2 in detail

| Platform | kbuild `ARCH` | First kernel | First GCC | QEMU | Machine | CPU | Console | Image | Config |
|---|---|---|---|---|---|---|---|---|---|
| i386 | `i386`, `x86` from 2.6.24 | 0.01 | any | `qemu-system-i386` | `isapc` to 2.0, `pc` after | `486` to 2.2, `pentium` to 2.6.23, `pentium3` after (6.4) | VGA capture to 2.1.24, `ttyS0` from 2.1.25 | `Image`/`zImage`, `bzImage` from 1.3.73 | `i386_defconfig` from 2.6.24, `defconfig` or the era's default before |
| x86_64 | `x86_64`, `x86` from 2.6.24 | 2.4.20 and 2.5.5; graded from 2.6.0 | 3.1 | `qemu-system-x86_64` | `pc` to 2.6.x, `q35` from 3.0 | `qemu64` to 3.x, `max` with `la57=off` after (6.4) | `ttyS0` | `bzImage` | `x86_64_defconfig` |
| arm64 | `arm64` | 3.7 | 4.8 | `qemu-system-aarch64` | `virt,gic-version=max` | `cortex-a57` below 4.16, `max,lpa2=off` below 5.12, `max,pauth-impdef=on` after | `ttyAMA0` | `Image` | `defconfig` |
| arm | `arm` | 2.1.x; graded from 3.2 on `virt` | 2.95 (in the tree); columns from 4.8.5 on kernel.org | `qemu-system-arm` | `vexpress-a9` below 3.14, `virt` after (unverified boundary) | `cortex-a9` or `cortex-a15` | `ttyAMA0` | `zImage` | `vexpress_defconfig` then `multi_v7_defconfig` |
| riscv64 | `riscv` | 4.15; boots `virt` to user space from 4.19 | 7.1 | `qemu-system-riscv64` | `virt` with OpenSBI from QEMU | `rv64` | `ttyS0` | `Image` | `defconfig` |
| ppc64le | `powerpc` | 3.13 (little endian) | 4.8 | `qemu-system-ppc64` | `pseries` for older kernels, `powernv` for newer (6.4) | `power8`, `power9`, `power10` | `hvc0` | `vmlinux`, `zImage.epapr` | `pseries_le_defconfig`, `powernv_defconfig` |
| s390x | `s390` | 2.4 (as `s390x`) | any in the tree | `qemu-system-s390x` | `s390-ccw-virtio` | `qemu` | `ttysclp0` | `bzImage` | `defconfig` |
| loongarch64 | `loongarch` | 5.19; boots from about 6.1 (unverified) | 12.1; 13 with binutils 2.40 for explicit relocations | `qemu-system-loongarch64` | `virt` with EFI firmware | `la464` | `ttyS0` | `vmlinuz.efi` | `loongson3_defconfig` |

"First GCC" is the first GCC with a back end that can build that architecture's kernel. Cells with an older GCC are **n/a**, not **fails**. The kernel's own minimums (4.8 for arm64 before its 5.1 floor, 12 for parisc64 from `min-tool-version.sh`) are **fails** at L1, because the kernel refuses them.

## 6.3 Cross builds everywhere

Every build is a cross build from an x86_64 machine through `CROSS_COMPILE`, including x86_64 itself (5.3). The kernel does not run target programs while it builds, so a cross build is its natural build. The one place a native build is compared is the `ubuntu-24.04-arm` runner check of 09.8, which rebuilds a sample of arm64 cells natively and compares `vmlinux` bytes. A difference there is a finding about the toolchain, not about the kernel.

## 6.4 Machine and CPU models change with the kernel's age

Old kernels do not boot on the newest emulated hardware, and new kernels need features old models lack. The platform's machine and CPU are chosen per kernel range in `platforms.toml`, and the choice is part of the cell's identity. Rules:

1. **The model is chosen for the kernel, never for the GCC.** A whole row boots on the same machine. That keeps the row a toolchain comparison.
2. **Prefer the oldest model that the newest kernel of the range boots on.** A model that is too new shows up as a hang in `calibrate_delay`, a TSC failure, or an unsupported feature trap. Those are kernel and emulator problems, and they must not look like a compiler result.
3. **A model boundary moves only with evidence.** When the era GCC's cell fails L5 and the L5k twin passes, the boundary is wrong. When the reference boots and a column does not, the model is not the explanation.

Known constraints:
- x86 kernels before the late 2007 TSC synchronisation rework hang under KVM with more than one CPU. Everything up to 2.6.24 boots with `-smp 1` under TCG.
- `-cpu max` on x86 under TCG hung with 5 level paging on QEMU 6.1. `la57=off` is set on every x86 CPU model.
- `-no-hpet` and `-no-acpi` are gone from QEMU 9.0. The spelling is `-machine hpet=off,acpi=off`, used for 2.4 and older.
- arm64 kernels before 4.16 need `cortex-a57` or `cortex-a72`. Before 5.12, `max` must have `lpa2=off`. From QEMU 6.0, pointer authentication with `pauth-impdef=on` keeps TCG fast.
- ppc64le kernels before about 4.x boot on `pseries` with `power8`, not `powernv` (unverified boundary).

## 6.5 QEMU

The default QEMU is the one in the `gk-boot` container, pinned by digest in `hosts.toml`. At G1 that is Debian trixie's QEMU 10.0.13 from the same snapshot as the hosts, not a build from source as first planned: the container digest already pins every byte, and building QEMU in the forge can wait for the day a target needs a QEMU Debian does not ship. Two more are pinned for what the newest one cannot do:
- **QEMU 9.0.x** for nios2, whose emulation was removed in QEMU 9.1. That is tier 4, so it is only used on demand.
- **An older QEMU**, chosen at G3, if a museum kernel boots on it and not on the default. Guenter Roeck's kerneltests.org keeps several QEMU versions for the same reason.

A new default QEMU pin re-runs the dense stripes' boots (09.7). Every row whose boot result changes is investigated before the pin is accepted, because a QEMU change that turns cells red would otherwise be recorded as a toolchain result.

## 6.6 KVM and TCG

TCG is the reference for every platform. KVM is used for x86_64 and i386 guests on the x86_64 machines, from 2.6.25 on, where the budget matters most, and only after a check at G1: the era diagonal's x86 boots are run under both and must give the same verdict. Kernels before 2.6.25 always use TCG. KVM exposes the host CPU's quirks to kernels that predate them, and the saving is not worth a doubtful result.

## 6.7 Budgets

Each platform has a build budget and a boot budget in `platforms.toml`. The boot budget is three times the median boot time of the era GCC's cell on that platform and kernel range, plus 60 seconds, measured at G1 and G3. The boot that runs every KUnit suite before init has its own budget, set the same way, since the whole set took 328 seconds on 7.2.8 x86_64 under TCG, which is more than the plain boot budget. A cell over budget has failed that rung. There is no retry beyond the three runs of 02.4, and the budget is never raised for one column. A build-only configuration such as `allmodconfig` gets ten times the build budget, since it compiles about ten times as much.
