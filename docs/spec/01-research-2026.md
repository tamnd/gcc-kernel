# Research, October 2026

This document collects the facts the rest of the spec rests on: GCC's release history, the kernel's minimum GCC over time, the breakages each GCC release caused and the kernel commits that fixed them, what is known about building the oldest kernels, the Debian releases that serve as hosts, and the prior art. It was gathered on 2026-10-01 from gcc.gnu.org, the GitHub mirror of Linus's tree, the `gregkh/linux` stable branch heads, LKML archives, kernel.org, Debian's archive, and the projects named in 01.7. git.kernel.org and lore.kernel.org were not reachable during the research, so commit tags were checked on GitHub.

The document has nine sections: GCC releases (1.1), minimums (1.2), blacklists (1.3), breakages (1.4), architectures (1.5), early kernels (1.6), Debian hosts (1.7), prior art (1.8) and binutils (1.9).

Marks: a fact without a mark was checked against a primary source. **(unverified)** means it comes from one secondary source or from memory, and G0 or the stage that needs it checks it. "First tag" is the first mainline tag that contains the commit. Stable presence was checked by reading the file at the head of each `linux-X.Y.y` branch, which says whether a fix is there today, not in which point release it arrived.

## 1.1 GCC release history

From gcc.gnu.org/releases.html. The table lists the series, their first and last releases, and what matters about them for the kernel. Every point release is in `gccs.toml` with its date.

| Series | First | Last | Last date | Notes for the kernel |
|---|---|---|---|---|
| 1.x | 0.9, 1987-03-22 | 1.42 | 1992-09-20 | 1.40 (1991-06-01) built 0.01 to 0.12. 1.37.1 was the Minix-386 port's base |
| 2.0 to 2.3 | 2.0, 1992-02-22 | 2.3.3 | 1992-12-26 | ran in parallel with 1.41 and 1.42. 2.3.3 is 0.99.10's minimum |
| 2.4 | 2.4.0, 1993-05-17 | 2.4.5 | 1993-06-20 | 1.0's minimum |
| 2.5 | 2.5.0, 1993-10-22 | 2.5.8 | 1994-01-24 | 1.1 and 1.2's minimum |
| 2.6 | 2.6.0, 1994-07-14 | 2.6.3 | 1994-11-30 | 2.6.0 is named in 1.2's documentation as a version to avoid **(unverified wording)** |
| 2.7 | 2.7.0, 1995-06-16 | 2.7.2.3 | 1997-08-22 | the compiler of 1.3, 2.0 and 2.2 |
| 2.8 | 2.8.0, 1998-01-07 | 2.8.1 | 1998-03-02 | FSF line during EGCS, little used for kernels |
| EGCS | 1.0, 1997-12-03 | 1.1.2 | 1999-03-15 | 1.1.2 (`2.91.66`) is the 2.2 alternative and the 2.4.0 minimum |
| 2.95 | 2.95, 1999-07-31 | 2.95.3 | 2001-03-16 | 2.4 and 2.6.0 to 2.6.15 minimum. Red Hat's unofficial 2.96 is refused with frame pointers |
| 3.0 | 3.0, 2001-06-18 | 3.0.4 | 2002-02-20 | refused from 2.6.29 |
| 3.1 | 3.1, 2002-05-15 | 3.1.1 | 2002-07-25 | refused from 2.6.29. x86_64's minimum |
| 3.2 | 3.2, 2002-08-14 | 3.2.3 | 2003-04-22 | minimum 2.6.16 to 4.18 |
| 3.3 | 3.3, 2003-05-13 | 3.3.6 | 2005-05-03 | arm's "known good" from 2.6.16 |
| 3.4 | 3.4.0, 2004-04-18 | 3.4.6 | 2006-03-06 | |
| 4.0 | 4.0.0, 2005-04-20 | 4.0.4 | 2007-01-31 | `compiler-gcc4.h` from January 2005 |
| 4.1 | 4.1.0, 2006-02-28 | 4.1.2 | 2007-02-13 | 4.1.0 and 4.1.1 refused from 2.6.29 |
| 4.2 | 4.2.0, 2007-05-13 | 4.2.4 | 2008-05-19 | oldest kernel.org crosstool binary |
| 4.3 | 4.3.0, 2008-03-05 | 4.3.6 | 2011-06-27 | |
| 4.4 | 4.4.0, 2009-04-21 | 4.4.7 | 2012-03-13 | |
| 4.5 | 4.5.0, 2010-04-14 | 4.5.4 | 2012-07-02 | first `asm goto` |
| 4.6 | 4.6.0, 2011-03-25 | 4.6.4 | 2013-04-12 | minimum 4.19 to 5.7. First broad kernel.org crosstool set (4.6.3) |
| 4.7 | 4.7.0, 2012-03-22 | 4.7.4 | 2014-06-12 | |
| 4.8 | 4.8.0, 2013-03-22 | 4.8.5 | 2015-06-23 | first arm64. 4.8.0 to 4.8.2 refused on arm. crosstool-ng's floor |
| 4.9 | 4.9.0, 2014-04-22 | 4.9.4 | 2016-08-03 | minimum 5.8 to 5.14. 4.9.0 and 4.9.1 miscompile `load_balance()` |
| 5 | 5.1, 2015-04-22 | 5.5 | 2017-10-10 | minimum 5.15 to 6.15 |
| 6 | 6.1, 2016-04-27 | 6.5 | 2018-10-26 | Debian and Ubuntu builds default to PIE |
| 7 | 7.1, 2017-05-02 | 7.5 | 2019-11-14 | first riscv |
| 8 | 8.1, 2018-05-02 | 8.5 | 2021-05-14 | minimum from 6.16 (x86 from 6.15) |
| 9 | 9.1, 2019-05-03 | 9.5 | 2022-05-27 | |
| 10 | 10.1, 2020-05-07 | 10.5 | 2023-07-07 | `-fno-common` default |
| 11 | 11.1, 2021-04-27 | 11.5 | 2024-07-19 | |
| 12 | 12.1, 2022-05-06 | 12.5 | 2025-07-11 | first loongarch. Closed |
| 13 | 13.1, 2023-04-26 | 13.5 | 2026-09-11 | closed |
| 14 | 14.1, 2024-05-07 | 14.4 | 2026-06-26 | maintained. Six diagnostics become errors |
| 15 | 15.1, 2025-04-25 | 15.3 | 2026-06-12 | maintained. `-std=gnu23` default |
| 16 | 16.1, 2026-04-30 | 16.2 | 2026-08-07 | maintained. rucc's target |

