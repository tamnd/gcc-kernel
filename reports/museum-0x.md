# The 0.x experiment

This is the written result of spec 07.7 and of the G3 exit criterion that asks which of 0.01, 0.11, 0.12 and 0.99.15 built unpatched and booted, with what, or why not. Nothing here is a cell of the matrix. Every run below is ungraded and was done by hand on server2, and the method is written out at the end so that it can be run again.

## The answer

Pristine 0.11 and 0.12, from kernel.org's signed tarballs, build unpatched with Linus's own GCC 1.40 binary of September 1991 and boot to a shell, where a small smoke script passes. 0.01 does not build outside Minix. None of 0.95 to 0.99.15 builds with the GCC 2.5.8 bundle of the M0 era, each for a reason that is about the toolchain of its day and not about the kernel.

| Tree | Compiler | Built in | Built | Booted | What stopped it |
|---|---|---|:-:|:-:|---|
| 0.01 | gcc 1.40 of 1991-09-22 | the Linux 0.11 system, in Bochs | no | | `boot/boot.s` is written for the Minix assembler, and the size of the system is cut out of Minix's `ls -l` |
| 0.11 | the gcc 1.40 of the oldlinux disk | the Linux 0.11 system, in Bochs | no | | `gcc-cc1: Invalid option 'string-insns'`, the class `private-gcc-flag` |
| 0.11 | gcc 1.40 of 1991-09-22 | the Linux 0.11 system, in Bochs | yes | yes | nothing |
| 0.12 | gcc 1.40 of 1991-09-22 | the Linux 0.11 system, in Bochs | yes | yes | nothing, once the host has a `cpp` |
| 0.95 | gcc-2.5.8 bundle | `gk-host-hamm` | no | | the tree has no `config` target, so gk stops at L1 |
| 0.96c | gcc-2.5.8 bundle | `gk-host-hamm` | no | | the same |
| 0.99.11 | gcc-2.5.8 bundle | `gk-host-hamm` | no | | the Makefile compiles with `-x c++`, and the bundle has no `cc1plus` |
| 0.99.12 | gcc-2.5.8 bundle | `gk-host-hamm` | no | | the same |
| 0.99.13 | gcc-2.5.8 bundle | `gk-host-hamm` | no | | `conflicting types for 'panic'`: `kernel.h` says `volatile void` and `sched.h` says `void`, which 2.5.8 refuses |
| 0.99.15 | gcc-2.5.8 bundle | `gk-host-hamm` | no | | the link is `ld -T 100000`, the GNU ld 1.x way to set the text address, and binutils 2.8.1 reads it as a linker script |

So the kernel plan's question 4 has a yes: the `-mstring-insns` GCC was never lost. It is in `gccbin.tar.Z` on oldlinux, the binaries dated 22 September 1991, and its `gcc-cc1` takes the flag. It is the original binary, not a rebuild, so "unmodified toolchains" (spec 02.5) holds.

## The trees

All kernel tarballs come from kernel.org and match its signed `sha256sums.asc`. 0.99.14 is not on kernel.org.

| Tree | URL under `https://cdn.kernel.org/pub/linux/kernel/` | sha256 |
|---|---|---|
| 0.01 | `Historic/linux-0.01.tar.gz` | `24454f830cdb571e2c4ad15481119c43b3cafd48dd869a9b2945d1036d1dc68d` |
| 0.11 | `Historic/old-versions/linux-0.11.tar.gz` | `660997a6b1f03ee5aadce0ac806ffa653dedf1dea6d82c6731f9c9ddf0092673` |
| 0.12 | `Historic/old-versions/linux-0.12.tar.gz` | `302436edd2542dbe4ae082a618c5908a0a4bcd26f0319c8409c8d5a92b3761ee` |
| 0.95 | `Historic/old-versions/linux-0.95.tar.gz` | `c6ccd23e081c7f9e34ed2bff258e07ec10666d1df3e11444de766ff90713ef6f` |
| 0.96c | `Historic/old-versions/linux-0.96c.tar.gz` | `a913dd74fdc899af5a96e7ea1dd05ab1590a924fc58c444c9befffeeb3cc11d0` |
| 0.99.11 | `Historic/v0.99/linux-0.99.11.tar.gz` | `ab956c388d40910e8a49d2fecbd7cc5578108d963ec61e476afeefdea7cbb0b4` |
| 0.99.12 | `Historic/v0.99/linux-0.99.12.tar.gz` | `9553181da6482baa4b96d41019fd328759c4c1eca3ae07a8aef4201082ff19e5` |
| 0.99.13 | `Historic/v0.99/linux-0.99.13.tar.gz` | `c2d6f3539d570b285ad37c1f382730de2dbe5270f4b921f93ff4b253c80dffbf` |
| 0.99.15 | `Historic/v0.99/linux-0.99.15.tar.gz` | `6a5fe24a903a5fd892edc31e56ed807cac4b494ab9325fbafbe70a29689fd9ff` |

