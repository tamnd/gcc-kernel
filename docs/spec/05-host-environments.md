# Host environments

A kernel build runs far more than the compiler. `make`, `perl`, `bison`, `flex`, `bc`, `python`, `openssl`, `libelf`, a host C compiler for everything under `scripts/` and `tools/`, and, for the old kernels, `as86` and `ld86`. Each of these has broken old kernels on its own, without a target compiler being involved. This document fixes the rule that keeps those failures out of the matrix, the host environments that implement it, and how the compiler under test runs inside them.

## 5.1 The rule: the kernel picks the host

The host environment H of a cell is a function of the kernel K alone, through its era in `eras.toml`. Every cell of a matrix row shares one H, so the only thing that changes along a row is the target toolchain. A failure that appears in one column and not another is the toolchain's.

The host is the distribution that was current when the kernel was, the one its developers would have used. Two known host failures show why the choice matters, and both disappear in the era's host:

| Failure | Kernels | Host tool | Why |
|---|---|---|---|
| `mixed implicit and normal rules` | before 2.6.36 | GNU make 3.82 and later | make 3.82 made an old Makefile idiom fatal. Squeeze and wheezy ship make 3.81 |
| `Can't use 'defined(@array)'` in `kernel/timeconst.pl` | 2.6.33 to 3.9 | perl 5.22 and later | the construct became fatal. Jessie's perl 5.20 still takes it |
| `multiple definition of 'yylloc'` in `scripts/dtc` | before 5.6, and stable branches before the backport (present today on 4.4 and later) | host GCC 10 and later | `-fno-common` became the default for `HOSTCC`, not only `CC` |
| `-Werror` in `tools/lib/subcmd` and objtool | around 5.15 to 5.17 | host GCC 12 | `-Wuse-after-free` in `xrealloc` |
| deprecated API warnings in `scripts/sign-file.c` and `certs/extract-cert.c` | 4.3 to around 5.18, exact range unverified | OpenSSL 3 | the ENGINE API deprecations; noisy everywhere, fatal where the tree builds host programs with `-Werror` |
| a.out tools missing | before 1.3.5, a.out only kernels | modern binutils | the `i386linux` a.out back end was removed from binutils around 2.31 |

## 5.2 The host environments

Each is a container image built from `provision/hosts/<name>/Dockerfile`, pushed to `ghcr.io/tamnd/gcc-kernel`, and pinned by digest in `hosts.toml`. The Debian releases come from the `debian/eol` images and `archive.debian.org`, and from `snapshot.debian.org` when a package has gone from the archive. The GCC and binutils versions of the old releases are from the archive's `Packages` indexes (document 01.7). The `make` versions are from memory and are checked by `gk hosts check` at G0, which rewrites this table from what the images report.

| Host | Base | Arch | Host GCC | binutils | make | Kernels it hosts |
|---|---|---|---|---|---|---|
| `gk-host-minix` | Minix-386 1.5.10 with Bruce Evans's 386 patches, in QEMU | emulated | gcc 1.37.1 or 1.40 | Minix `as`, `ld`, GNU gas 1.x | Minix make | 0.01 (document 07.7) |
| `gk-host-linux012` | a Linux 0.12 root image with gcc 1.40 binaries, in QEMU | emulated | gcc 1.40 | as86, gas, gld | make of the image | 0.10 to 0.12 (document 07.7) |
| `gk-host-bo` | `debian/eol:bo` | i386 | 2.7.2.1 | 2.7.0.9, plus `aout-binutils` 2.7 | 3.75 | 0.99.x to 1.2.13 |
| `gk-host-hamm` | `debian/eol:hamm` | i386 | 2.7.2.3 | 2.9.1 | 3.76 | 1.3.x, 2.0.x |
| `gk-host-slink` | `debian/eol:slink` | i386 | 2.7.2.3 | 2.9.1.0.19a | 3.77 | 2.1.x, 2.2.x |
| `gk-host-woody` | `debian/eol:woody` | i386 | 2.95.4 | 2.12.90.0.1 | 3.79.1 | 2.4.x |
| `gk-host-sarge` | `debian/eol:sarge` | i386, amd64 | 3.3.5 | 2.15 | 3.80 | 2.6.0 to 2.6.15 |
| `gk-host-etch` | `debian/eol:etch` | i386, amd64 | 4.1.1 | 2.17 | 3.81 | 2.6.16 to 2.6.25 |
| `gk-host-lenny` | `debian/eol:lenny` | i386, amd64 | 4.3.2 | 2.18.1 | 3.81 | 2.6.26 to 2.6.31 |
| `gk-host-squeeze` | `debian/eol:squeeze` | i386, amd64 | 4.4.5 | 2.20.1 | 3.81 | 2.6.32 to 3.1 |
| `gk-host-wheezy` | `debian/eol:wheezy` | i386, amd64 | 4.7.2 | 2.22 | 3.81 | 3.2 to 3.16 |
| `gk-host-jessie` | `debian/eol:jessie` | amd64 | 4.9.2 | 2.25 | 4.0 | 3.17 to 4.9 |
| `gk-host-stretch` | `debian/eol:stretch` | amd64 | 6.3.0 | 2.28 | 4.1 | 4.10 to 4.19 |
| `gk-host-buster` | `debian/eol:buster` | amd64 | 8.3.0 | 2.31.1 | 4.2.1 | 4.20 to 5.10 |
| `gk-host-bullseye` | `debian/eol:bullseye` | amd64 | 10.2.1 | 2.35.2 | 4.3 | 5.11 to 5.19 |
| `gk-host-bookworm` | `debian:bookworm` | amd64 | 12.2.0 | 2.40 | 4.3 | 6.0 to 6.12 |
| `gk-host-trixie` | `debian:trixie` | amd64 | 14.2.0 | 2.44 | 4.4.1 | 6.13 to current |

