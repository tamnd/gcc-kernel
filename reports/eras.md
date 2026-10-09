# Eras

Whether the era GCC of each era works for every kernel in it (spec 11.2), for rucc-kernel's `personas.toml`. Each row is the newest cell of every GCC column run on one kernel, platform and configuration, and the working set runs from the oldest column that works to the newest. Where the era GCC does not work, the nearest working column is the fallback persona, and a person decides, because the persona also has to match what the era's distributions shipped. The kernel plan's open question 10, which 3.x and 4.x branches a later GCC can build, has its own report in [question-10.md](question-10.md). Written by `gk publish` from `matrix/matrix.json`.

| Era | Plan eras | From | Era GCC | Rows | Era GCC works | Era GCC does not | Era GCC not run |
|---|---|---|---|--:|--:|--:|--:|
| M0 | E0 | 1.0 | gcc-2.5.8 | 2 | 1 | 1 | 0 |
| M1 | E0, E1 | 1.3 | gcc-2.7.2.3 | 1 | 1 | 0 | 0 |
| M2 | E1 | 2.1 | gcc-2.7.2.3 | 1 | 1 | 0 | 0 |
| M3 | E2 | 2.3 | gcc-2.95.3 | 1 | 1 | 0 | 0 |
| M4 | E3 | 2.5 | gcc-3.3.6 | 38 | 22 | 16 | 0 |
| M5 | E4 | 2.6.16 | gcc-4.1.2 | 20 | 18 | 2 | 0 |
| M6 | E4 | 2.6.26 | gcc-4.3.5 | 51 | 46 | 1 | 4 |
| M7 | E5 | 3.0 | gcc-4.7.2 | 11 | 3 | 0 | 8 |
| M8 | E6 | 3.18 | gcc-4.9.2 | 1 | 1 | 0 | 0 |
| M9 | E7 | 4.2 | gcc-6.3.0 | 6 | 4 | 2 | 0 |
| M10 | E8 | 4.18 | gcc-8.3.0 | 6 | 0 | 5 | 1 |
| M11 | E8 | 5.5 | debian-bullseye-gcc-10 | 5 | 0 | 2 | 3 |
| M12 | E9 | 5.12 | debian-bullseye-gcc-10 | 5 | 0 | 2 | 3 |
| M13 | E10 | 5.18 | gcc-12.2.0 | 21 | 6 | 6 | 9 |
| M14 | E11 | 6.15 | gcc-14.2.0 | 18 | 4 | 6 | 8 |

## Proposed changes to personas.toml

Each line is a row where the era GCC does not work. One such row is evidence that the era should split or change its persona (spec 03.3), and the cell says why.

