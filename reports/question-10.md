# Open question 10

The kernel plan's open question 10 asks which 3.x and 4.x stable branches a later GCC can build. This report answers it from the matrix (spec 11.2). Each branch is read at its last point, and a later GCC is the era GCC of any era after the branch's own, as the column that stands for it on the platform. A row counts the newest cell of every GCC column run on the last point, so the answer grows as the frontier searches of G2 reach more branches. Written by `gk publish` from `matrix/matrix.json`.

Rows for a last point so far: 14, covering 14 of the 41 branches from 3.0 to 4.20. A later era's GCC builds the last point in 3 of them. In 1 every later era GCC that ran fails, so the branch stays with its own era's GCC or an older one. The rest have no later era GCC cell yet.

| Branch | Last point | Platform | Configuration | Era | Era GCC | Newest working column | Later era GCCs that work | Later era GCCs that do not |
|---|---|---|---|---|---|---|---|---|
| 3.0 | 3.0.101 | x86_64 | defconfig+gk | M7 | gcc-4.7.2 | gcc-4.9.4 | gcc-4.9.2 | gcc-6.3.0 |
| 3.1 | 3.1.10 | x86_64 | defconfig+gk | M7 | gcc-4.7.2 | gcc-4.9.2 | gcc-4.9.2 | gcc-6.3.0, gcc-8.3.0, gcc-10.5.0, gcc-12.2.0, gcc-14.2.0 |
| 3.2 | 3.2.102 | x86_64 | defconfig+gk | M7 | gcc-4.7.2 | gcc-4.7.2 |  | gcc-4.9.2 |
| 3.3 | 3.3.8 | x86_64 | defconfig+gk | M7 | gcc-4.7.2 | gcc-4.9.2 | gcc-4.9.2 | gcc-6.3.0 |
| 3.4 | 3.4.113 | x86_64 | defconfig+gk | M7 | gcc-4.7.2 | none |  |  |
| 3.10 | 3.10.108 | x86_64 | defconfig+gk | M7 | gcc-4.7.2 | ubuntu-xenial-gcc-5 |  |  |
| 3.12 | 3.12.74 | x86_64 | defconfig+gk | M7 | gcc-4.7.2 | ubuntu-xenial-gcc-5 |  |  |
| 3.14 | 3.14.79 | x86_64 | defconfig+gk | M7 | gcc-4.7.2 | ubuntu-xenial-gcc-5 |  |  |
| 3.16 | 3.16.85 | x86_64 | defconfig+gk | M7 | gcc-4.7.2 | ubuntu-xenial-gcc-5 |  |  |
| 3.18 | 3.18.140 | x86_64 | defconfig+gk | M8 | gcc-4.9.2 | ubuntu-xenial-gcc-5 |  |  |
| 4.4 | 4.4.302 | x86_64 | defconfig+gk | M9 | gcc-6.3.0 | gcc-6.3.0 |  |  |
| 4.7 | 4.7.10 | x86_64 | defconfig+gk | M9 | gcc-6.3.0 | gcc-6.3.0 |  |  |
| 4.8 | 4.8.17 | x86_64 | defconfig+gk | M9 | gcc-6.3.0 | debian-stretch-gcc-6 |  |  |
| 4.19 | 4.19.325 | x86_64 | defconfig+gk | M10 | gcc-8.3.0 | ubuntu-eoan-gcc-9 |  |  |
