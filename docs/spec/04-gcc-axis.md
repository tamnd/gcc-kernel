# The GCC axis

The columns of the matrix are toolchains: a GCC and the binutils it is paired with. This document fixes which GCC releases are columns, the distribution flavors, how every toolchain is built so that it runs in every host, how binutils is paired with GCC, and the binutils sweep that varies binutils alone.

## 4.1 What a column is

A column is one entry of `gccs.toml`. It names a GCC version, a flavor (upstream or a distribution package), the targets it is built for, and the binutils rule. For each platform it yields at most one toolchain bundle, identified by digest. The column's identity in a cell is the bundle digest, not the version string. Two bundles of the same version built differently are different columns, which can only happen through a change to `gccs.toml` that shows in review.

## 4.2 Upstream columns

**Last points.** The last release of every series, 36 columns:

1.42, 2.3.3, 2.4.5, 2.5.8, 2.6.3, 2.7.2.3, 2.8.1, EGCS 1.1.2, 2.95.3, 3.0.4, 3.1.1, 3.2.3, 3.3.6, 3.4.6, 4.0.4, 4.1.2, 4.2.4, 4.3.6, 4.4.7, 4.5.4, 4.6.4, 4.7.4, 4.8.5, 4.9.4, 5.5, 6.5, 7.5, 8.5, 9.5, 10.5, 11.5, 12.5, 13.5, 14.4, 15.3, 16.2.

1.42 is there for completeness. The 0.x kernels were built with 1.40, which is a point column below.

**Point columns.** About 20 more, each there for a reason written next to it in `gccs.toml`:

| Column | Why |
|---|---|
| 1.37.1, 1.40 | the Minix-386 base and the 0.01 to 0.12 compiler (07.7) |
| 2.7.2, 2.7.2.1 | named as minimums by 1.3 and 2.0 |
| 2.95.2 | Debian potato's compiler, and the first 2.95 in wide use |
| 3.0, 3.1 | refused from 2.6.29 (01.3): the matrix records the refusal's start |
| 3.2, 3.3 | first points of the 2.6.16 and arm minimums |
| 4.1.0, 4.1.1 | blacklisted from 2.6.29 for miscompiling `__weak` |
| 4.6.0 | first point of the 4.19 minimum |
| 4.8.0, 4.8.2 | the `asm goto` bug (PR 58670) and the arm blacklist (PR 58854) |
| 4.9.0, 4.9.1 | the `load_balance()` miscompile (PR 61801), not blacklisted |
| 5.1 | first point of the 5.15 minimum |
| 8.1 | first point of the 6.16 minimum |
| 10.1 | first `-fno-common` default, and the `start_secondary` boot panic |
| 14.2 | the objtool warnings of PR 116174 |
| 15.1 | first `-std=gnu23` default |
| 16.1 | first GCC 16, next to 16.2 so rucc's target has both points |

**First point columns.** The frontier search needs to know whether an edge falls at a major version or inside it. When a frontier edge falls between two last points, point-release refinement (09.3) builds the other releases of that series on demand. They become columns only when they turn out to decide an edge, and are recorded in `gccs.toml` with the reason `refinement`.

**Excluded.**
- Trunk (GCC 17.0 snapshots) is not a column. A snapshot is not a release, and it changes weekly. `gk column gcc-17-snapshot-<date>` can run one on demand into a separate report, which is how 10.8's new GCC report gets rehearsed before 17.1.
- Red Hat's 2.96, and vendor branches (Linaro, CodeSourcery, Apple). The kernel refuses 2.96 with frame pointers. The vendor branches are not GCC releases.
- Clang. It is a different question, and ClangBuiltLinux's CI answers it.

**Targets.** Each column is built for every platform whose first GCC (06.2) is at or below it, and whose back end the release still has. Removals are recorded in `gccs.toml` as `until`: ia64 and nios2 end before 15.1. A platform whose back end a column lacks is **n/a** for that column.

## 4.3 Distribution flavor columns

Distributions ship GCC with changed defaults, and several of those changes broke kernels. An upstream GCC of the same version would not show those breaks. Each distribution column is the distribution's own binary package, not a rebuild, run in that distribution's container. It carries the distribution's patches, which is the point. Examples:

| Column | Default it changed | Kernels it hits | Fix |
|---|---|---|---|
| Debian stretch `gcc-6`, Ubuntu 16.10 `gcc-6` | PIE | 4.8 and earlier without the fix | 8ae94224c9d7 and c6a385539175, v4.9-rc6, backported to 4.8, 3.12, 3.16.40 |
| Ubuntu 14.10 and later `gcc` | `-fstack-protector-strong` | kernels whose boot or vDSO code is not built with `-fno-stack-protector` **(which ones is what the column measures)** | per architecture |
| Ubuntu 19.10 and later `gcc` | `-fcf-protection` | x86 kernels before the flag was filtered **(unverified which)** | |
| Ubuntu 19.10 and later `gcc` | `-fstack-clash-protection` | **(unverified whether any kernel noticed)** | |
| Debian and Ubuntu, `--enable-default-pie` everywhere | PIE on every architecture | tier 2 kernels of the same period | |
| Fedora and RHEL `gcc` | `_FORTIFY_SOURCE` and annobin via spec files | only with the distribution's `redhat-rpm-config`, which a kernel build does not pick up. Not a column unless a finding shows otherwise | |

A distribution column is run only on the kernels around each change: the row range from two years before the fix to the fix, on x86_64 and one tier 2 platform. That is enough to reproduce the break and the fix. It is not searched over the whole axis.

Distribution columns cannot be static bundles: their binaries come from the distribution. They run only in their own distribution's host image, so they only get cells where the kernel's era host is that image or where the binary runs in the era host (checked by `gk forge verify --host`). Everything else is **not run** with that reason.

## 4.4 The forge

Every upstream column is built from its release tarball, hash-checked, unpatched, by `gk forge`. The product is a bundle: `bin/<triple>-{gcc,cpp,as,ld,ld.bfd,objcopy,objdump,nm,ar,ranlib,strip,readelf}`, `libexec/gcc/<triple>/<ver>/{cc1,collect2,lto1,lto-wrapper}`, `lib/gcc/<triple>/<ver>/` with `libgcc.a` and the target headers GCC installs, and a `bundle.json` with versions, configure lines, and hashes of every file. Bundles carry no C library and no C++ front end, because the kernel needs neither. That is the kernel.org crosstool recipe (`nolibc`), and the forge follows it wherever it can so the two can be compared.

**Static.** Every binary in a bundle is linked statically, on an i386 or x86_64 host, so it runs in any host container (05.3). GMP, MPFR, MPC and ISL are built in-tree with the versions the release's `contrib/download_prerequisites` names, or, before that script existed, the versions its installation notes name.

**Forge containers.** A GCC release builds with a host compiler and tools of its own time, and often fails to build with modern ones. The forge is therefore a chain of containers, each building the releases of its period, the same idea as the host environments but for building the toolchain:

| Forge | Base | Builds | Host compiler |
|---|---|---|---|
| `gk-forge-modern` | trixie, amd64 | 9.5 to 16.2 | GCC 14 |
| `gk-forge-bullseye` | bullseye, amd64 | 5.5 to 8.5 | GCC 10, with `-std=gnu++98` where the release needs it |
| `gk-forge-jessie` | jessie, amd64 | 4.6 to 4.9 | GCC 4.9 |
| `gk-forge-squeeze` | squeeze, amd64 and i386 | 4.2 to 4.5 | GCC 4.4 |
| `gk-forge-etch` | etch, i386 | 3.2 to 4.1 | GCC 3.4 and 4.1 |
| `gk-forge-woody` | woody, i386 | 2.95.3, EGCS 1.1.2, 3.0, 3.1 | GCC 2.95.4 |
| `gk-forge-slink` | slink or hamm, i386 | 2.7.2.x, 2.8.1 | GCC 2.7.2.3 |
| `gk-forge-bo` | bo, i386 | 2.3.3 to 2.6.3 | GCC 2.7.2.1 |
| `gk-forge-linux012` | the emulated Linux 0.12 image | 1.37.1 to 1.42 | GCC 1.40 of the image |

The table is the first assignment. A release that fails in its forge is tried in the neighbouring forges before it is recorded as **unbuildable**. The forge a release actually built in is recorded in `gccs.toml`.

**Cross builds of old GCC.** Before about GCC 3.x, building a cross compiler for a target other than the build machine was fragile, and before 2.95, `i386-linux` a.out and ELF targets were the only ones that matter. For the museum, the i386 GCC is built native in an i386 container (a native GCC is a cross compiler to its own target when put behind `CROSS_COMPILE`). Old cross compilers for arm and the rest are built only from 2.95.3 on, and only on demand for tier 3 and 4.

**Static link on old hosts.** Old glibc static linking works, with warnings for NSS functions that GCC does not call. Where a release's `collect2` or driver insists on dynamic features, the forge falls back to linking against the forge's glibc dynamically and ships the loader and libraries inside the bundle with an `rpath` of `$ORIGIN`, which runs in any host. `bundle.json` records which way was used.