The kernel ranges are the first assignment, made by release date. They are hypotheses in the same sense as the kernel plan's eras: a range moves only when a cell with the era GCC fails on a host tool, and `eras.toml` records the reason. The first sweep's era diagonal (09.6) is what tests them.

The ranges are narrower than the kernel plan's eras, deliberately. The kernel plan groups by GCC persona. This repository groups by host, and hosts break kernels at finer boundaries than GCC versions do. `eras.toml` records both numberings, so a gcc-kernel era maps onto a kernel plan era (E0 to E11) without ambiguity.

Each host image contains exactly the host side: its distribution's `gcc` as `HOSTCC`, `make`, `perl`, `bison`, `flex`, `bc`, `python` where the era had it, `libelf-dev`, `libssl-dev`, `cpio`, `xz`, `kmod` or `module-init-tools`, and `bin86` for kernels before 2.3.21. It contains no GCC plugin headers, so `CONFIG_GCC_PLUGINS` stays off (5.6). It never contains the target toolchain.

## 5.3 How the compiler under test runs in a host it is not from

The row of a matrix puts GCC 16 inside the etch host and GCC 2.95 inside the trixie host. A toolchain has to run in every host. `gk` gets there in this order of preference:

1. **A static bundle.** The forge (04.4) links every toolchain binary statically: `gcc`, `cc1`, `collect2`, `as`, `ld`, `objcopy` and the rest, with GMP, MPFR, MPC and ISL built in-tree. A static binary for i386 or x86_64 runs in any container on an x86_64 host kernel. `gk forge verify` checks that no binary in a bundle has a `PT_INTERP` or a `DT_NEEDED` entry.
2. **The kernel.org crosstool binary**, when it runs in the host. These binaries are dynamically linked against the glibc of the machine that built them. They run in the modern hosts and fail in the old ones. `gk forge verify --host H` runs each kernel.org bundle in each host and records where it works, and `gk` uses it only there.
3. **Neither.** A toolchain that cannot be built static and has no kernel.org binary that runs in H has no cell in H's rows. The row shows those columns as **not run** with the reason "toolchain does not run in host", never as **fails**.

The bundle is mounted read-only at `/opt/gk/t/<id>` and the build is given `CROSS_COMPILE=/opt/gk/t/<id>/bin/<triple>-`, even for a native build. So `$(CROSS_COMPILE)gcc`, `as`, `ld`, `objcopy`, `nm` and `ar` all come from the bundle, and nothing of the target side comes from the host's `PATH`. The kernel's `HOSTCC` stays the host's `gcc`.

`PATH` is set by `gk` to the host's system directories followed by nothing else. That matters for 2.2.26, whose Makefile searches `PATH` for `gcc272` and `kgcc` before `cc` and would silently pick a host compiler if one were there. The host images for those eras do not install `gcc272` or `kgcc`, and `CC` is always given explicitly.

## 5.4 The i386 and a.out hosts

Everything up to 2.4 is i386, and the hosts for it are i386 images run with `--platform linux/386` on the x86_64 machines. An x86_64 host kernel runs i386 ELF binaries without help. It does not run a.out binaries: Linux removed a.out support on x86 in 5.19. So:
- every tool in an i386 host is ELF. Debian `bo` and `hamm` are ELF distributions that ship `aout-gcc` and `aout-binutils` as ELF programs that produce a.out objects, which is what 0.99 to 1.2 need;
- the period Linux a.out binaries (the gcc 1.40 `gccbin` of 1992) are only run inside an emulated old Linux, which is the `gk-host-linux012` environment.

## 5.5 Native mode

The matrix keeps `HOSTCC` fixed. The question people usually ask is different: "if my distribution's GCC is version N, can I build kernel K?" That is native mode, where `HOSTCC` is the compiler under test too. `gk cell --native` runs it:
- `HOSTCC=/opt/gk/t/<id>/bin/<triple>-gcc`, only on the platform whose architecture is the machine's;
- the bundle needs a C library for host programs, so native mode uses the full toolchains of the distribution flavor (04.3), not the bare cross bundles.

Native mode results go in a separate table (`reports/native.md`) and never in `matrix.json`. A native failure that the matrix cell does not have is a host program failure, and the report says which program.

## 5.6 Settings fixed in every host

| Setting | Value | Why |
|---|---|---|
| `KBUILD_BUILD_TIMESTAMP` | the kernel's release date | reproducible `vmlinux` bytes for the 2% determinism check (02.4) |
| `KBUILD_BUILD_USER`, `KBUILD_BUILD_HOST` | `gk`, `gcc-kernel` | the same |
| `SOURCE_DATE_EPOCH` | the same date | for the tools that read it |
| `LC_ALL` | `C` | old `Makefile` and `perl` code is locale sensitive |
| `TZ` | `UTC` | |
| GCC plugins | not installed | `CONFIG_GCC_PLUGINS` needs the plugin headers of the target GCC, which bundles do not ship. Off everywhere, on both the matrix and the reference |
| network | none | a build that fetches something is a failure |
| CPUs | 8 per build | recorded, and the same for every cell |
| memory | 16 GB per build | an OOM kill is a result |
| disk | a fresh scratch volume | deleted after the cell |

## 5.7 Checking a host

`gk hosts check` runs each image and compares `make --version`, `perl -v`, `gcc --version`, `ld --version`, `bison --version`, `flex --version` and the presence of `as86` with `hosts.toml`. It also builds the era's era-GCC cell for the first and last kernel of the host's range to L4. A host that fails its own check is not used, and the cells that would have used it are **not run**.