Trunk is GCC 17.0 and is not a column (04.2).

## 1.2 The kernel's minimum GCC, and how it is enforced

| Kernels | Minimum | Enforcement |
|---|---|---|
| 0.01 to 0.12 | gcc 1.40 with Linus's `-mstring-insns` patch | the Makefile's flags |
| 0.99.10 | 2.3.3 | documentation only |
| 0.99.15 | 2.4.5 | documentation only |
| 1.0 | 2.4.5 | documentation only |
| 1.1, 1.2 | 2.5.8, avoid 2.6.0 | documentation only |
| 1.3.x | 2.7.2, binutils 2.6.0.12 | documentation only |
| 2.0 | 2.7.2 **(unverified: no 2.0 `Documentation/Changes` was fetched)** | documentation only |
| 2.2 | 2.7.2.3 or EGCS 1.1.2 | documentation. The Makefile of late 2.2 prefers `kgcc` or `gcc272` from `PATH` |
| 2.4.0 | EGCS 1.1.2 (`2.91.66`) | `#error` below 2.91. The documented minimum moved to 2.95.3 during 2.4 **(which point release is unverified)** |
| 2.6.0 to 2.6.15 | 2.95.3 | `init/main.c`: `#error` "Sorry, your GCC is too old. It builds incorrect kernels." below 2.95, and an `#error` for 2.96 with `CONFIG_FRAME_POINTER` |
| 2.6.16 to 4.18 | 3.2 | a136564 "remove gcc-2 checks", v2.6.16-rc1. 3.0 and 3.1 refused from 2.6.29 by 6680598 (`compiler-gcc3.h`), and by `GCC_VERSION < 30200` in `compiler-gcc.h` from 4.2 |
| 4.19 to 5.7 | 4.6 | cafa0010cd51, `#if GCC_VERSION < 40600` |
| 5.8 to 5.14 | 4.9 | 5429ef62bcf3 (4.8, v5.8-rc1), then 6ec4476ac825 (4.9, v5.8-rc5, for `_Generic`) |
| 5.12 on | same | aec6c60a01d3 moves the check into Kconfig (`scripts/cc-version.sh`, "Sorry, this compiler is not supported."). `scripts/min-tool-version.sh` follows, dated 5.13 **(unverified)** |
| 5.15 to 6.15 | 5.1 | 76ae847497bc, v5.15-rc2 |
| 6.15, x86 | 8.1 | a3e8fe814ad1, v6.15-rc1 |
| 6.16 on | 8.1 on every architecture, binutils 2.30 | 118c40b7b503 "kbuild: require gcc-8 and binutils-2.30", v6.16-rc1 |

