# Eras

Whether the era GCC of each era works for every kernel in it (spec 11.2), for rucc-kernel's `personas.toml`. Each row is the newest cell of every GCC column run on one kernel, platform and configuration, and the working set runs from the oldest column that works to the newest. Where the era GCC does not work, the nearest working column is the fallback persona, and a person decides, because the persona also has to match what the era's distributions shipped. Written by `gk publish` from `matrix/matrix.json`.

| Era | Plan eras | From | Era GCC | Rows | Era GCC works | Era GCC does not | Era GCC not run |
|---|---|---|---|--:|--:|--:|--:|
| M13 | E10 | 5.18 | gcc-12.2.0 | 1 | 1 | 0 | 0 |
| M14 | E11 | 6.15 | gcc-14.2.0 | 5 | 2 | 2 | 1 |

## Proposed changes to personas.toml

Each line is a row where the era GCC does not work. One such row is evidence that the era should split or change its persona (spec 03.3), and the cell says why.

- 6.18.54 on x86_64 (defconfig+gk): gcc-14.2.0 runs at L6, and the nearest working column is gcc-10.5.0.
- 7.2.8 on x86_64 (defconfig+gk): gcc-14.2.0 runs at L7, and the nearest working column is gcc-14.4.0.

## M13, from 5.18, era GCC gcc-12.2.0

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 6.12.111 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟩 | gcc-12.2.0 |  |

## M14, from 6.15, era GCC gcc-14.2.0

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 7.2.8 | arm64 | defconfig+gk | gcc-14.2.0 | 🟩 | gcc-14.2.0 |  |
| 7.2.8 | arm64 | tinyconfig+gk | gcc-14.2.0 |  | empty |  |
| 7.2.8 | i386 | defconfig+gk | gcc-14.2.0 | 🟩 | gcc-8.5.0 to gcc-16.1.0 |  |
| 6.18.54 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | gcc-10.5.0 | gcc-10.5.0 |
| 7.2.8 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | gcc-8.5.0 to gcc-16.2.0 | gcc-14.4.0 |
