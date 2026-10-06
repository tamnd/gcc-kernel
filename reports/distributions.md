# Distribution columns

A distribution column is a GCC as a distribution shipped it, run in that distribution's own image with its own binutils (spec 04.3). What sets it apart from the upstream release it was built from is its defaults, such as PIE, the stack protector and `-fcf-protection`, and this report shows which kernels notice them. Each row is the newest cell of the column, beside the newest cell of the upstream release it was built from, or of the newest upstream release of the same major series when the matrix lacks that one, on the same kernel, platform and configuration.

Written by `gk report distributions` from the result store. `gk cell K <column> --platform P` runs a row.

## debian-stretch-gcc-6, debian gcc-6 6.3.0

Debian stretch's own gcc-6, built with --enable-default-pie. Kernels before 8ae94224c9d7 and c6a385539175 (v4.9-rc6) fail to build with it, and upstream 6.3.0 does not show that. Run in gk-host-stretch with stretch's binutils 2.28, on the rows around the fix (spec 04.3).

| Kernel | Platform | Configuration | Verdict | Class | gcc-6.3.0 |
|---|---|---|---|---|---|
| 4.4.302 | x86_64 | defconfig+gk | fails at L2 | `over-budget` | not run |
| 4.7.10 | x86_64 | defconfig+gk | fails at L2 | `distro-pie` | not run |
| 4.8 | x86_64 | defconfig+gk | fails at L2 | `distro-pie` | not run |
| 4.8.17 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 4.9 | x86_64 | defconfig+gk | works at L8 |  | not run |

## ubuntu-hardy-gcc-4.2, ubuntu gcc-4.2 4.2.4

Ubuntu 8.04's own gcc-4.2, which builds with the plain -fstack-protector by default, as Ubuntu's GCC has since 6.10. Upstream 4.2.4 works on 2.6.17, so the two columns differ only in that default on the kernels from before kbuild passed -fno-stack-protector (spec 04.3).

| Kernel | Platform | Configuration | Verdict | Class | gcc-4.2.4 |
|---|---|---|---|---|---|
| 2.6.15 | x86_64 | defconfig+gk | fails at L2 | `distro-ssp` | works at L8 |
| 2.6.16 | x86_64 | defconfig+gk | fails at L2 | `distro-ssp` | works at L8 |
| 2.6.17 | x86_64 | defconfig+gk | fails at L2 | `distro-ssp` | works at L8 |
| 2.6.18 | x86_64 | defconfig+gk | works at L8 |  | works at L8 |
| 2.6.19 | x86_64 | defconfig+gk | builds at L4 | `over-budget` | builds at L4 |

## ubuntu-xenial-gcc-5, ubuntu gcc-5 5.4.0

Ubuntu 16.04's own gcc-5, which builds with -fstack-protector-strong by default and, unlike Ubuntu 16.10 and Debian stretch, without PIE. It shows which kernels notice the stronger stack protector alone (spec 04.3).

| Kernel | Platform | Configuration | Verdict | Class | gcc-5.5.0 |
|---|---|---|---|---|---|
| 2.6.32.71 | x86_64 | defconfig+gk | fails at L0 | `no-compiler-header` | not run |
| 3.0.101 | x86_64 | defconfig+gk | fails at L0 | `no-compiler-header` | not run |
| 3.2.102 | x86_64 | defconfig+gk | fails at L2 | `gnu11-extern-inline` | fails at L2 |
| 3.4.113 | x86_64 | defconfig+gk | fails at L2 | `gnu11-extern-inline` | fails at L2 |
| 3.10.108 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 3.12.74 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 3.14.79 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 3.16.85 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 3.18.140 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 4.4 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 4.4.302 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 4.9 | x86_64 | defconfig+gk | works at L8 |  | not run |

## ubuntu-eoan-gcc-9, ubuntu gcc-9 9.2.1

Ubuntu 19.10's own gcc-9, which builds with -fcf-protection and -fstack-clash-protection by default. The first rejects the -mindirect-branch flags of a retpoline kernel, so x86 kernels with CONFIG_RETPOLINE fail until kbuild passes -fcf-protection=none (spec 04.3).

| Kernel | Platform | Configuration | Verdict | Class | gcc-9.5.0 |
|---|---|---|---|---|---|
| 5.2-g43c95d3694cc | x86_64 | defconfig+gk | fails at L2 | `distro-cf-protection` | not run |
| 4.19 | x86_64 | defconfig+gk | fails at L2 | `distro-cf-protection` | fails at L2 |
| 4.19.325 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 5.2 | x86_64 | defconfig+gk | fails at L2 | `distro-cf-protection` | fails at L2 |
| 5.3 | x86_64 | defconfig+gk | builds at L4 |  | not run |
| 5.3.18 | x86_64 | defconfig+gk | works at L8 |  | not run |
| 5.4 | x86_64 | defconfig+gk | works at L8 |  | not run |

## Changed defaults

The signatures of kind `default-change`, and the cells of any column classified under each.

| Class | Flavors | GCC | Fixed by | Reproduced on |
|---|---|---|---|---|
| `distro-ssp` | ubuntu | >=4.1 | `34c162f79e37` (v2.6.18-rc1), `eb2cafa1d902` (v2.6.18-rc4) | 2.6.15 ubuntu-hardy-gcc-4.2, 2.6.16 ubuntu-hardy-gcc-4.2, 2.6.17 ubuntu-hardy-gcc-4.2 |
| `distro-pie` | debian, ubuntu | >=6 | `8ae94224c9d7` (v4.9-rc6), `c6a385539175` (v4.9-rc6) | 4.7.10 debian-stretch-gcc-6, 4.8 debian-stretch-gcc-6 |
| `distro-cf-protection` | ubuntu | >=9 | `29be86d7f9cb` (v5.3) | 4.19 ubuntu-eoan-gcc-9, 5.2 ubuntu-eoan-gcc-9, 5.2-g43c95d3694cc ubuntu-eoan-gcc-9 |
| `fno-common` | any | >=10 | `e33a814e772c` (v5.6) | 2.6.10 gcc-11.5.0, 2.6.10 gcc-15.3.0, 2.6.10 gcc-16.1.0, 2.6.11 gcc-10.5.0, 2.6.11 gcc-11.5.0, 2.6.11 gcc-12.2.0, 2.6.11 gcc-12.5.0, 2.6.11 gcc-13.5.0, 2.6.11 gcc-14.2.0, 2.6.11 gcc-14.4.0, 2.6.11 gcc-15.3.0, 2.6.11 gcc-16.2.0, 2.6.9 gcc-12.2.0, 2.6.9 gcc-15.3.0 |
| `gnu11-extern-inline` | any | >=5 | `8f375e10ee47` (v3.11-rc1), `51b97e354ba9` (v3.18-rc5) | 3.2.102 gcc-5.5.0, 3.2.102 ubuntu-xenial-gcc-5, 3.4.113 gcc-5.5.0, 3.4.113 ubuntu-xenial-gcc-5 |
| `gnu23-bool` | any | >=15 | `b3bee1e7c3f2` (v6.7), `ee2ab467bddf` (v6.14), `8ba14d9f490a` (v6.14), `3b8b80e99376` (v6.14), `947d5d036c78` (v6.13), `5a821e2d69e2` (v6.16), `7cbb015e2d3d` (v6.16), `0f4ae7c6ecb8` (v6.16) | 2.6.22 gcc-15.3.0, 2.6.22 gcc-16.1.0, 2.6.22 gcc-16.2.0, 2.6.28 gcc-15.3.0, 2.6.28 gcc-16.1.0, 2.6.28 gcc-16.2.0 |