The maximum side. Before 4.2, the kernel needed a header per GCC major version. 2.6.12's `compiler.h` has `#if __GNUC__ > 4` → `#error no compiler-gcc.h file for this gcc version`. From 2.6.29 (f153b82 "Sanitize gcc version header includes"), `#include gcc_header(__GNUC__)` fails with a missing `compiler-gccN.h` instead. `compiler-gcc5.h` arrived in v3.18-rc1 (71458cfc782e) and was backported to 3.10 and 3.13. cb984d101b30 (v4.2-rc1) folded the headers into one and ended the problem. It was backported to 3.10 in 2016, and 4.1.x users were told to move to 4.1.43. No `compiler-gcc6.h` ever existed upstream. So a kernel between 2.6.29 and 4.1 has a hard maximum GCC major, and the matrix measures it as an L1 failure with the class `no-compiler-header`.

## 1.3 Blacklists and known miscompiles

| GCC | Where | Condition | Commit | First tag |
|---|---|---|---|---|
| below 2.95; 2.96 with frame pointers | `init/main.c` | `__GNUC__` and `__GNUC_MINOR__` | before git | 2.6.12 or earlier |
| 3.0, 3.1 | `compiler-gcc3.h` | `__GNUC_MINOR__ < 2` | 6680598 | v2.6.29-rc1 |
| arm, 3.x below 3.3 | `arch/arm/kernel/asm-offsets.c` | "Known good compilers: 3.3" | a136564 | v2.6.16-rc1 |
| 4.1.0, 4.1.1 | `compiler-gcc4.h` | "Your version of gcc miscompiles the __weak directive" (GCC PR 27781) | f9d1425 | v2.6.29-rc1 |
| up to 4.8.1, then all | `compiler-gcc4.h` | `asm goto` miscompile (GCC PR 58670): `asm_volatile_goto()` adds an empty `asm("")` | 3f0116c, made unconditional by a9f180345f53 | v3.12-rc5, v3.14-rc3 |
| arm, 4.8.0 to 4.8.2 | `arch/arm/kernel/asm-offsets.c` | "Your compiler is too buggy; it is known to miscompile kernels" (PR 58854, file system corruption) | 7fc150543c73 | v3.18-rc3, removed in 5.8 |
| 4.9.0, 4.9.1 | none | the scheduler's `load_balance()` spill (PR 61801). Not blacklisted: worked around with `-fno-var-tracking-assignments` | 2062afb4f804 | v3.16-rc7, and 3.13 stable |
| arm, 4.7 to 4.8 | `mm/migrate.c` | ICE avoided with `ICE_noinline` | removed by 6ec4476ac825 | 5.8 |

A common claim, that GCC 4.9.0 is blacklisted on x86_64, was not found and is treated as false. Nor was anything found for a GCC 4.5 `asm goto` bug: PR 58670 is a 4.8 bug.

For the matrix, the refusals are expected **fails** at L1 and are classified by the `blacklist` class. The workaround for 4.9.0 means a kernel before 3.16 built with 4.9.0 or 4.9.1 may build and then fail in the scheduler at run time, which is exactly the kind of hole that interior sampling (09.3) is for. The 4.9.0 and 4.9.1 point columns exist for that reason (04.2).

## 1.4 Breakages per GCC release

Each row is a failure mode seen in the wild, the kernel commit that fixed it, and where the fix is. These seed the signature catalog (08.4), which gives each a class name.

**GCC 4.0 to 6: the per-major header.** Covered in 1.2. GCC 6 on 3.x and 4.1 without the backport fails with `fatal error: linux/compiler-gcc6.h: No such file or directory`.

**GCC 6 with default PIE (Debian, Ubuntu).** `code model kernel does not support PIC mode`. Fixed by 8ae94224c9d7 "kbuild: add -fno-PIE" and c6a385539175 "kbuild: Steal gcc's pie from the very beginning", both v4.9-rc6. Backported to 4.8, 3.12 and 3.16.40. Upstream GCC 6 does not default to PIE, so this is a distribution flavor cell (04.3).

**GCC 7.** About 1500 lines of new warnings: `-Wformat-truncation`, `-Wformat-overflow`, `-Wint-in-bool-context`. bd664f6b3e37 "disable new gcc-7.1.1 warnings for now", v4.13-rc1, backported to 4.4, 4.9 and 3.16.63. Only fatal where a directory builds with `-Werror`.

**GCC 8.**
- `-Wpacked-not-aligned`: 321cb0308a9e, v4.16-rc1.
- `-Wattribute-alias` from `SYSCALL_DEFINEx`: bee20031772a, v4.18-rc3.
- `-Wstringop-truncation`: 217c3e019675, v4.19-rc2, backported to 4.9.
- objtool on `.cold` subfunctions: 13810435b9a7, v4.17-rc6, backported to 4.14. Without it, objtool reports false warnings, and they are fatal where objtool is.
- objtool on GCC 8 switch tables ("stack state mismatch"): fd35c88b7441, v4.17-rc6, backported to 4.14.