## The toolchain and the host

The files come from the oldlinux mirror on GitHub, `https://raw.githubusercontent.com/oldlinux-web/oldlinux-files/master/`.

| File | Path | sha256 |
|---|---|---|
| `gccbin.tar.Z` | `Linux-0.11/binaries/compilers/gccbin.tar.Z` | `d417c5727f02eff74595f658c87db9ef785439178a79425f1f867f11cd44dadf` |
| `linux-0.11-devel-060625.zip` | `bochs/linux-0.11-devel-060625.zip` | `c077a08f7b2489c06e8cb5c6768427247438ca0b3eba01fa7bb1a3de6f3838bb` |
| `hdc-0.11-new.img`, from the zip | | `61c136901658dfa38291382f0ef4933120fef7b716cf96fffebcb896d229b28a` |
| `bootimage-0.11-hd`, from the zip | | `b484347fddbef98db0b7682b4edddb1213a2cc8aa6ecf8639022c6d26b5758f7` |

`gccbin.tar.Z` holds `gcc`, `gcc-cc1`, `gcc-cpp`, `gcc-as`, `gcc-ld`, `libc.a` and the rest, all dated 1991-09-22. The three that were used:

| Binary | sha256 |
|---|---|
| `gcc` | `c9dd3c9482aa67aae124fce6e1f27a843d22051c8abb5264e3af0f7ba576904b` |
| `gcc-cc1` | `3c1cf1ec8f64c368a6caf34b2d4cb9f838615ac47c6fb4e810f14cd373ac3415` |
| `gcc-cpp` | `e54db5ad5b3af74ac72a1d85fcadde4bb3517a3f63daa8d6615a1c19df964cb8` |

The devel zip is Zhao Jiong's Linux 0.11 development system from oldlinux.org. Its hard disk has two Minix partitions with CHS 410/16/38, and it carries `as86` and `ld86` of January 1992, `gas`, `gld`, make and bash. Its own gcc 1.40 is a rebuild from March 2004 that does not know `-mstring-insns`, and its copy of the 0.11 source has the flag taken out of every Makefile. Neither was used for the result: the kernels are the kernel.org tarballs, and the compiler is the 1991 binary, copied over the 2004 one at the start of each run.

0.12 needs one thing the host does not have: its Makefile runs `cpp` for `boot/bootsect.S`, and the 0.11 system only has `/usr/local/lib/gcc-cpp`. A one line `/usr/bin/cpp` that execs `gcc-cpp` is enough. That is the host, not the kernel or the compiler.

## The builds

0.11 printed `gcc version 1.40`, compiled every unit with `-mstring-insns`, and ended with:

```
Root device is (3, 6)
Boot sector 512 bytes.
Setup is 312 bytes.
System is 118784 bytes.
```

The Image is 121344 bytes with sha256 `665ef3421e60ae0e62bcbc86a8b27e568730e9c432d0331f36371572abc9d443`. There were only warnings.

0.12 did the same and ended with:

```
Root device is (3, 6)
Boot sector 512 bytes.
Setup is 1372 bytes.
System is 147456 bytes.
```

The Image is 150016 bytes with sha256 `dace5d230161667c2bc037854094678a448a0dbbea3a7101d7a02b2ce9442c3c`, and the build printed six warnings. Each build took about six minutes of wall time in Bochs, on a host loaded to around 60.

Both Makefiles set `ROOT_DEV=/dev/hd6`, Linus's own disk. To boot, the two bytes at offset 508 of the boot sector were set to `01 03`, the first partition of the first disk, which is what `rdev` does and what the oldlinux boot images have. Nothing else in the Image was changed.

0.01 got as far as `boot/boot`. The Makefile builds the system first, cuts its size out of `ls -l tools/system` with `cut -c25-31`, which only works with Minix's column layout, and then assembles `boot.s`, which uses `|` for comments as the Minix assembler does. `as86` rejects almost every line. Both need the Minix-386 host of spec 05.2, `gk-host-minix`, which was not built inside the time box.

## The boots

Both Images were booted from a 1.44 MB floppy with the oldlinux disk as the first IDE disk, under Bochs 2.7 from Debian bookworm. `/etc/rc` on the disk was replaced with this script, which writes to `/dev/tty1`, the first serial port in these kernels:

```
/etc/update &
echo "/dev/hd1 /" > /etc/mtab
exec > /dev/tty1 2>&1
echo GK-BOOTED built-0.11
echo gk-file > /tmp/gk-file
cat /tmp/gk-file
ls / | wc
echo hello | cat
sync
echo GK-END smoke
```

Both printed `GK-BOOTED`, read the file back, ran the pipe and reached `GK-END smoke`. `wc` is not on that disk, which is the one line that did not run. 0.12 also printed `Partition tables ok.`, its free blocks and inodes and `Free mem: 12582912 bytes`, and left a root shell on the screen.