- 1.0 on i386 (defconfig+gk): gcc-2.5.8 fails at L2, and no column works.
- 2.6.3 on i386 (defconfig+gk): gcc-3.3.6 builds at L4, and the nearest working column is gcc-3.1.1.
- 2.6.7 on i386 (defconfig+gk): gcc-3.3.6 builds at L4, and the nearest working column is gcc-3.4.6.
- 2.6.0 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and no column works.
- 2.6.1 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and no column works.
- 2.6.2 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and the nearest working column is gcc-3.2.3.
- 2.6.3 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and no column works.
- 2.6.4 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and the nearest working column is gcc-3.2.3.
- 2.6.5 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and the nearest working column is gcc-3.2.3.
- 2.6.6 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and no column works.
- 2.6.7 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and the nearest working column is gcc-3.2.3.
- 2.6.8 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and no column works.
- 2.6.9 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and the nearest working column is gcc-3.2.3.
- 2.6.10 on x86_64 (defconfig+gk): gcc-3.3.6 fails at L2, and the nearest working column is gcc-3.2.3.
- 2.6.11 on x86_64 (defconfig+gk): gcc-3.3.6 builds at L4, and the nearest working column is gcc-3.4.6.
- 2.6.12 on x86_64 (defconfig+gk): gcc-3.3.6 builds at L4, and the nearest working column is gcc-3.4.6.
- 2.6.13 on x86_64 (defconfig+gk): gcc-3.3.6 builds at L4, and no column works.
- 2.6.20 on i386 (defconfig+gk): gcc-4.1.2 builds at L4, and the nearest working column is gcc-4.3.5.
- 2.6.19 on x86_64 (defconfig+gk): gcc-4.1.2 builds at L4, and the nearest working column is gcc-4.0.4.
- 2.6.39.4 on i386 (defconfig+gk): gcc-4.3.5 fails at L2, and the nearest working column is gcc-4.0.4.
- 4.8.17 on x86_64 (defconfig+gk): gcc-6.3.0 fails at L2, and the nearest working column is debian-stretch-gcc-6.
- 4.9 on x86_64 (defconfig+gk): gcc-6.3.0 fails at L2, and the nearest working column is debian-stretch-gcc-6.
- 4.19 on x86_64 (defconfig+gk): gcc-8.3.0 fails at L2, and no column works.
- 4.19.325 on x86_64 (defconfig+gk): gcc-8.3.0 fails at L2, and the nearest working column is ubuntu-eoan-gcc-9.
- 5.3 on x86_64 (defconfig+gk): gcc-8.3.0 fails at L2, and no column works.
- 5.3.18 on x86_64 (defconfig+gk): gcc-8.3.0 fails at L2, and the nearest working column is ubuntu-eoan-gcc-9.
- 5.4 on x86_64 (defconfig+gk): gcc-8.3.0 fails at L2, and the nearest working column is ubuntu-eoan-gcc-9.
- 5.10.270 on x86_64 (defconfig+gk): gcc-10.5.0 runs at L7, and no column works.
- 5.10.271 on x86_64 (defconfig+gk): gcc-10.5.0 runs at L7, and no column works.
- 5.15.221 on x86_64 (defconfig+gk): gcc-10.5.0 runs at L7, and no column works.
- 5.15.222 on x86_64 (defconfig+gk): gcc-10.5.0 runs at L7, and no column works.
- 6.12.111 on arm64 (defconfig+gk): gcc-12.2.0 fails at L2, and no column works.
- 6.1.189 on i386 (defconfig+gk): gcc-12.2.0 fails at L2, and no column works.
- 6.6.157 on i386 (defconfig+gk): gcc-12.2.0 fails at L2, and no column works.
- 6.1.188 on x86_64 (defconfig+gk): gcc-12.2.0 builds at L4, and no column works.
- 6.1.189 on x86_64 (defconfig+gk): gcc-12.2.0 runs at L6, and no column works.
- 6.6.158 on x86_64 (defconfig+gk): gcc-12.2.0 fails at L2, and no column works.
- 6.18.54 on arm64 (defconfig+gk): gcc-14.2.0 fails at L2, and no column works.
- 7.2.8 on arm64 (defconfig+gk): gcc-14.2.0 fails at L2, and no column works.
- 6.18.55 on i386 (defconfig+gk): gcc-14.2.0 fails at L2, and no column works.
- 6.18.54 on x86_64 (defconfig+gk): gcc-14.2.0 runs at L7, and the nearest working column is gcc-14.4.0.
- 6.18.55 on x86_64 (defconfig+gk): gcc-14.2.0 fails at L2, and no column works.
- 7.2.8 on x86_64 (defconfig+gk): gcc-14.2.0 runs at L7, and the nearest working column is gcc-14.4.0.

## M0, from 1.0, era GCC gcc-2.5.8

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 1.0 | i386 | defconfig+gk | gcc-2.5.8 | 🟥 | empty |  |
| 1.2.13 | i386 | defconfig+gk | gcc-2.5.8 | 🟩 | gcc-2.5.8 to gcc-2.6.3 |  |

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
| 2.4.37.11 | i386 | defconfig+gk | gcc-2.95.3 | 🟩 | gcc-2.95.3 to gcc-4.1.2 |  |