**GCC 9.**
- `-Wmissing-attributes` on `module_init` and `module_exit` aliases: c0d9782f5b6d (`__copy`) and a6e60d84989f, v5.0-rc8, backported to 4.19 and 4.9.
- `-Waddress-of-packed-member`: 6f303d60534c, v5.1.
- `asm inline` support: eb111869301e, v5.4-rc1. A feature, not a breakage, but a persona gate (11.3).

**GCC 10.**
- `-fno-common` default. The dtc `yylloc` link error, fixed by e33a814e772c, v5.6, backported to 4.4 and later. This is a host tool failure when `HOSTCC` is GCC 10 (05.1), and a target failure nowhere, because the kernel is built with `-fno-common` anyway. Also cpupower (2de7fb60a474), perf (e4d9b04b973b, 168200b6d6ea), bpf and KVM selftests, usbip.
- Boot panic "stack-protector: Kernel stack is corrupted in: start_secondary". GCC 10 tail-calls `cpu_startup_entry()` after the canary is set. a9a3ed1eff36 "x86: Fix early boot crash on gcc-10, third try", v5.7, backported to 4.4 to 5.4. This is the canonical L5 failure: a kernel that builds and does not boot.
- `-Wzero-length-bounds` (5c45de21a222), `-Wstringop-overflow` (5a76021c2eff), `-Warray-bounds`, all disabled in v5.7 and backported widely.
- gcov: 40249c696207, v5.9.

**GCC 11.**
- `-Wstringop-overread`, `-Warray-parameter`: e7c6e405e171, e14cfb3bdd0f, cdc34cb8f25d, v5.13.
- `-Wmisleading-indentation`: 40cc3a80bb42, v5.13.
- plugins: 67a5a6801305, v5.11 (irrelevant with plugins off).
- `-Warray-bounds` disabled for 11: 5a41237ad1d4, v6.2. `-Wstringop-overflow` disabled for all GCC: 021533194476, v6.8.

**GCC 12.**
- `-Warray-bounds` everywhere: f0be87c42cbd, v5.19, introducing `CC_NO_ARRAY_BOUNDS`, backported to 5.15 as an unconditional `-Wno-array-bounds`.
- `-Wdangling-pointer`: 49beadbd47c2, v5.19, backported to 4.14 and later.
- x86 boot literal addresses: aeb84412037b, v5.19.
- s390 `-Warray-bounds`: 8b202ee21839, v5.18, backported to 5.4 and 5.10.
- `-Wuse-after-free` in `tools/lib/subcmd`: a host tool failure (05.1). No kernel-wide disable was found.

**GCC 13.**
- C2x enum typing: an enum with negative values and values above `LONG_MAX` becomes 64-bit on 32-bit architectures. 525ff9c29657 "workqueue: fix enum type for gcc-13", v6.5.
- `-Wenum-int-mismatch` in drivers: 545094d993f4 and others, v6.2 onward.
- `-Warray-bounds` for 13: 0da6e5fd6c37, v6.3.

**GCC 14.** Six diagnostics become errors by default: `implicit-int`, `implicit-function-declaration`, `declaration-missing-parameter-type`, `return-mismatch`, `int-conversion`, `incompatible-pointer-types`.
- The kernel proper already turned most of these into errors itself (4.4 has `-Werror-implicit-function-declaration` and `-Werror=implicit-int`; 4.19 adds `-Werror=incompatible-pointer-types`), so kernel C mostly does not notice.
- The Kconfig menu's `scripts/kconfig/lxdialog/check-lxdialog.sh` has `main() {}` on 4.4, 4.9 and 4.14. With GCC 14 as `HOSTCC`, `make menuconfig` reports missing ncurses. No upstream fix: the script was replaced before 4.19. It does not affect `defconfig`, and the matrix's hosts use their own GCC, so this does not reach the matrix. It is a native mode finding (05.5).
- `-Wcalloc-transposed-args` and `-Walloc-size` in objtool (e2e13630f93d) and perf (7bbe8f0071df), v6.8. objtool is a host tool built with `-Werror`, so this only matters in native mode.
- objtool "falls through to next function" and missing `ENDBR` warnings with 14.2 (GCC PR 116174), not worked around in the kernel. Reported fixed in 14.3 **(unverified)**. This is an L4o twin case on x86 kernels with `CONFIG_OBJTOOL_WERROR`, and a reason for the 14.2 point column.

