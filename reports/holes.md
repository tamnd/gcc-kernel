# Holes

A hole is a cell that does not work between two columns of the same row that do (spec 08.5). It is where one GCC release broke a kernel that the releases around it built and ran, or where the rig was noisy, which is what the flaky mark and the three runs are for. Written by `gk publish` from `matrix/matrix.json`.

2 holes.

| Kernel | Platform | Configuration | GCC | Cell | Rung | Class | First error |
|---|---|---|---|:-:|---|---|---|
| 7.2.8 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟧⚠️ | L5 | unclassified | the boots disagreed |
| 7.2.8 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | L7 | gcc14-objtool-endbr |  |