## M4, from 2.5, era GCC gcc-3.3.6

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 2.6.0 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.1.1 to gcc-3.3.6 |  |
| 2.6.1 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.1.1 to gcc-3.3.6 |  |
| 2.6.2 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.1.1 to gcc-3.3.6 |  |
| 2.6.3 | i386 | defconfig+gk | gcc-3.3.6 | 🟧 | gcc-3.1.1 to gcc-3.2.3 | gcc-3.1.1 |
| 2.6.4 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.1.1 to gcc-3.4.6 |  |
| 2.6.5 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.1.1 to gcc-3.4.6 |  |
| 2.6.6 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 |  |
| 2.6.7 | i386 | defconfig+gk | gcc-3.3.6 | 🟧 | gcc-3.4.6 | gcc-3.4.6 |
| 2.6.8 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 to gcc-3.4.6 |  |
| 2.6.9 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 to gcc-3.4.6 |  |
| 2.6.10 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 to gcc-3.4.6 |  |
| 2.6.11 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 to gcc-3.4.6 |  |
| 2.6.11.12 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 to gcc-3.4.6 |  |
| 2.6.12 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 to gcc-4.1.2 |  |
| 2.6.12.6 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 to gcc-4.2.4 |  |
| 2.6.13 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 to gcc-4.2.4 |  |
| 2.6.13.1 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 |  |
| 2.6.14 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.3.6 to gcc-4.2.4 |  |
| 2.6.15 | i386 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.1.1 to gcc-4.2.4 |  |
| 2.6.0 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | empty |  |
| 2.6.1 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | empty |  |
| 2.6.2 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | gcc-3.2.3 | gcc-3.2.3 |
| 2.6.3 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | empty |  |
| 2.6.4 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | gcc-3.2.3 | gcc-3.2.3 |
| 2.6.5 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | gcc-3.2.3 | gcc-3.2.3 |
| 2.6.6 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | empty |  |
| 2.6.7 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | gcc-3.2.3 | gcc-3.2.3 |
| 2.6.8 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | empty |  |
| 2.6.9 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | gcc-3.2.3 | gcc-3.2.3 |
| 2.6.10 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟥 | gcc-3.2.3 | gcc-3.2.3 |
| 2.6.11 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟧 | gcc-3.4.6 | gcc-3.4.6 |
| 2.6.11.12 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.2.3 to gcc-3.3.6 |  |
| 2.6.12 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟧 | gcc-3.4.6 | gcc-3.4.6 |
| 2.6.12.6 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.2.3 to gcc-4.2.4 |  |
| 2.6.13 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟧 | empty |  |
| 2.6.13.1 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.2.3 to gcc-4.2.4 |  |
| 2.6.14 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.2.3 to gcc-4.2.4 |  |
| 2.6.15 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟩 | gcc-3.2.3 to gcc-4.2.4 |  |

## M5, from 2.6.16, era GCC gcc-4.1.2

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 2.6.16 | i386 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.2.4 |  |
| 2.6.17 | i386 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.2.3 to gcc-4.5.4 |  |
| 2.6.18 | i386 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.1.2 |  |
| 2.6.19 | i386 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.20 | i386 | defconfig+gk | gcc-4.1.2 | 🟧 | gcc-3.3.6 to gcc-4.5.4 | gcc-4.3.5 |
| 2.6.21 | i386 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.22 | i386 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.23 | i386 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.3.6 |  |
| 2.6.24 | i386 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.3.6 |  |
| 2.6.25 | i386 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.16 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.2.3 to gcc-4.5.4 |  |
| 2.6.17 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-4.1.2 to gcc-4.2.4 |  |
| 2.6.18 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.19 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟧 | gcc-3.3.6 to gcc-4.0.4 | gcc-4.0.4 |
| 2.6.20 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.2.3 to gcc-4.3.5 |  |
| 2.6.21 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.2.3 to gcc-4.3.5 |  |
| 2.6.22 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.3.5 |  |
| 2.6.23 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.4.6 to gcc-4.3.5 |  |
| 2.6.24 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.2.3 to gcc-4.3.5 |  |
| 2.6.25 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟩 | gcc-3.3.6 to gcc-4.3.5 |  |