**GCC 15.** `-std=gnu23` becomes the default, making `bool`, `true` and `false` keywords, and `{0}` no longer zeroes a whole union.
- The kernel proper always passes `-std`: gnu89 from 51b97e354ba9, gnu11 from v5.18 (e8c07082a810). Only Makefiles that reset `KBUILD_CFLAGS` break: boot code, EFI stub, decompressors, vDSOs.
- x86 boot: b3bee1e7c3f2 (v6.7) and ee2ab467bddf (v6.14, `Cc: stable`, in 6.1.y as 0f82f6f15563). EFI stub: 8ba14d9f490a, v6.14. s390: 3b8b80e99376, v6.14. loongarch: 947d5d036c78, v6.13. powerpc boot: 5a821e2d69e2, v6.16 (on 6.6 and 6.12, not 6.1). parisc: 7cbb015e2d3d, v6.16. mips vDSO: 0f4ae7c6ecb8, v6.16.
- Union zeroing: dce4aab8441d "kbuild: Use -fzero-init-padding-bits=all", v6.14, moved to the top-level Makefile in v6.15. On 6.12 and later stable, not on 6.6 or 6.1. A silent change in generated code where a kernel relied on `{0}` clearing padding. A candidate L5 to L7 failure on 6.6 and 6.1 with GCC 15 or 16, which the matrix tests.
- `-Wunterminated-string-initialization`: d5d45a7f2619, be913e7c4034, 9d7a0577c9db, 4f79eaa2ceac, all v6.15. On 6.12 and later stable, not on 6.6 or 6.1.
- 5.10.y: a backport series to build with GCC 15 was posted in October 2025 and declined because of the branch's end of life. The branch head nonetheless has explicit `-std=gnu89` in the x86 boot Makefiles **(how it got there is unverified)**. The matrix settles whether 5.10.last builds with GCC 15.

**GCC 16.** The only C porting change is `-Wunused-but-set-variable` and `-Wunused-but-set-parameter` at level 3 by default. C++ defaults to gnu++20, which matters for plugins.
- Plugins: `CONST_CAST_TREE` removed. 905c559e5149, v7.1, on 6.18.y and 7.0.y. 6.12.y and older still fail with plugins on. Also a40282dd3c48, v6.18. Off in the matrix (05.6), so this is recorded as a known failure outside the matrix.
- modpost section mismatches from IPA inlining `__init` callees: 4c9ad387aa2d "iommu, debugobjects: avoid gcc-16.1 section mismatch warnings", v7.1. An L4 failure where section mismatches are fatal. The first real GCC 16 kernel failure class, and rucc must match it under the GCC 16 persona (11.1).
- `-Wunused-but-set-*=3` in bpf selftests and arm64 hugetlbpage (97fb54d86d21, 729a2e8e9ac4), `-Wnonnull` in vsock (e25dbf561e03), v7.0. `-Wuninitialized` in landlock (c4e941bb7654), only in v7.3-rc5 so far.
- loongarch objtool "unreachable instruction" warnings with 16.1, attributed to 06e24745985c **(unverified whether fixed)**.

## 1.5 Architectures

| Architecture | Kernel | GCC | Notes |
|---|---|---|---|
| i386 | 0.01 | 1.40 | `arch/i386` until 2.6.24, `arch/x86` after |
| x86_64 | 2.4.20 and 2.5.5 | 3.1 | merged into `arch/x86` in 2.6.24 |
| arm | 1.3.x era **(unverified)** | 2.95 in tree | kernel.org crosstool has arm from 4.8.5 |
| arm64 | 3.7 | 4.8 | QEMU `-cpu` quirks: `cortex-a72` or `a57` before 4.16, `max,lpa2=off` before 5.12 |
| riscv | 4.15 | 7.1 | first to boot `virt` to user space: 4.19 |
| loongarch | 5.19 | 12.1 | explicit relocations need GCC 13 and binutils 2.40 |
| parisc64 | old | 12 | the kernel requires GCC 12 for 64-bit parisc via `min-tool-version.sh` |
| microblaze | 2.6.30 | | console `ttyUL0` |
| nios2 | 3.19 **(unverified)** | | gone from GCC 15.1. QEMU emulation removed after 9.0 |
| ia64 | 2.4 era | | removed from the kernel in 6.7 and from GCC 15.1 |
| arc, csky, hexagon | | arc from GCC 8.1, csky from 9.2 on kernel.org | no upstream QEMU system emulator |
| s390x | 2.4 era | | console `ttysclp0`. kernel.org has `s390x` as a separate target only from 16.2 |

