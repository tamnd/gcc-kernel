# Holes

A hole is a cell that does not work between two columns of the same row that do (spec 08.5). It is where one GCC release broke a kernel that the releases around it built and ran, or where the rig was noisy, which is what the flaky mark and the three runs are for. Written by `gk publish` from `matrix/matrix.json`.

17 holes.

| Kernel | Platform | Configuration | GCC | Cell | Rung | Class | First error |
|---|---|---|---|:-:|---|---|---|
| 6.12.111 | i386 | defconfig+gk | gcc-11.5.0 | 🟥 | L2 | disk-full | drivers/gpu/drm/i915/display/intel_fifo_underrun.c: /src/drivers/gpu/drm/i915/display/intel_fifo_underrun.c:536:1: fatal error: error writing to /tmp/ccALpihd.s: No space left on device |
| 6.12.111 | i386 | defconfig+gk | gcc-12.5.0 | 🟨⚠️ | L6 | unclassified | the boots disagreed |
| 2.6.14 | x86_64 | defconfig+gk | gcc-4.1.2 | 🟧⚠️ | L4 | over-budget | the boots disagreed |
| 2.6.16 | x86_64 | defconfig+gk | gcc-4.0.4 | 🟧 | L4 | over-budget |  |
| 2.6.16 | x86_64 | defconfig+gk | ubuntu-hardy-gcc-4.2 | 🟥 | L2 | distro-ssp |  |
| 2.6.16 | x86_64 | defconfig+gk | gcc-4.3.5 | 🟧⚠️ | L4 | over-budget | the boots disagreed |
| 2.6.16 | x86_64 | defconfig+gk | gcc-4.3.6 | 🟧⚠️ | L4 | over-budget | the boots disagreed |
| 2.6.28 | x86_64 | defconfig+gk | gcc-3.4.6 | 🟧⚠️ | L4 | unclassified | the boots disagreed |
| 2.6.28 | x86_64 | defconfig+gk | gcc-4.3.6 | 🟧 | L4 | unclassified |  |
| 2.6.29 | x86_64 | defconfig+gk | gcc-4.0.4 | 🟧 | L4 | unclassified |  |
| 2.6.31 | x86_64 | defconfig+gk | gcc-3.4.6 | 🟥 | L2 | unclassified |  |
| 6.6.157 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | L7 | gcc14-objtool-endbr |  |
| 6.12.111 | x86_64 | defconfig+gk | gcc-12.5.0 | 🟧⚠️ | L4 | unclassified | the boots disagreed |
| 6.12.111 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | L7 | gcc14-objtool-endbr |  |
| 6.18.54 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | L7 | gcc14-objtool-endbr |  |
| 7.2.8 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟧⚠️ | L5 | unclassified | the boots disagreed |
| 7.2.8 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | L7 | gcc14-objtool-endbr |  |
