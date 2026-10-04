# Eras

Whether the era GCC of each era works for every kernel in it (spec 11.2), for rucc-kernel's `personas.toml`. Each row is the newest cell of every GCC column run on one kernel, platform and configuration, and the working set runs from the oldest column that works to the newest. Where the era GCC does not work, the nearest working column is the fallback persona, and a person decides, because the persona also has to match what the era's distributions shipped. Written by `gk publish` from `matrix/matrix.json`.

| Era | Plan eras | From | Era GCC | Rows | Era GCC works | Era GCC does not | Era GCC not run |
|---|---|---|---|--:|--:|--:|--:|
| M0 | E0 | 1.0 | gcc-2.5.8 | 2 | 1 | 1 | 0 |
| M1 | E0, E1 | 1.3 | gcc-2.7.2.3 | 1 | 1 | 0 | 0 |
| M2 | E1 | 2.1 | gcc-2.7.2.3 | 1 | 1 | 0 | 0 |
| M3 | E2 | 2.3 | gcc-2.95.3 | 1 | 1 | 0 | 0 |
| M4 | E3 | 2.5 | gcc-3.4.6 | 3 | 0 | 3 | 0 |
| M6 | E4 | 2.6.26 | gcc-4.3.5 | 1 | 1 | 0 | 0 |
| M13 | E10 | 5.18 | gcc-12.2.0 | 1 | 1 | 0 | 0 |
| M14 | E11 | 6.15 | gcc-14.2.0 | 5 | 1 | 3 | 1 |

## Proposed changes to personas.toml

Each line is a row where the era GCC does not work. One such row is evidence that the era should split or change its persona (spec 03.3), and the cell says why.

- 1.0 on i386 (defconfig+gk): gcc-2.5.8 fails at L2, and no column works.
- 2.6.0 on i386 (defconfig+gk): gcc-3.4.6 fails at L2, and no column works.
- 2.6.1 on i386 (defconfig+gk): gcc-3.4.6 fails at L2, and no column works.
- 2.6.0 on x86_64 (defconfig+gk): gcc-3.4.6 fails at L2, and no column works.
- 7.2.8 on arm64 (defconfig+gk): gcc-14.2.0 fails at L2, and no column works.
- 6.18.54 on x86_64 (defconfig+gk): gcc-14.2.0 runs at L7, and the nearest working column is gcc-14.4.0.
- 7.2.8 on x86_64 (defconfig+gk): gcc-14.2.0 runs at L7, and the nearest working column is gcc-14.4.0.

## M0, from 1.0, era GCC gcc-2.5.8

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 1.0 | i386 | defconfig+gk | gcc-2.5.8 | 🟥 | empty |  |
| 1.2.13 | i386 | defconfig+gk | gcc-2.5.8 | 🟩 | gcc-2.5.8 |  |

## M1, from 1.3, era GCC gcc-2.7.2.3

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 2.0.40 | i386 | defconfig+gk | gcc-2.7.2.3 | 🟩 | gcc-2.7.2.3 |  |

## M2, from 2.1, era GCC gcc-2.7.2.3

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 2.2.26 | i386 | defconfig+gk | gcc-2.7.2.3 | 🟩 | gcc-2.7.2.3 to gcc-2.95.3 |  |

## M3, from 2.3, era GCC gcc-2.95.3

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 2.4.37.11 | i386 | defconfig+gk | gcc-2.95.3 | 🟩 | gcc-2.95.3 to gcc-3.1.1 |  |

## M4, from 2.5, era GCC gcc-3.4.6

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 2.6.0 | i386 | defconfig+gk | gcc-3.4.6 | 🟥 | empty |  |
| 2.6.1 | i386 | defconfig+gk | gcc-3.4.6 | 🟥 | empty |  |
| 2.6.0 | x86_64 | defconfig+gk | gcc-3.4.6 | 🟥 | empty |  |

## M6, from 2.6.26, era GCC gcc-4.3.5

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 2.6.32 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.3.5 |  |

## M13, from 5.18, era GCC gcc-12.2.0

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 6.12.111 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟩 | gcc-11.5.0 to gcc-13.5.0 |  |

## M14, from 6.15, era GCC gcc-14.2.0

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 7.2.8 | arm64 | defconfig+gk | gcc-14.2.0 | 🟥 | empty |  |
| 7.2.8 | arm64 | tinyconfig+gk | gcc-14.2.0 |  | empty |  |
| 7.2.8 | i386 | defconfig+gk | gcc-14.2.0 | 🟩 | gcc-8.5.0 to gcc-16.2.0 |  |
| 6.18.54 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | gcc-10.5.0 to gcc-16.2.0 | gcc-14.4.0 |
| 7.2.8 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | gcc-8.5.0 to gcc-16.2.0 | gcc-14.4.0 |