Consoles: s390 `ttysclp0`, sh4 on `r2d` `ttySC1`, microblaze `ttyUL0`, ppc64 on `pseries` `hvc0`, arm and arm64 `ttyAMA0`, x86 and riscv `ttyS0`.

## 1.6 Early kernels and their toolchains

**0.01** (17 September 1991). Built on Minix-386 with GCC 1.40 carrying Linus's private patch for `-mstring-insns`, with the Minix `as -0 -a` and `ld -0` for the boot sector, and gas and gld for the kernel, producing a.out. The root is on `hd6` (0x306, the first partition of the second disk). The disk geometry is compiled in (`LINUS_HD` with 5 heads, 17 sectors, 980 cylinders). `HIGH_MEMORY` is 8 MB. There is no way to pass a kernel to a loader: it boots from a floppy image. The tarball's SHA-256 is `24454f830cdb571e2c4ad15481119c43b3cafd48dd869a9b2945d1036d1dc68d`.

**0.02 and 0.03** do not survive. **0.10** as distributed is a reconstruction by Theodore Ts'o. **0.11 and 0.12** use `as86` and `ld86` and GCC 1.40 with `-fcombine-regs -mstring-insns`.

**0.95 to 0.99.** GCC 2.x arrives: 0.99.10 wants 2.3.3, 0.99.15 wants 2.4.5. The command line (the `0xA33F` magic) arrives in 0.99.10, which is also where QEMU's `-kernel` can boot a zImage, without initrd.

**1.0** (13 March 1994) wants GCC 2.4.5 or newer. **1.1 and 1.2** want 2.5.8 and warn against 2.6.0. They link a.out with `-qmagic`. 1.2.13 has no ELF option.

**1.3.** `CONFIG_KERNEL_ELF` first appears in 1.3.5. 1.3.x wants GCC 2.7.2 and binutils 2.6.0.12. Boot protocol 2.00 with bzImage and initrd arrives in 1.3.73. ELF only from 2.1.0.

**2.0** wants 2.7.2. 2.0.40 wants 2.7.2.1 and binutils 2.8.1.0.23, and says GCC 3.x is unlikely to work.

**2.1 and 2.2.** The serial console arrives in 2.1.25. 2.2 wants 2.7.2.3 or EGCS 1.1.2. 2.2.26's Makefile picks `gcc272` or `kgcc` from `PATH` automatically. bin86 is needed up to 2.3.20.

**2.4.** 2.4.0 needs EGCS 1.1.2 and `#error`s below 2.91. 2.4.37.11 wants 2.95.3, works with 3.3, 3.4, 4.0 and 4.1, and says 4.2 and later are unsupported.

**2.5 and 2.6.0.** initramfs arrives in 2.5.46. Floppy boot is removed in 2.5.65. 2.6.0 wants GCC 2.95.3, binutils 2.12 and make 3.78.

**Host facts.** GNU make 3.82 and later break kernels before 2.6.36. The binutils `i386linux` a.out back end was removed around 2.31. Linux dropped a.out on x86 in 5.19. GCC 2.95 does not build hosted on x86_64, so it is built in i386 containers (04.4).

**Sources.**
- kernel.org Historic, with `.tar.sign` files and `sha256sums.asc`.
- `history.git`, whose root commit is bb441db, and mpe's `linux-fullhistory`, which joins it to modern git.
- oldlinux-web and oldlinux-files on GitHub, with GCC 1.21 to 1.42, gas, binutils, `gccbin` (binary GCC for Linux 0.1x), as86, and root and boot images.

## 1.7 Debian releases as host environments

From the archive's `Packages` indexes. These set the host table of 05.2.

| Release | Date | Default gcc | binutils | Also shipped |
|---|---|---|---|---|
| buzz 1.1 | 1996-06-17 | 2.7.2 | 2.6 | `aout-gcc` 2.6.3 |
| rex 1.2 | 1996-12-12 | 2.7.2.1 | | |
| bo 1.3 | 1997-06-05 | 2.7.2.1 | 2.7.0.9 | `aout-gcc` 2.7.2.1, `aout-binutils` 2.7 |
| hamm 2.0 | 1998-07-24 | 2.7.2.3 | 2.9.1 | EGCS 1.0.3 |
| slink 2.1 | 1999-03-09 | 2.7.2.3 | 2.9.1.0.19a | EGCS 1.1.2 |
| potato 2.2 | 2000-08-15 | 2.95.2 | 2.9.5.0.37 | `gcc272` |
| woody 3.0 | 2002-07-19 | 2.95.4 | 2.12.90.0.1 | `gcc-3.0` |
| sarge 3.1 | 2005-06-06 | 3.3.5 | 2.15 | 2.95.4, 3.4.3 |
| etch 4.0 | 2007-04-08 | 4.1.1 | 2.17 | `gcc272`, 2.95.4, 3.3.6, 3.4.6 |
| lenny 5.0 | 2009-02 | 4.3.2 | 2.18.1 | |
| squeeze 6.0 | 2011-02 | 4.4.5 | 2.20.1 | |

