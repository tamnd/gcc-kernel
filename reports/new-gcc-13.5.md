# GCC 13.5

The release report for gcc-13.5.0 (spec 10.8), over the 7 kernels of the Current set with defconfig+gk. It is written by `gk report new-gcc gcc-13.5.0` from the result store, and a crossing that has no cell yet is shown as not run.

## What builds and runs

### x86_64

Kernels with a cell: 2 of 7. Of those, 2 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.188 | - | not run | | | |
| 6.6.157 | - | not run | | | |
| 6.12.111 | - | not run | | | |
| 6.18.54 | - | runs | L6 | 0 |  |
| 7.2.8 | - | works | L8 | 0 |  |

### i386

Kernels with a cell: 1 of 7. Of those, 1 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.188 | - | not run | | | |
| 6.6.157 | - | not run | | | |
| 6.12.111 | - | not run | | | |
| 6.18.54 | - | not run | | | |
| 7.2.8 | - | works | L8 | 0 |  |

### arm64

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.188 | - | not run | | | |
| 6.6.157 | - | not run | | | |
| 6.12.111 | - | not run | | | |
| 6.18.54 | - | not run | | | |
| 7.2.8 | - | not run | | | |

### arm

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.188 | - | not run | | | |
| 6.6.157 | - | not run | | | |
| 6.12.111 | - | not run | | | |
| 6.18.54 | - | not run | | | |
| 7.2.8 | - | not run | | | |

### riscv64

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.188 | - | not run | | | |
| 6.6.157 | - | not run | | | |
| 6.12.111 | - | not run | | | |
| 6.18.54 | - | not run | | | |
| 7.2.8 | - | not run | | | |

### ppc64le

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.188 | - | not run | | | |
| 6.6.157 | - | not run | | | |
| 6.12.111 | - | not run | | | |
| 6.18.54 | - | not run | | | |
| 7.2.8 | - | not run | | | |

### s390x

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.188 | - | not run | | | |
| 6.6.157 | - | not run | | | |
| 6.12.111 | - | not run | | | |
| 6.18.54 | - | not run | | | |
| 7.2.8 | - | not run | | | |

### loongarch64

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.188 | - | not run | | | |
| 6.6.157 | - | not run | | | |
| 6.12.111 | - | not run | | | |
| 6.18.54 | - | not run | | | |
| 7.2.8 | - | not run | | | |

## Regressions against the same series

gcc-13.5.0 is the first release of its series in gccs.toml, so there is nothing to compare with.

## New warnings

Over the 3 cells that have a gcc-12.5.0 cell to compare with, no warning option fires more often than it did.

## Configuration differences

### 6.18.54 on x86_64

5 symbols differ.

| symbol | gcc-12.5.0 | gcc-13.5.0 |
|---|---|---|
| AS_VERSION | 24500 | 20285426 |
| CC_HAS_ASSUME | (absent) | y |
| CC_VERSION_TEXT | "x86_64-linux-gnu-gcc (GCC) 12.5.0" | "x86_64-linux-gnu-gcc (GCC) 13.5.0" |
| GCC_VERSION | 120500 | 130500 |
| LD_VERSION | 24500 | 20285426 |

### 7.2.8 on x86_64

5 symbols differ.

| symbol | gcc-12.5.0 | gcc-13.5.0 |
|---|---|---|
| AS_VERSION | 24500 | 20285426 |
| CC_HAS_ASSUME | (absent) | y |
| CC_VERSION_TEXT | "x86_64-linux-gnu-gcc (GCC) 12.5.0" | "x86_64-linux-gnu-gcc (GCC) 13.5.0" |
| GCC_VERSION | 120500 | 130500 |
| LD_VERSION | 24500 | 20285426 |

### 7.2.8 on i386

5 symbols differ.

| symbol | gcc-12.5.0 | gcc-13.5.0 |
|---|---|---|
| AS_VERSION | 24500 | 20285426 |
| CC_HAS_ASSUME | (absent) | y |
| CC_VERSION_TEXT | "i686-linux-gnu-gcc (GCC) 12.5.0" | "i686-linux-gnu-gcc (GCC) 13.5.0" |
| GCC_VERSION | 120500 | 130500 |
| LD_VERSION | 24500 | 20285426 |
