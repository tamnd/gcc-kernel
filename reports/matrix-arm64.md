# arm64

One square per cell: 🟩 works, 🟢 works on the smaller museum suite of a kernel before 2.6, 🟨 runs, 🟧 builds, 🟥 fails, · n/a, and blank where the cell has not run yet. ⚠️ marks a flaky cell, whose boots disagreed and which keeps the lowest rung. Written by `gk publish` from `matrix/matrix.json`.

## defconfig+gk

| Kernel | 4.8.5 | 4.9.2 | 4.9.4 | 5.5.0 | 6.3.0 | 6.5.0 | 7.5.0 | 8.3.0 | 8.5.0 | 9.5.0 | 10.5.0 | 11.5.0 | 12.2.0 | 12.5.0 | 13.5.0 | 14.2.0 | 14.4.0 | 15.3.0 | 16.1.0 | 16.2.0 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 7.2.8 |  |  |  |  |  |  |  |  | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 |

## tinyconfig+gk

| Kernel | 4.8.5 | 4.9.2 | 4.9.4 | 5.5.0 | 6.3.0 | 6.5.0 | 7.5.0 | 8.3.0 | 8.5.0 | 9.5.0 | 10.5.0 | 11.5.0 | 12.2.0 | 12.5.0 | 13.5.0 | 14.2.0 | 14.4.0 | 15.3.0 | 16.1.0 | 16.2.0 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 7.2.8 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 🟨 |