## M6, from 2.6.26, era GCC gcc-4.3.5

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 2.6.26 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.3.6 |  |
| 2.6.27 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.3.5 |  |
| 2.6.27.62 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.28 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.28.10 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.3.6 |  |
| 2.6.29 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.4.6 to gcc-4.3.6 |  |
| 2.6.29.6 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.30 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.4.6 to gcc-4.3.5 |  |
| 2.6.30.10 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.31 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.31.14 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.32 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.2.4 to gcc-4.3.5 |  |
| 2.6.32.71 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.33 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.1.2 to gcc-4.5.4 |  |
| 2.6.33.20 | i386 | defconfig+gk | gcc-4.3.5 |  | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.34 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.34.15 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.2.4 to gcc-4.5.4 |  |
| 2.6.35 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.35.14 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.36 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.36.4 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.37 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.3.5 |  |
| 2.6.37.6 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.38 | i386 | defconfig+gk | gcc-4.3.5 |  | empty |  |
| 2.6.38.8 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.2.4 to gcc-4.3.6 |  |
| 2.6.39 | i386 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.39.4 | i386 | defconfig+gk | gcc-4.3.5 | 🟥 | gcc-3.3.6 to gcc-4.2.4 | gcc-4.0.4 |
| 2.6.26 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.3.5 |  |
| 2.6.27 | x86_64 | defconfig+gk | gcc-4.3.5 |  | empty |  |
| 2.6.27.62 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.28 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.28.10 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.2.4 to gcc-4.5.4 |  |
| 2.6.29 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.29.6 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.30 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.1.2 to gcc-4.3.5 |  |
| 2.6.30.10 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.5.4 |  |
| 2.6.31 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-3.3.6 to gcc-4.4.7 |  |
| 2.6.31.14 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.4.7 |  |
| 2.6.32 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.3.5 |  |
| 2.6.32.71 | x86_64 | defconfig+gk | gcc-4.3.5 |  | empty |  |
| 2.6.33 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.33.20 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.34 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |
| 2.6.34.15 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.2.4 to gcc-4.5.4 |  |
| 2.6.35 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.1.2 to gcc-4.4.7 |  |
| 2.6.35.14 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.3.6 |  |
| 2.6.36 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.3.6 |  |
| 2.6.36.4 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.2.4 to gcc-4.5.4 |  |
| 2.6.37.6 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.2.4 to gcc-4.3.5 |  |
| 2.6.38 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.2.4 to gcc-4.5.4 |  |
| 2.6.39 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟩 | gcc-4.0.4 to gcc-4.5.4 |  |

## M7, from 3.0, era GCC gcc-4.7.2

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 3.0 | x86_64 | defconfig+gk | gcc-4.7.2 |  | empty |  |
| 3.0.101 | x86_64 | defconfig+gk | gcc-4.7.2 | 🟩 | gcc-4.0.4 to gcc-4.9.4 |  |
| 3.1 | x86_64 | defconfig+gk | gcc-4.7.2 |  | empty |  |
| 3.1.10 | x86_64 | defconfig+gk | gcc-4.7.2 | 🟩 | gcc-3.3.6 to gcc-4.9.2 |  |
| 3.2.102 | x86_64 | defconfig+gk | gcc-4.7.2 | 🟩 | gcc-4.7.2 |  |
| 3.3.8 | x86_64 | defconfig+gk | gcc-4.7.2 |  | gcc-4.9.2 |  |
| 3.4.113 | x86_64 | defconfig+gk | gcc-4.7.2 |  | empty |  |
| 3.10.108 | x86_64 | defconfig+gk | gcc-4.7.2 |  | ubuntu-xenial-gcc-5 |  |
| 3.12.74 | x86_64 | defconfig+gk | gcc-4.7.2 |  | ubuntu-xenial-gcc-5 |  |
| 3.14.79 | x86_64 | defconfig+gk | gcc-4.7.2 |  | ubuntu-xenial-gcc-5 |  |
| 3.16.85 | x86_64 | defconfig+gk | gcc-4.7.2 |  | ubuntu-xenial-gcc-5 |  |

