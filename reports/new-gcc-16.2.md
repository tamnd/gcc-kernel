# GCC 16.2

The release report for gcc-16.2.0 (spec 10.8), over the 7 kernels of the Current set with defconfig+gk. It is written by `gk report new-gcc gcc-16.2.0` from the result store, and a crossing that has no cell yet is shown as not run.

## What builds and runs

### x86_64

Kernels with a cell: 1 of 7. Of those, 1 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.270 | longterm | not run | | | |
| 5.15.221 | longterm | not run | | | |
| 6.1.188 | longterm | not run | | | |
| 6.6.157 | longterm | not run | | | |
| 6.12.111 | longterm | not run | | | |
| 6.18.54 | longterm | not run | | | |
| 7.2.8 | stable | works | L8 | 0 |  |

### i386

Kernels with a cell: 1 of 7. Of those, 1 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.270 | longterm | not run | | | |
| 5.15.221 | longterm | not run | | | |
| 6.1.188 | longterm | not run | | | |
| 6.6.157 | longterm | not run | | | |
| 6.12.111 | longterm | not run | | | |
| 6.18.54 | longterm | not run | | | |
| 7.2.8 | stable | runs | L7 | 0 |  |

### arm64

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.270 | longterm | not run | | | |
| 5.15.221 | longterm | not run | | | |
| 6.1.188 | longterm | not run | | | |
| 6.6.157 | longterm | not run | | | |
| 6.12.111 | longterm | not run | | | |
| 6.18.54 | longterm | not run | | | |
| 7.2.8 | stable | not run | | | |

## Regressions against the same series

Kernels that gcc-16.1.0 got further with. The signatures arrive with G3, so the first error is shown as the compiler wrote it.

| Platform | Kernel | gcc-16.1.0 | gcc-16.2.0 | First error |
|---|---|---|---|---|
| i386 | 7.2.8 | works | runs |  |

## New warnings

Over the 2 cells that have a gcc-16.1.0 cell to compare with, no warning option fires more often than it did.

## Configuration differences

### 7.2.8 on x86_64

2 symbols differ.

| symbol | gcc-16.1.0 | gcc-16.2.0 |
|---|---|---|
| CC_VERSION_TEXT | "x86_64-linux-gnu-gcc (GCC) 16.1.0" | "x86_64-linux-gnu-gcc (GCC) 16.2.0" |
| GCC_VERSION | 160100 | 160200 |

### 7.2.8 on i386

2 symbols differ.

| symbol | gcc-16.1.0 | gcc-16.2.0 |
|---|---|---|
| CC_VERSION_TEXT | "i686-linux-gnu-gcc (GCC) 16.1.0" | "i686-linux-gnu-gcc (GCC) 16.2.0" |
| GCC_VERSION | 160100 | 160200 |
