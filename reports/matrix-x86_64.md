# x86_64

One square per cell: 🟩 works, 🟨 runs, 🟧 builds, 🟥 fails, · n/a, and blank where the cell has not run yet. ⚠️ marks a flaky cell, whose boots disagreed and which keeps the lowest rung. Written by `gk publish` from `matrix/matrix.json`.

## defconfig+gk

| Kernel | 8.5.0 | 9.5.0 | 10.5.0 | 11.5.0 | 12.2.0 | 12.5.0 | 13.5.0 | 14.2.0 | 14.4.0 | 15.3.0 | 16.1.0 | 16.2.0 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 6.12.111 |  |  |  |  | 🟩 |  |  |  |  |  |  |  |
| 6.18.54 | 🟨 | 🟨 | 🟩 | 🟨 | 🟨 | 🟨 | 🟨 | 🟨⚠️ | 🟨 | 🟧⚠️ | 🟨 |  |
| 7.2.8 | 🟩 | 🟩 | 🟩 | 🟩 | 🟩 | 🟩 | 🟨⚠️ | 🟨 | 🟩 | 🟩 | 🟩 | 🟩 |