## M8, from 3.18, era GCC gcc-4.9.2

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 3.18.140 | x86_64 | defconfig+gk | gcc-4.9.2 | 🟩 | gcc-4.9.2 to ubuntu-xenial-gcc-5 |  |

## M9, from 4.2, era GCC gcc-6.3.0

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 4.4 | x86_64 | defconfig+gk | gcc-6.3.0 | 🟩 | ubuntu-xenial-gcc-5 to gcc-6.3.0 |  |
| 4.4.302 | x86_64 | defconfig+gk | gcc-6.3.0 | 🟩 | ubuntu-xenial-gcc-5 to gcc-6.3.0 |  |
| 4.7.10 | x86_64 | defconfig+gk | gcc-6.3.0 | 🟩 | gcc-6.3.0 |  |
| 4.8 | x86_64 | defconfig+gk | gcc-6.3.0 | 🟩 | gcc-6.3.0 |  |
| 4.8.17 | x86_64 | defconfig+gk | gcc-6.3.0 | 🟥 | debian-stretch-gcc-6 | debian-stretch-gcc-6 |
| 4.9 | x86_64 | defconfig+gk | gcc-6.3.0 | 🟥 | ubuntu-xenial-gcc-5 to debian-stretch-gcc-6 | debian-stretch-gcc-6 |

## M10, from 4.18, era GCC gcc-8.3.0

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 4.19 | x86_64 | defconfig+gk | gcc-8.3.0 | 🟥 | empty |  |
| 4.19.325 | x86_64 | defconfig+gk | gcc-8.3.0 | 🟥 | ubuntu-eoan-gcc-9 | ubuntu-eoan-gcc-9 |
| 5.2 | x86_64 | defconfig+gk | gcc-8.3.0 |  | empty |  |
| 5.3 | x86_64 | defconfig+gk | gcc-8.3.0 | 🟥 | empty |  |
| 5.3.18 | x86_64 | defconfig+gk | gcc-8.3.0 | 🟥 | ubuntu-eoan-gcc-9 | ubuntu-eoan-gcc-9 |
| 5.4 | x86_64 | defconfig+gk | gcc-8.3.0 | 🟥 | ubuntu-eoan-gcc-9 | ubuntu-eoan-gcc-9 |

## M11, from 5.5, era GCC debian-bullseye-gcc-10

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 5.10.271 | arm64 | tinyconfig+gk | gcc-10.5.0 |  | empty |  |
| 5.10.271 | i386 | defconfig+gk | gcc-10.5.0 |  | empty |  |
| 5.10.271 | i386 | tinyconfig+gk | gcc-10.5.0 |  | empty |  |
| 5.10.270 | x86_64 | defconfig+gk | gcc-10.5.0 | 🟨 | empty |  |
| 5.10.271 | x86_64 | defconfig+gk | gcc-10.5.0 | 🟨 | empty |  |

## M12, from 5.12, era GCC debian-bullseye-gcc-10

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 5.15.222 | arm64 | tinyconfig+gk | gcc-10.5.0 |  | empty |  |
| 5.15.222 | i386 | defconfig+gk | gcc-10.5.0 |  | empty |  |
| 5.15.222 | i386 | tinyconfig+gk | gcc-10.5.0 |  | empty |  |
| 5.15.221 | x86_64 | defconfig+gk | gcc-10.5.0 | 🟨 | empty |  |
| 5.15.222 | x86_64 | defconfig+gk | gcc-10.5.0 | 🟨 | empty |  |

