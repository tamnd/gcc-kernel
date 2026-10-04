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
| `tinyconfig+gk` | 3.17 on | `tinyconfig`, then `configs/tiny.gk`, then the fragment |
| `allnoconfig+gk` | 2.6.x to 3.16, on demand | `allnoconfig`, then `configs/tiny.gk`, then the fragment |
| `oldconfig-default` | 2.4 and earlier | `yes "" | make oldconfig` from the tree's default `.config` or `arch/i386/defconfig`, then the fragment's options set through the same `oldconfig` pass |
| `allmodconfig` | Current set, build only | `allmodconfig` with `CONFIG_WERROR=n`, `CONFIG_GCC_PLUGINS=n`, `CONFIG_RUST=n`, reaching at most L4 |

`configs/tiny.gk` holds what gk-init and the fragment need and the two smallest targets switch off: `BINFMT_ELF` for gk-init, `TTY` and `PRINTK` for the console, `SHMEM` for `TMPFS`, and `BUG` so that `WARN_ON` still prints for L8. That way a tiny cell that does not boot failed on the compiler rather than on a missing feature. It is part of the configuration's digest, in front of the fragment. `gk cell` and `gk search` take the configuration with `--config`, and `defconfig+gk` is the default. `allmodconfig` takes only the lines of the fragment that turn something off, which are `WERROR`, `GCC_PLUGINS`, `RUST` and `DEBUG_INFO_BTF`, because the rest would turn modules into built-ins and change what the configuration tests.

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

Era fragments drop what the era does not have: KUnit before 5.5, devtmpfs before 2.6.32, `PANIC_ON_OOPS` before 2.6.x, the PL011 console for trees without it. The M13 and M14 fragments also turn off `KUNIT_FAULT_TEST`, whose suite oopses on purpose and so panics a kernel built with `PANIC_ON_OOPS`. The first M13 cell, 6.12.111 with gcc-12.2.0, died that way in its KUnit boots, and so did the M14 era cell for 6.18.54. 7.2.8 skips the suite by itself. Two other suites, the DRM scheduler tests and the ratelimit stress test, fail under TCG on a loaded host with any compiler, and `KUNIT_ALL_TESTS` hides their options so a fragment cannot turn them off. A fragment line `# gk:kunit-skip <option>...` names such tests instead. After the fragment is applied, `gk` turns `KUNIT_ALL_TESTS` off in the resolved `.config`, which shows the test options again with the values it gave them, turns the named ones off and runs `olddefconfig` a second time into `kunit-skip.log`. On 7.2.8 that pass changes nothing but the named tests and `DRM_SCHED`, which only the scheduler test selects. M14 skips `DRM_SCHED_KUNIT_TEST` and `RATELIMIT_KUNIT_TEST`. The KUnit `module!=` filter on the command line looked simpler and was tried first, but on 7.2.8 it panics in `attr_string_filter` on a test case with no module name, so it is not used. `configs/fragment.<era>` holds each, and `gk` records which requested options did not take effect after the configuration step. An option that did not take effect with the era GCC is expected. One that took effect with the era GCC and not with another column is a finding: it means a Kconfig compiler probe disagreed, which document 11.3 wants.

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
qemu-system-<arch> -machine <machine> -cpu <cpu> -accel tcg -smp 2 -m 1024 \
  -kernel <image> -initrd <gk-initramfs> \
  -append "console=<console> panic=-1 oops=panic gk.suite=<suite> gk.kernel=<version> gk.cpus=2" \
  -nographic -monitor none -nic none -no-reboot