The later releases (wheezy 4.7.2 through trixie 14.2.0) are in 05.2. The `debian/eol` images cover buzz to hamm for `386` only, add `amd64` at slink, and have `386`, `amd64` and `armv5` from potato to etch.

## 1.8 Prior art

| Project | What it does | What gcc-kernel takes |
|---|---|---|
| kernel.org crosstool (`mirrors.edge.kernel.org/pub/tools/crosstool/`) | Arnd Bergmann's `nolibc` cross compilers, signed, with `sha256sums.asc`. Main set: 4.9.4, 5.5.0, 6.5.0, 7.5.0, 8.5.0, 9.5.0, 10.5.0, 11.5.0, 12.5.0, 13.5.0, 14.4.0, 15.3.0, 16.2.0. One-offs back to 4.2.4. First broad set 4.6.3, arm and arm64 from 4.8.5, riscv from 7.3.0, loongarch64 from 12.1.0, arc from 8.1.0, csky from 9.2.0. ia64 and nios2 gone from 15.1.0. `s390x` as its own target only in 16.2.0 | a cross-check of the forge (04.4), and a source of binaries where they run in the host (05.3). Path: `bin/<host>/<ver>/<host>-gcc-<ver>-nolibc-<target>.tar.xz` |
| TuxMake (Linaro, MIT) | reproducible kernel builds in containers `tuxmake/<arch>_gcc-N` for gcc-8 to gcc-16, and `korg-gcc` | the container discipline and the build log conventions. Its floor is GCC 8, so it covers only part of the matrix |
| TuxRun | boots kernels on QEMU devices with a rootfs | per-device QEMU arguments to compare with 06.2 |
| KernelCI | builds and boots with a few pinned toolchains (gcc-12, clang-15 and 17) | not a matrix. A source of boot regressions on the Current set |
| 0-day and `make.cross` | Intel's per-commit build bot with kernel.org crosstool | its `make.cross` wrapper's per-arch compiler choice |
| ClangBuiltLinux `continuous-integration2` and `boot-utils` | clang matrix over trees and architectures; per-arch QEMU boot scripts | the `-cpu` quirks of 06.4 and the initramfs approach |
| Guenter Roeck's `linux-build-test` | builds and boots every stable branch on many architectures, per-arch rootfs scripts, several QEMU versions including patched ones | the tier 3 machines and consoles, and the idea of pinning more than one QEMU (06.5) |
| crosstool-ng | builds cross toolchains. Floor GCC 4.8.5 since 1.23 | the forge's modern chain could use it. The legacy chain cannot |
| `a13xp0p0v/kernel-build-containers` (GPL-3.0) | containers for GCC 4.9 to 16 and clang | a cross-check of which GCC builds which kernel on x86_64 |
| nixpkgs | old GCCs as derivations | a source of build recipes for 4.x toolchains **(unverified how far back they build today)** |

None of these answers the question of document 02. Each fixes a small set of compilers and follows the current kernels, or builds compilers without the kernel. gcc-kernel's matrix is the product none of them publishes: every GCC release against every kernel release, with the edges explained.

## 1.9 binutils

Release dates from ftp.gnu.org file dates. 2.16 to 2.21 are approximate, because those tarballs were uploaded again later.

| binutils | Date | binutils | Date |
|---|---|---|---|
| 2.9.1 | 1998-05-01 | 2.33.1 | 2019-10-12 |
| 2.10 | 2000-06-19 | 2.34 | 2020-02-01 |
| 2.12 | 2002-03-08 | 2.35 | 2020-07-24 |
| 2.14 | 2003-06-12 | 2.36 | 2021-01-24 |
| 2.16 to 2.21 | 2005 to 2010 **(unverified)** | 2.37 | 2021-07-18 |
| 2.22 | 2011-11-21 | 2.38 | 2022-02-09 |
| 2.23 | 2012-10-22 | 2.39 | 2022-08-05 |
| 2.24 | 2013-12-02 | 2.40 | 2023-01-14 |
| 2.25 | 2014-12-23 | 2.41 | 2023-07-30 |
| 2.26 | 2016-01-25 | 2.42 | 2024-01-29 |
| 2.27 | 2016-08-03 | 2.43 | 2024-08-04 |
| 2.28 | 2017-03-02 | 2.44 | 2025-02-02 |
| 2.29 | 2017-07-24 | 2.45 | 2025-07-27 |
| 2.30 | 2018-01-28 | 2.46.0 | 2026-02-10 |
| 2.31.1 | 2018-07-18 | 2.47 | 2026-07-26 |
| 2.32 | 2019-02-02 | | |