## M13, from 5.18, era GCC gcc-12.2.0

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 6.12.111 | arm64 | defconfig+gk | gcc-12.2.0 | 🟥 | empty |  |
| 6.12.112 | arm64 | defconfig+gk | gcc-12.2.0 |  | empty |  |
| 6.1.189 | arm64 | tinyconfig+gk | gcc-12.2.0 |  | empty |  |
| 6.6.158 | arm64 | tinyconfig+gk | gcc-12.2.0 |  | empty |  |
| 6.12.112 | arm64 | tinyconfig+gk | gcc-12.2.0 |  | empty |  |
| 6.1.189 | i386 | defconfig+gk | gcc-12.2.0 | 🟥 | empty |  |
| 6.6.157 | i386 | defconfig+gk | gcc-12.2.0 | 🟥 | empty |  |
| 6.6.158 | i386 | defconfig+gk | gcc-12.2.0 |  | empty |  |
| 6.12.111 | i386 | defconfig+gk | gcc-12.2.0 | 🟩 | gcc-8.5.0 to gcc-14.4.0 |  |
| 6.12.112 | i386 | defconfig+gk | gcc-12.2.0 |  | empty |  |
| 6.1.189 | i386 | tinyconfig+gk | gcc-12.2.0 |  | empty |  |
| 6.6.158 | i386 | tinyconfig+gk | gcc-12.2.0 |  | empty |  |
| 6.12.112 | i386 | tinyconfig+gk | gcc-12.2.0 |  | empty |  |
| 6.1.188 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟧 | empty |  |
| 6.1.189 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟨 | empty |  |
| 6.6.157 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟩 | gcc-8.5.0 to gcc-16.2.0 |  |
| 6.6.158 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟥 | empty |  |
| 6.12.111 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟩 | gcc-8.5.0 to gcc-14.4.0 |  |
| 6.12.112 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟩 | gcc-12.2.0 |  |
| 6.6.158 | x86_64 | tinyconfig+gk | gcc-12.2.0 | 🟩 | gcc-5.5.0 to gcc-13.5.0 |  |
| 6.12.112 | x86_64 | tinyconfig+gk | gcc-12.2.0 | 🟩 | gcc-5.5.0 to gcc-13.5.0 |  |

## M14, from 6.15, era GCC gcc-14.2.0

| Kernel | Platform | Configuration | Era column | Era cell | Working set | Nearest working column |
|---|---|---|---|:-:|---|---|
| 6.18.54 | arm64 | defconfig+gk | gcc-14.2.0 | 🟥 | empty |  |
| 6.18.55 | arm64 | defconfig+gk | gcc-14.2.0 |  | empty |  |
| 7.2.8 | arm64 | defconfig+gk | gcc-14.2.0 | 🟥 | empty |  |
| 7.2.9 | arm64 | defconfig+gk | gcc-14.2.0 |  | empty |  |
| 6.18.55 | arm64 | tinyconfig+gk | gcc-14.2.0 |  | empty |  |
| 7.2.8 | arm64 | tinyconfig+gk | gcc-14.2.0 |  | empty |  |
| 7.2.9 | arm64 | tinyconfig+gk | gcc-14.2.0 |  | empty |  |
| 6.18.54 | i386 | defconfig+gk | gcc-14.2.0 | 🟩 | gcc-8.5.0 to gcc-16.2.0 |  |
| 6.18.55 | i386 | defconfig+gk | gcc-14.2.0 | 🟥 | empty |  |
| 7.2.8 | i386 | defconfig+gk | gcc-14.2.0 | 🟩 | gcc-8.5.0 to gcc-16.2.0 |  |
| 7.2.9 | i386 | defconfig+gk | gcc-14.2.0 |  | empty |  |
| 6.18.55 | i386 | tinyconfig+gk | gcc-14.2.0 |  | empty |  |
| 7.2.9 | i386 | tinyconfig+gk | gcc-14.2.0 |  | empty |  |
| 6.18.54 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | gcc-8.5.0 to gcc-16.2.0 | gcc-14.4.0 |
| 6.18.55 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟥 | empty |  |
| 7.2.8 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | gcc-8.5.0 to gcc-16.2.0 | gcc-14.4.0 |
| 6.18.55 | x86_64 | tinyconfig+gk | gcc-14.2.0 | 🟩 | gcc-8.3.0 to gcc-16.2.0 |  |
| 7.2.9 | x86_64 | tinyconfig+gk | gcc-14.2.0 | 🟩 | gcc-8.3.0 to gcc-16.2.0 |  |
