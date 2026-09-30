# Building and booting a cell

This document is the procedure: what `gk cell` does from a pinned tree to a verdict. It covers the configuration, the build, the boot rig, the init program and its suites, and the old kernels that need a different path at every step. Most of it follows the kernel plan's boot rig ([the kernel plan's document 11](https://github.com/tamnd/rucc-kernel/blob/main/docs/plan/11-boot-and-tests.md)), because the two repositories should boot kernels the same way.

## 7.1 The steps

| Step | Rung | What runs |
|---|---|---|
| 1 | L0 | fetch the tarball from the cache, check SHA-256, unpack into scratch |
| 2 | L1 | the accept probe of 09.4, which is a subset of step 3 |
| 3 | L2 | `make ARCH=... CROSS_COMPILE=... O=out <config target>`, then merge the fragment, then `olddefconfig` (or `oldconfig` with defaults on old kernels), then check that every fragment option is set |
| 4 | L3 | `make ARCH=... CROSS_COMPILE=... O=out -j8 <image> modules` with `CC` through `gk-cc` |
| 5 | L4 | the image exists, objtool and modpost exited cleanly, the modules and the initramfs are packed |
| 6 | L5 | boot with the rig |
| 7 | L6 | the smoke suite |
| 8 | L7 | KUnit and in-kernel self tests, where the tree has them |
| 9 | L8 | the log and console checks |

`CC` is set to `gk-cc -- /opt/gk/t/<id>/bin/<triple>-gcc`, so kbuild's own `$(CROSS_COMPILE)gcc` is replaced by the same compiler behind the shim. Kernels older than `O=` support (2.4 and earlier) are built in a scratch copy of the tree, which is thrown away after the cell.

## 7.2 Configuration

The graded configurations:

| Name | Kernels | Recipe |
|---|---|---|
| `defconfig+gk` | all | the platform's defconfig target, then the era's fragment |
| `tinyconfig+gk` | 3.17 on | `tinyconfig`, then the fragment |
| `allnoconfig+gk` | 2.6.x to 3.16, on demand | `allnoconfig`, then the fragment |
| `oldconfig-default` | 2.4 and earlier | `yes "" | make oldconfig` from the tree's default `.config` or `arch/i386/defconfig`, then the fragment's options set through the same `oldconfig` pass |
| `allmodconfig` | Current set, build only | `allmodconfig` with `CONFIG_WERROR=n`, `CONFIG_GCC_PLUGINS=n`, `CONFIG_RUST=n`, reaching at most L4 |

The fragment only turns things on, as in the kernel plan's section 11.1, and adds the switches a matrix of compilers needs everywhere:

```
CONFIG_WERROR=n                # the kernel's own default for defconfig; stated so allmodconfig and defconfig agree
CONFIG_GCC_PLUGINS=n           # no plugin headers in bundles (05.6)
CONFIG_RUST=n
CONFIG_DEBUG_INFO_BTF=n        # needs pahole of the kernel's era, a host tool that varies separately
CONFIG_SERIAL_8250=y
CONFIG_SERIAL_8250_CONSOLE=y
CONFIG_SERIAL_AMBA_PL011=y
CONFIG_SERIAL_AMBA_PL011_CONSOLE=y
CONFIG_BLK_DEV_INITRD=y
CONFIG_DEVTMPFS=y
CONFIG_DEVTMPFS_MOUNT=y
CONFIG_TMPFS=y
CONFIG_PROC_FS=y
CONFIG_SYSFS=y
CONFIG_PRINTK_TIME=y
CONFIG_PANIC_ON_OOPS=y
CONFIG_KUNIT=y
CONFIG_KUNIT_ALL_TESTS=y       # built in, run at boot, so the rig needs no modprobe
```

The fragment turns two things off, `CONFIG_WERROR` and `CONFIG_GCC_PLUGINS`. That breaks the kernel plan's rule of only turning things on, and it is deliberate. `CONFIG_WERROR` is already off in `defconfig`, and setting it states the default. Plugins are off because the plugin headers are part of a GCC installation that the matrix does not have, so every cell would otherwise differ in configuration by whether headers happen to exist. rucc-kernel keeps plugins off for the same reason.

`CONFIG_DEBUG_INFO_BTF=n` keeps `pahole` out of the build. BTF generation depends on the pahole version, a host tool with its own history of breaking kernels, and that is not the question here.

Era fragments drop what the era does not have: KUnit before 5.5, devtmpfs before 2.6.32, `PANIC_ON_OOPS` before 2.6.x, the PL011 console for trees without it. `configs/fragment.<era>` holds each, and `gk` records which requested options did not take effect after the configuration step. An option that did not take effect with the era GCC is expected. One that took effect with the era GCC and not with another column is a finding: it means a Kconfig compiler probe disagreed, which document 11.3 wants.

Two kinds of configuration differences between columns are expected and not failures: `CC_VERSION_TEXT`, `GCC_VERSION`, `AS_VERSION`, `LD_VERSION` and the `CC_HAS_*` family that probes the compiler. They are what the configuration differential measures.

## 7.3 The build

- `make -j8` without `-k` for the strict rung. The first error stops the build, and its unit and message are the cell's first error.
- `V=1` so the log has every command.
- The L3w twin adds `KCFLAGS=-Wno-error` and nothing else, and the result says so.
- The `-k` run of 09.5 happens only for edge cells.
- `modules` is built wherever `CONFIG_MODULES` is set, because `modpost` and the module link are where section and symbol problems show.
- On x86_64 and i386 from 4.6, objtool runs as part of the build. Its warnings are counted, and its errors fail L4. On 6.x kernels with `CONFIG_OBJTOOL_WERROR`, warnings are errors by the kernel's own choice, and the L4o twin exists for those.
- The build budget is in `platforms.toml`. A build over budget is **fails** at the rung it was in, with the class `timeout`.

## 7.4 The boot rig

The rig is the kernel plan's rig with its old kernel paths made first class.

```
qemu-system-<arch> -machine <machine> -cpu <cpu> -smp <n> -m <mem> \
  -kernel <image> -initrd <gk-initramfs> \
  -append "console=<console> panic=-1 oops=panic gk.suite=<suite>" \
  -nographic -no-reboot
```

| Kernels | Boot path | Root | Results through |
|---|---|---|---|
| 2.6.0 on | `-kernel` and `-initrd` with a cpio initramfs | initramfs | serial console, framed as `GK-BEGIN <suite>` and `GK-END <suite> <status>` |
| 1.3.73 to 2.4 | `-kernel` and `-initrd` with an ext2 initrd image, `root=/dev/ram0` | initrd | serial console from 2.1.25, VGA text capture before (7.6) |
| 0.99.10 to 1.3.72 | `-kernel zImage`, no initrd, which QEMU refuses for boot protocol below 2.00 | a Minix or ext2 image on IDE, `root=/dev/hda` on the command line | VGA text capture |
| before 0.99.10 | floppy boot image with `-fda`, root on IDE, root device written into the boot image at offset 508 by the kernel's own `tools/build` and `ROOT_DEV` default | IDE image | VGA text capture |

The initramfs and initrd images are built once per platform and era by the era GCC in the era host, pinned by hash, and are identical for every column of a row. The kernel is the only thing that changes along a row.

`gk-init` is PID 1. It:
1. mounts what the kernel has: `proc`, `sysfs`, `devtmpfs`, `tmpfs`, `debugfs`;
2. prints `GK-BOOTED <uname -r>`;
3. reads `gk.suite` from the command line, or runs `smoke` when the kernel has no command line handling;
4. writes results to the console;
5. powers off with `reboot(RB_POWER_OFF)`, or halts on kernels without power off, which the rig detects as the end marker followed by silence for 5 seconds.

It is statically linked with no C library, using the raw system call instruction of each platform, so that one small program runs on every kernel of a platform without depending on a C library that assumes newer system calls. There are three builds of it: `init.c` for 2.6 on, `init-museum.S` for 1.x to 2.4 using only the system calls 1.0 had, and the 0.x check of 7.7.

## 7.5 The suites

**Smoke, 2.6 on.** About 20 checks, each a line: fork and wait; exec of itself in a child mode; a pipe round trip; signal delivery; `mmap` and `munmap` of 64 MB and a checksum over it; 100 threads through `clone` where the kernel has it; `/proc/self/maps` readable; a file written and read back on tmpfs; `nanosleep` returns; `gettimeofday` advances; `uname` matches the build; every online CPU seen in `/proc/cpuinfo`. The kernel plan's smoke suite is larger because it has a busybox userland. This one has no userland beyond `gk-init`, which keeps it independent of any C library and of any compiler but the era GCC.

**Smoke, 1.x to 2.4.** fork, exec, pipe, write and read back on the root filesystem, `sync`, and the end marker.

**KUnit, 5.5 on.** `CONFIG_KUNIT_ALL_TESTS=y` runs every suite at boot. The TAP output on the console is parsed by `gk-tap`. A suite passes if the era GCC's cell for the same kernel passes it in three runs of three. Otherwise it is not graded for that kernel, exactly as the kernel plan grades against its reference. L7 fails when a graded suite fails.

**In-kernel self tests before KUnit.** Where they exist and are built in by the fragment: `CONFIG_TEST_*` modules built in, the crypto self tests, the `RCU` torture test with a short duration (on demand), `CONFIG_DEBUG_LOCKING_API_SELFTESTS`. Graded the same way as KUnit.

## 7.6 VGA capture for kernels without a serial console

Kernels before 2.1.25 cannot put their console on a serial port. The rig runs QEMU with `-display none` and reads the VGA text buffer through the QEMU monitor's `pmemsave` of `0xb8000` once a second, turning each screen into lines of text. `gk-init-museum` prints its markers at the start of a line and clears the screen before its first line, so the capture is unambiguous. This is slower and it is only used where there is no alternative.

## 7.7 The 0.x experiment

0.x is in the matrix as reference only, and the kernel plan's open question 4 asks whether it is possible at all. The facts from document 01.6:
- **0.01** was built on Minix-386 with GCC 1.40, and its Makefile passes `-mstring-insns`, a flag of Linus's private GCC patch. Stock GCC 1.40 refuses the flag. The disk geometry is compiled in (`LINUS_HD` in `include/linux/config.h`), and the root is the first partition of the second disk, a Minix v1 file system.
- **0.02 and 0.03** have no surviving source.
- **0.10 to 0.12** use `as86` and `ld86` and the same GCC flags. They build under a self-hosted Linux 0.12 image with the gcc 1.40 binaries of the time.
- **0.95 to 0.99.x** move to GCC 2.x (2.3.3 by 0.99.10, 2.4.5 by 0.99.15), so they belong in the `gk-host-bo` environment with `aout-gcc`, if it takes them.

The experiment, time-boxed at G3:
1. Build `gk-host-minix` and try 0.01 with the Minix GCC 1.37.1 and 1.40 that are available. `-mstring-insns` either works, because the available binary has the patch, or the cell is **fails** at L3 with the class `private-gcc-flag`. Overriding `CFLAGS` on the make command line to drop the flag would be a configuration change that changes code generation, which 2.5 forbids, so it is not tried as a graded cell. It may be tried and reported as **ungraded**, to learn whether anything else stands in the way.
2. Build 0.11 and 0.12 in `gk-host-linux012`, with the gcc 1.40 in the image.
3. Boot what was built with `-fda` and an IDE disk whose geometry matches the tree's, on `isapc`.
4. Try 0.95, 0.96c, 0.97, 0.98 and 0.99.15 in `gk-host-bo` with `aout-gcc` 2.6.3 and 2.7.2.1.

The result is a written report, `reports/museum-0x.md`, that says for each tree what was built, with what, and whether it booted. Only trees that reach L5 unpatched with an era toolchain become rows of the matrix. The others stay in the report.