**The kernel's minimum binutils:** 2.21 from 5.3 (1fb12b35e5ff), 2.23 from 5.7 (0aa78b105f57), 2.25 from 6.2 (e4412739472b), 2.30 from 6.16 (118c40b7b503). Enforced in Kconfig through `min-tool-version.sh` since 2021 (02aff8592204, ba64beb17493). The older documented minimums, 2.12 for 2.6.0 and 2.20 later, are **(unverified)**.

**Breakages.**

| binutils | Platform | Breakage | Fix | First tag | Stable |
|---|---|---|---|---|---|
| 2.16 | x86 | `movl %ds,(mem)`: "suffix or operands invalid for `mov'" | fd51f666fa59 | v2.6.12-rc4 | a 2.4 patch on LKML |
| 2.24 | ppc64le | weak symbols miscompiled | 60e065f70bdb refuses 2.24 | v4.2 | |
| 2.26 | i386 | `R_386_GOT32X`: the compressed kernel fails to boot, "Failed to allocate space for phdrs" | 6d92bc9d483a (compressed kernel built as PIE) | v4.6 | 3.10.105, 3.12.66, 4.4.26 |
| 2.26 and later | x86 | GOTPCRELX relaxation in the decompressor | 09e43968db40, 5214028dd89e, cb7927fda002 | v5.9, v5.8, v6.15 | |
| 2.29 | arm Thumb-2 | ADR low bit change oopses on syscall return | afc9f65e01cd | v4.18 | |
| 2.31 | x86_64 | `R_X86_64_PLT32` rejected by the module loader and relocs tool | b21ebf2fb4cd | v4.16 | 3.18.100, 4.4.122, 4.9.88, 4.14.28 |
| 2.31 | x86_64 | `-z separate-code` and 4 KB max page size break 2 MB alignment | e3d03598e8ae | v4.16 | 4.4.125, 4.9.91, 4.14.31 |
| 2.35 | riscv | vDSO symbol generation | c2c81bb2f691 | v5.10 | |
| 2.36, 2.37 | x86 | stripped section symbols confuse objtool | 24ff65257375 | v5.15 | |
| 2.37 | ppc | recordmcount failure | 25ea739ea1d4 | v6.5 | |
| 2.38 | ppc | "unrecognized opcode: stbcix" | 8667d0d64dd1 | v5.18 | |
| 2.38 | riscv | Zicsr and Zifencei split: "unrecognized opcode `csrr`" | 6df2a016c0c8, then e89c2e815e76, ca09f772ccca, ef21fa7c198e | v5.17 | 4.19.262, 5.4.180, 5.10.101, 5.15.24 |
| 2.39 | all | "missing .note.GNU-stack" and "LOAD segment with RWX permissions" warnings | 0d362be5b142, ffcf9c5700e4 (x86 vDSO and boot), bd71558d585a (UML) | v6.0 | 4.14.291, 4.19.256, 5.4.211, 5.10.137, 5.15.61 |
| 2.40 | x86 | real mode "found `movsd'; assuming `movsl'" | 7c6dd961d0c8 | v6.2 | |
| 2.41 | loongarch | relaxation on by default: module load fails, "Unknown relocation type 102" | 03c53eb90c0c | v6.5 | |
| 2.42 | loongarch | objtool "unreachable instruction" | e91c5e4c21b0 | v6.9 | |
| 2.46 | ppc | `-z text` default: "read-only segment has dynamic relocations" | 97f902dd4c99 | v7.2 | |

From 6.18, ec4a3992bc0b makes linker and assembler warnings respect `CONFIG_WERROR`, so a new binutils warning fails a `CONFIG_WERROR` build. 16-bit boot code is built with `-m16` from 3.15 (de3accdaec88), and the `.code16gcc` fallback was removed in 5.11 (2838307b019d). Not confirmed: a binutils 2.21 break on 2.6.x, a `.code16gcc` break caused by binutils, and any x86 break in 2.29 or 2.30.

**Distribution pairings.** Debian: bullseye 2.35.2 with GCC 10, bookworm 2.40 with 12, trixie 2.44 with 14, sid 2.47 with 16.1. Ubuntu: 14.04 2.24, 16.04 2.26.1, 18.04 2.30, 20.04 2.34, 22.04 2.38, 24.04 2.42, 26.04 2.46.
