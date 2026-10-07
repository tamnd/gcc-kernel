# Warning census

The compiler's warnings on the Current set, per GCC column (spec 11.5). Each cell's `warnings.jsonl` has them by unit and option, and this page adds them up over the newest cell at every crossing of a Current kernel with an upstream column, on every platform. A unit built with `-Werror` that warns is a broken build waiting for a compiler that warns more, so those units are listed on their own. Written by `gk publish`.

## defconfig+gk

### By GCC column

| GCC | Cells | Warnings | Per cell | Units | Options | Units under -Werror | Fatal |
|---|--:|--:|--:|--:|--:|--:|--:|
| 8.5.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 9.5.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 10.5.0 | 3 | 1 | 0.3 | 1 | 1 | 0 | 0 |
| 11.5.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 12.2.0 | 4 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 12.5.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 13.5.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 14.2.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 14.4.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 15.3.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 16.1.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 16.2.0 | 17 | 1 | 0.1 | 1 | 1 | 0 | 0 |

Warnings per cell:

```text
8.5.0                                                 0.0
9.5.0                                                 0.0
10.5.0       ████████████████████████████████████████ 0.3
11.5.0                                                0.0
12.2.0                                                0.0
12.5.0                                                0.0
13.5.0                                                0.0
14.2.0                                                0.0
14.4.0                                                0.0
15.3.0                                                0.0
16.1.0                                                0.0
16.2.0       ███████                                  0.1
```

### By option

The first column is the oldest one that gives the warning on any Current cell, which is the release that introduced it for these kernels.

| Option | First column | Last column | Cells | Warnings |
|---|---|---|--:|--:|
| -Wunused-variable | 10.5.0 | 16.2.0 | 2 | 2 |

### Under -Werror

No unit built with `-Werror` warns.

## tinyconfig+gk

### By GCC column

| GCC | Cells | Warnings | Per cell | Units | Options | Units under -Werror | Fatal |
|---|--:|--:|--:|--:|--:|--:|--:|
| 16.2.0 | 1 | 0 | 0.0 | 0 | 0 | 0 | 0 |

No column warns on any of the 1 cells.
