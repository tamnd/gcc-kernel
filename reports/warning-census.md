# Warning census

The compiler's warnings on the Current set, per GCC column (spec 11.5). Each cell's `warnings.jsonl` has them by unit and option, and this page adds them up over the newest cell at every crossing of a Current kernel with an upstream column, on every platform. A unit built with `-Werror` that warns is a broken build waiting for a compiler that warns more, so those units are listed on their own. Written by `gk publish`.

## defconfig+gk

### By GCC column

| GCC | Cells | Warnings | Per cell | Units | Options | Units under -Werror | Fatal |
|---|--:|--:|--:|--:|--:|--:|--:|
| 8.5.0 | 8 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 9.5.0 | 8 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 10.5.0 | 8 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 11.5.0 | 8 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 12.2.0 | 8 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 12.5.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 13.5.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 14.2.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 14.4.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 15.3.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 16.1.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 16.2.0 | 11 | 0 | 0.0 | 0 | 0 | 0 | 0 |

No column warns on any of the 93 cells.

## tinyconfig+gk

### By GCC column

| GCC | Cells | Warnings | Per cell | Units | Options | Units under -Werror | Fatal |
|---|--:|--:|--:|--:|--:|--:|--:|
| 8.5.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 9.5.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 10.5.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 11.5.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 12.2.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 12.5.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 13.5.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 14.2.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 14.4.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 15.3.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 16.1.0 | 7 | 0 | 0.0 | 0 | 0 | 0 | 0 |
| 16.2.0 | 8 | 0 | 0.0 | 0 | 0 | 0 | 0 |

No column warns on any of the 85 cells.
