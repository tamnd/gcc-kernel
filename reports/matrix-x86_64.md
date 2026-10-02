# x86_64

One letter per cell: W works, R runs, B builds, F fails, · n/a, and blank where the cell has not run. A cell marked with `*` was flaky, its boots disagreed and it keeps the lowest. Written by `gk publish` from `matrix/matrix.json`.

## defconfig+gk

| Kernel | 8.5.0 | 9.5.0 | 10.5.0 | 11.5.0 | 12.2.0 | 12.5.0 | 13.5.0 | 14.2.0 | 14.4.0 | 15.3.0 | 16.1.0 | 16.2.0 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 6.12.111 |  |  |  |  | W |  |  |  |  |  |  |  |
| 7.2.8 | R* | W | R* | W | R* | W | W | R | W | W | W | W |
