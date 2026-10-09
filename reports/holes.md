# Holes

A hole is a cell that does not work between two columns of the same row that do (spec 08.5). It is where one GCC release broke a kernel that the releases around it built and ran, or where the rig was noisy, which is what the flaky mark and the three runs are for. Written by `gk publish` from `matrix/matrix.json`.

28 holes.

| Kernel | Platform | Configuration | GCC | Cell | Rung | Class | First error |
|---|---|---|---|:-:|---|---|---|
| 2.4.37.11 | i386 | defconfig+gk | gcc-3.2.3 | 🟥 | L3 | woody-depmod-kstrtab |  |
| 2.4.37.11 | i386 | defconfig+gk | gcc-3.3.6 | 🟥 | L3 | woody-depmod-kstrtab |  |
| 2.6.1 | i386 | defconfig+gk | gcc-3.2.3 | 🟧⚠️ | L5 | tcg-tsc-clock | the boots disagreed |
| 2.6.15 | i386 | defconfig+gk | gcc-3.2.3 | 🟥 | L2 | binutils-too-old-bignum | drivers/net/s2io.c: {standard input}:260: Error: bignum invalid |
| 2.6.15 | i386 | defconfig+gk | gcc-4.1.2 | 🟧⚠️ | L5 | tcg-tsc-clock | the boots disagreed |
| 2.6.20 | i386 | defconfig+gk | gcc-4.0.4 | 🟧⚠️ | L4 | tcg-ioapic-timer | the boots disagreed |
| 2.6.20 | i386 | defconfig+gk | gcc-4.1.2 | 🟧⚠️ | L4 | tcg-ioapic-timer | the boots disagreed |
| 2.6.20 | i386 | defconfig+gk | gcc-4.2.4 | 🟧⚠️ | L4 | tcg-ioapic-timer | the boots disagreed |
| 2.6.39 | i386 | defconfig+gk | gcc-3.4.6 | 🟧 | L4 | unclassified |  |
| 6.12.111 | i386 | defconfig+gk | gcc-9.5.0 | 🟥 | L2 | over-budget |  |
| 6.12.111 | i386 | defconfig+gk | gcc-11.5.0 | 🟥 | L2 | disk-full | drivers/gpu/drm/i915/display/intel_fifo_underrun.c: /src/drivers/gpu/drm/i915/display/intel_fifo_underrun.c:536:1: fatal error: error writing to /tmp/ccALpihd.s: No space left on device |
| 6.12.111 | i386 | defconfig+gk | gcc-12.5.0 | 🟨⚠️ | L6 | unclassified | the boots disagreed |
| 2.6.16 | x86_64 | defconfig+gk | ubuntu-hardy-gcc-4.2 | 🟥 | L2 | distro-ssp |  |
| 2.6.18 | x86_64 | defconfig+gk | ubuntu-hardy-gcc-4.2 | 🟧 | L4 | unclassified |  |
| 2.6.24 | x86_64 | defconfig+gk | gcc-3.3.6 | 🟧 | L4 | unclassified |  |
| 2.6.28 | x86_64 | defconfig+gk | gcc-3.4.6 | 🟧 | L4 | tcg-ioapic-timer |  |
| 2.6.31 | x86_64 | defconfig+gk | gcc-3.4.6 | 🟥 | L2 | gcc34-this-ip-label |  |
| 4.4.302 | x86_64 | defconfig+gk | debian-stretch-gcc-6 | 🟥 | L2 | over-budget |  |
| 6.6.157 | x86_64 | defconfig+gk | gcc-9.5.0 | 🟨⚠️ | L6 | unclassified | the boots disagreed |
| 6.6.157 | x86_64 | defconfig+gk | gcc-10.5.0 | 🟨⚠️ | L6 | unclassified | the boots disagreed |
| 6.6.157 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | L7 | gcc14-objtool-endbr |  |
| 6.12.111 | x86_64 | defconfig+gk | gcc-10.5.0 | 🟧⚠️ | L4 | over-budget | the boots disagreed |
| 6.12.111 | x86_64 | defconfig+gk | gcc-12.5.0 | 🟧⚠️ | L4 | unclassified | the boots disagreed |
| 6.12.111 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | L7 | gcc14-objtool-endbr |  |
| 6.18.54 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | L7 | gcc14-objtool-endbr |  |
| 7.2.8 | x86_64 | defconfig+gk | gcc-12.2.0 | 🟧⚠️ | L5 | unclassified | the boots disagreed |
| 7.2.8 | x86_64 | defconfig+gk | gcc-14.2.0 | 🟨 | L7 | gcc14-objtool-endbr |  |
| 6.12.112 | x86_64 | tinyconfig+gk | gcc-11.5.0 | 🟨⚠️ | L6 | unclassified | the boots disagreed |