**The cross-check against kernel.org.** For every release where kernel.org has a crosstool binary (01.8), `gk forge verify --against korg` builds `vmlinux` of the era diagonal's kernel with both the forged bundle and the kernel.org one, in a modern host where the kernel.org one runs, and compares `vmlinux` sections. They are not expected to be byte-identical (configure options, prerequisites versions and the build host differ). A verdict difference is a forge bug and blocks the column. The kernel.org bundles are also recorded as their own columns, `korg-<ver>`, on the Current set, which gives a second opinion at no forge cost.

**Reproducibility.** Forging the same release twice must give the same file hashes, apart from recorded exceptions. `SOURCE_DATE_EPOCH` is the release date, paths are normalized with `-ffile-prefix-map`, and the forge container is pinned by digest. A bundle whose rebuild differs is marked, and the difference investigated before it is used.

## 4.5 Pairing binutils with GCC

A column's GCC is paired with one binutils per platform. The rule is **the newest binutils release of the GCC's own time**:

1. take the GCC release date d;
2. pick the newest binutils release on or before d + 90 days;
3. if the target is not supported by that binutils, move forward until one supports it;
4. if the kernel's own minimum binutils (from `Documentation/Changes` or `min-tool-version.sh`) is newer, the cell uses the kernel's minimum instead, and records that it did.

Rule 4 keeps a new kernel with an old GCC from failing because of binutils, which the GCC axis is not about. The binutils sweep measures binutils separately.

Pairs are fixed in `binutils.toml` and listed in the published matrix. The known constraints the rule must satisfy:

| Constraint | Source |
|---|---|
| 6.16 and later require binutils 2.30 | 118c40b7b503 |
| loongarch explicit relocations need binutils 2.40 with GCC 13 | 01.5 |
| a.out output needs a binutils that still has the `i386linux` back end, before about 2.31 | 01.6 |
| older minimums: 2.21 from 5.3, 2.23 from 5.7, 2.25 from 6.2, enforced by `min-tool-version.sh` | 1fb12b35e5ff, 0aa78b105f57, e4412739472b |
| x86_64 kernels before 4.16 with binutils 2.31 or later fail (`R_X86_64_PLT32`) | b21ebf2fb4cd, so the time rule must not pick 2.31 for a 2017 GCC used on such a kernel: rule 2's 90 days keeps 7.3 on 2.30 |
| x86 kernels from about 4.19 test gas for `ADX`, `AVX512` and `SHA` instructions and drop crypto code when gas lacks them | the Kconfig `AS_*` symbols |

## 4.6 The binutils sweep

The binutils axis is swept separately, at G5, with GCC fixed at the kernel's era GCC. Columns are the last point of each binutils release from 2.6 to the newest. Rows are the longterm stripe on x86_64 and arm64. It answers the same questions as the GCC matrix for the assembler and linker: which binutils builds and boots which kernel, and what changes in the kernel's build with the binutils version (the `AS_*` and `LD_*` Kconfig symbols, `--build-id`, `-z noexecstack`, `--no-warn-rwx-segments`).

Known binutils breakages (01.9) seed the signature catalog under the classes `binutils-*` (08.4). The ones the sweep must reproduce, as a check that it works:
- binutils 2.31 on x86_64 kernels before 4.16: `R_X86_64_PLT32` rejected by the module loader and the relocs tool (b21ebf2fb4cd), and the `-z separate-code` default breaking 2 MB alignment (e3d03598e8ae). Both backported to 3.18, 4.4, 4.9 and 4.14.
- binutils 2.26 on i386 kernels before 4.6: `R_386_GOT32X` makes the compressed kernel fail to boot with "Failed to allocate space for phdrs" (6d92bc9d483a). An L5 failure caused by the linker.
- binutils 2.38 on riscv kernels before 5.17: the Zicsr and Zifencei split makes `csrr` an unknown opcode (6df2a016c0c8, backported to 4.19, 5.4, 5.10, 5.15).
- binutils 2.39: executable stack and RWX segment warnings, silenced by 0d362be5b142 in 6.0 and backported to 4.14 and later. From 6.18 (ec4a3992bc0b), `CONFIG_WERROR` makes linker warnings fatal, so the pairing matters more on new kernels than on old ones.

The sweep's output, `reports/binutils.md`, lists for each kernel the binutils range that works, and it feeds the kernel plan's persona choice of binutils (11.2).