QEMU does not boot these kernels. On `isapc` and on `pc`, with the root on the floppy, 0.11 and 0.12 mount it and then loop on `Reset-floppy called` until process 2 panics. With the root on the hard disk they stop with `Unexpected HD interrupt`, `HD-controller reset failed: 00` and `Kernel panic: HD controller not ready`. QEMU's floppy and IDE models do not behave the way these early drivers expect, and getting past that would mean changing the kernel. Bochs emulates the old controllers closely enough, so `gk-host-linux012` and the boot of 07.7 step 3 are Bochs and not QEMU.

A floppy image must be exactly 1474560 bytes. A shorter file makes QEMU guess a 2.88 MB drive with 36 sectors a track, and the 0.11 boot sector then hangs before it reaches `setup`.

The 0.11 userland was also tried under a 2.4 kernel, to build without an emulated 0.11. It runs: bash and gcc 1.40 start once `/dev/console` exists. But its libc reads directories with `read`, which 2.4 answers with `EISDIR`, so `ls` sees nothing and make fails with `getwd` and "No way to make target `Image'". That way is closed.

## The 0.9x trees

These ran as gk cells, ungraded, with the gcc-2.5.8 bundle for i486-linuxaout and binutils 2.8.1, in `gk-host-hamm` rather than `gk-host-bo` for the reason in `eras.toml`. The cells are in a scratch store and are not published. 0.97 and 0.98 were not tried.

| Tree | Rung reached | First error |
|---|---|---|
| 0.95 | L0 | `No rule to make target 'config'` |
| 0.96c | L0 | the same |
| 0.99.11 | L2 | `cannot exec cc1plus` for `init/main.c`, compiled with `-x c++` |
| 0.99.12 | L2 | the same |
| 0.99.13 | L2 | `include/linux/sched.h:87: conflicting types for 'panic'` |
| 0.99.15 | L2 | `gk-ld: cannot open linker script file 100000` at the link of `tools/zSystem` |

0.95 and 0.96c are configured by editing the top Makefile, so a config step for them would be a no-op and the build might go further. 0.99.11 and 0.99.12 are from the weeks when Linus compiled the kernel as C++ to get stricter type checks, so they need a GCC bundle with C++, which the forge does not build (spec 04.4 builds C only). 0.99.13 was built with a GCC 2.4, which let a `volatile void` function be declared again as `void`, and 2.5.8 does not. 0.99.15 wants the GNU ld 1.x of its day, where `-T` took the text address. 0.99.15 also stopped making uncompressed images, so `platforms.toml` now builds `zImage` from 0.99.12, the first tree whose Makefile has no `Image` target that works.

## What goes in the matrix

Spec 07.7 lets a 0.x tree become a row only if it reaches L5 unpatched with an era toolchain. 0.11 and 0.12 do, but only in Bochs, which gk does not drive yet, so they stay in this report until `gk-host-linux012` exists as a Bochs host with this disk and these binaries. None of the 0.9x trees reaches L5 with the bundles that exist, so none becomes a row, and `kernels.toml` is unchanged.

## How to run it again

Everything ran in docker on server2 in `/var/tmp/gk/0x`. The Bochs image is Debian bookworm with `bochs`, `bochsbios`, `vgabios` and `bochs-term`. Debian's Bochs has its debugger built in and no `nogui` display, so it runs with the RFB display, `display_library: rfb, options="timeout=0"`, and a debugger script that alternates `sb 2000000000`, `c` and a `writemem` of the text screen at `0xb8000`, so the screen can be read from outside. The configuration has 16 MB of memory, `cpu: ips=50000000` and `clock: sync=none`, the boot floppy as `floppya`, the oldlinux disk as `ata0-master` with `cylinders=410, heads=16, spt=38`, a second disk as `ata0-slave`, and `com1` written to a file.

The second disk carries the inputs. It has an MBR with two partitions of type 0x81. The first, which the kernel calls `/dev/hd6`, holds the kernel tar. The second, `/dev/hd7`, holds the uncompressed `gccbin.tar`. The `/etc/rc` of a build run unpacks `gccbin.tar` from `/dev/hd7`, copies `gcc`, `gcc-cc1` and `gcc-cpp` over the 2004 ones, unpacks the kernel from `/dev/hd6`, runs `make Image`, and writes the Image back to `/dev/hd6` with `cat`, where it is read from sector 1 of the second disk after the run. `/etc/rc` lives in a single zone of the Minix file system, and the run rewrites that zone in a fresh copy of the disk each time.

The 0.12 `setup` waits for a key at "Press <RETURN> to see SVGA-modes available" and reads it from port 0x60, so a key in the BIOS buffer does not help. The run puts the container on an internal docker network and sends a space bar over RFB.