```

`gk boot` runs it in the `gk-boot` container, which holds QEMU and nothing else and is pinned in `hosts.toml` like the hosts. The container's digest is the cell's `qemu` coordinate. The console comes back on stdout, every line goes to `boot.log`, and what the rig read goes to `boot.json`: the release from `GK-BOOTED`, every `GK-CHECK`, the status from `GK-END`, the first panic line and every warning splat. The run ends when the kernel powers off. The rig stops it when the platform's boot budget runs out, or when the console has been quiet for 5 seconds after the end marker. A cell boots its image this way three times, into `boot-1.log` to `boot-3.log` with a `.json` next to each, unless the first boot never reaches `gk-init`, and `cell.json` records the rung of each boot, the lowest as the cell's rung, and whether they disagreed (02.4).

| Kernels | Boot path | Root | Results through |
|---|---|---|---|
| 2.6.0 on | `-kernel` and `-initrd` with a cpio initramfs | initramfs | serial console, framed as `GK-BEGIN <suite>` and `GK-END <suite> <status>` |
| 2.3 to 2.4, boot protocol 2.03 | `-kernel` and `-initrd` with a Minix initrd image, `root=/dev/ram0` | initrd | serial console |
| 1.3.73 to 2.2, boot protocol 2.00 to 2.02 | a SYSLINUX disk on the first IDE drive that loads the kernel and the Minix initrd image, `root=/dev/ram0` | initrd | serial console from 2.1.25, VGA text capture before (7.6) |
| 0.99.10 to 1.3.72 | the same SYSLINUX disk with no initrd, since these zImages have no setup header | a Minix image on the second IDE drive, `root=/dev/hdb` | VGA text capture |
| before 0.99.10 | floppy boot image with `-fda`, root on IDE, root device written into the boot image at offset 508 by the kernel's own `tools/build` and `ROOT_DEV` default | IDE image | VGA text capture |

QEMU's own loader cannot start a kernel older than boot protocol 2.03. Whatever the protocol, `-kernel` writes `initrd_addr_max` at offset 0x22c of the setup code, a field that 2.03 added, and in older kernels that offset holds the first instructions of setup, so they die on an invalid opcode before printing anything. The `gk-boot` container carries SYSLINUX for them, and `gk-syslinux` writes a small FAT disk with the kernel, the initrd and the command line at the start of each boot. The rig picks the loader from the protocol in the image's setup header, not from the version.

Museum boots get one CPU, 32 MB and a CPU model that matches the era's default configuration: a 486 before 2.0, a Pentium for 2.0, and a Pentium III from 2.1, whose defaults are 686 and Pentium III kernels. From 2.1 the SMP kernels read the IO-APIC from the MP table that SeaBIOS writes and route the timer and the disks through it, which QEMU does not wire the way they expect, so the command line carries `noapic`.

The initramfs and initrd images are built once per platform and era by the era GCC in the era host, pinned by hash, and are identical for every column of a row. The kernel is the only thing that changes along a row. For 2.6 on, `gk init` builds `init.c` with the bundle and host that `platforms.toml` names under `init` for each platform, which is GCC 8.5 in the trixie host until the older columns are forged, and i386 borrows the x86_64 bundle with `-m32`. It writes the newc archive itself, with every owner, time and inode fixed, so the digest depends on the program alone and two machines get the same one.

`gk-init` is PID 1. It:
1. mounts what the kernel has: `proc`, `sysfs`, `devtmpfs`, `tmpfs`, `debugfs`;
2. prints `GK-BOOTED <uname -r>`;
3. reads `gk.suite` from the command line, or runs `smoke` when the kernel has no command line handling;
4. writes results to the console;
5. powers off with `reboot(RB_POWER_OFF)`, or halts on kernels without power off, which the rig detects as the end marker followed by silence for 5 seconds.

It is statically linked with no C library, using the raw system call instruction of each platform, so that one small program runs on every kernel of a platform without depending on a C library that assumes newer system calls. There are three builds of it: `init.c` for 2.6 on, `init-museum.c` for 1.x to 2.4 on i386, using only the system calls 1.0 had and packed on a Minix root image that gk writes itself, and the 0.x check of 7.7.

## 7.5 The suites

**Smoke, 2.6 on.** 18 checks, each a `GK-CHECK <name> pass` or `fail` line: init is PID 1; fork and wait; exec of itself in a child mode; a pipe round trip, within one process and from a child; signal delivery; `mmap` and `munmap` of 64 MB and a checksum over it; `brk` up and down; `/proc/self/maps` readable; `/dev/null` writable; sysfs mounted; a file written and read back on tmpfs; a directory made, renamed and removed; `dup2`; `nanosleep` of 20 ms seen by the monotonic clock; `uname` reports the kernel the rig booted, which it passes as `gk.kernel`; 100 children forked and reaped; every CPU QEMU was given, passed as `gk.cpus`, seen in `/proc/cpuinfo`. Threads through `clone` with a shared address space need a stack trampoline per platform and come later. The kernel plan's smoke suite is larger because it has a busybox userland. This one has no userland beyond `gk-init`, which keeps it independent of any C library and of any compiler but the era GCC.

**Smoke, 1.x to 2.4.** fork, exec, pipe, write and read back on the root filesystem, `sync`, and the end marker.

**KUnit, 5.5 on.** `CONFIG_KUNIT_ALL_TESTS=y` runs every suite at boot. That run takes minutes under TCG, so a cell boots for it separately, after each smoke boot that passed, with `gk.suite=kunit` and the KUnit budget of 06.7, and every other boot turns KUnit off on the command line. The TAP output on the console is parsed by `gk-tap`, and `kunit.json` lines every suite up across the three runs, names the cell it was graded against and lists what each run failed. A suite passes if the era GCC's cell for the same kernel passes it in three runs of three. When the era GCC is not a column, as the Debian build that M11 and M12 name is not, the newest column of its release series stands in for it, `gcc-10.5.0` for `debian-bullseye-gcc-10`, the same rule `gk pins changed` uses for its probes. Otherwise it is not graded for that kernel, exactly as the kernel plan grades against its reference. L7 fails when a graded suite fails.

**In-kernel self tests before KUnit.** Where they exist and are built in by the fragment: `CONFIG_TEST_*` modules built in, the crypto self tests, the `RCU` torture test with a short duration (on demand), `CONFIG_DEBUG_LOCKING_API_SELFTESTS`. Graded the same way as KUnit.

**Splats, for L8.** A run reaches L8 from L7 when the build logs have no `warning: objtool:` line, its smoke boot logged no splat, and its KUnit boot logged no splat the era GCC's cell did not log too. Some KUnit suites warn on purpose, `drm_atomic` and `int_log` among them on 7.2.8, so a KUnit splat only counts when it is new. Two splats are the same when they match once the CPU, the task and the offsets into functions are taken out, since those change from boot to boot and build to build. The era GCC's own cell has nothing to compare its KUnit splats with and is not held to them. `splats.json` lists what kept each run from L8.

## 7.6 VGA capture for kernels without a serial console

Kernels before 2.1.25 cannot put their console on a serial port. The rig runs QEMU with `-display none` and reads the VGA text buffer through the QEMU monitor's `pmemsave` of `0xb8000` once a second, turning each screen into lines of text. The monitor's file name is quoted, or it reads `4000 /vga/screen` as a division. The consoles of the era scroll by moving the start of the screen through video memory, so the command line carries `no-scroll`, which 2.0 knows, and `gk-init-museum` switches to the second virtual console and back before its first line, which copies the screen to the start of video memory on every kernel from 1.0. It then clears the screen and prints fewer lines than a screen holds, so the capture is unambiguous. This is slower and it is only used where there is no alternative.

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
