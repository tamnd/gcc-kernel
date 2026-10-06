# GCC 16.2

The release report for gcc-16.2.0 (spec 10.8), over the 7 kernels of the Current set with defconfig+gk. It is written by `gk report new-gcc gcc-16.2.0` from the result store, and a crossing that has no cell yet is shown as not run.

## What builds and runs

### x86_64

Kernels with a cell: 6 of 7. Of those, 4 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | fails | L2 | 0 |  |
| 5.15.222 | longterm | fails | L2 | 0 |  |
| 6.1.189 | longterm | runs | L6 | 0 |  |
| 6.6.158 | longterm | runs | L6 | 0 |  |
| 6.12.112 | longterm | runs | L6 | 0 |  |
| 6.18.55 | longterm | runs | L6 | 0 |  |
| 7.2.9 | stable | not run | | | |

### i386

Kernels with a cell: 7 of 7. Of those, 6 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | runs | L6 | 1 |  |
| 5.15.222 | longterm | runs | L6 | 0 |  |
| 6.1.189 | longterm | runs | L6 | 0 |  |
| 6.6.158 | longterm | runs | L6 | 0 |  |
| 6.12.112 | longterm | runs | L6 | 0 |  |
| 6.18.55 | longterm | runs | L6 | 0 |  |
| 7.2.9 | stable | builds | L5 | 0 |  |

### arm64

Kernels with a cell: 1 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | fails | L2 | 0 |  |
| 7.2.9 | stable | not run | | | |

### arm

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### riscv64

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### ppc64le

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### s390x

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### loongarch64

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### mips

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### ppc

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### ppc64

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### sparc64

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### m68k

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### alpha

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### parisc

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### sh4

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### openrisc

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### xtensa

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

### microblaze

Kernels with a cell: 0 of 7. Of those, 0 run.

| Kernel | Line | Verdict | Rung | Warnings | First error |
|---|---|---|---|--:|---|
| 5.10.271 | longterm | not run | | | |
| 5.15.222 | longterm | not run | | | |
| 6.1.189 | longterm | not run | | | |
| 6.6.158 | longterm | not run | | | |
| 6.12.112 | longterm | not run | | | |
| 6.18.55 | longterm | not run | | | |
| 7.2.9 | stable | not run | | | |

## Regressions against the same series

No kernel that gcc-16.1.0 got to a verdict with does worse with gcc-16.2.0.

## New warnings

No kernel has a cell with both gcc-16.1.0 and gcc-16.2.0 yet.

## Configuration differences

No kernel has a cell with both gcc-16.1.0 and gcc-16.2.0 yet.
