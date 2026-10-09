# Bisections

Every `fixed-by` commit of the failure catalog is confirmed by bisecting between a kernel the signature matches and one it does not, with the signature's GCC (spec 03.5 and 08.2). When the bisection finds another commit, the bisected commit wins and the signature is corrected. A signature can name several commits, one for each place the kernel had to change, and a bisection confirms the one its range and platform reach. Written by `gk publish` from `signatures.toml` and the bisections in the result store.

20 bisections in the store. 20 of the 67 signatures with a fixing commit have at least one of their commits confirmed. 29 of the others match no cell of the matrix yet, so there is no failing kernel to bisect from until one does. Cells counts the newest run of each coordinate that the signature matches.

## Signatures

| Signature | Rung | Kind | Fixing commits | Confirmed | Cells |
|---|---|---|---|---|--:|
| gcc-min-32 | L1..L3 | refusal | `fd285bb54d8a` | `fd285bb54d8a` | 60 |
| gcc-296-frame-pointer | L1 | refusal | `fd285bb54d8a` | `fd285bb54d8a` | 0 |
| gcc3-0-1-refused | L1..L3 | refusal | `6680598` |  | 5 |
| arm-gcc3-below-33 | L1 | refusal | `a136564` |  | 0 |
| gcc41-weak | L1 | blacklist | `f9d1425` |  | 0 |
| no-compiler-header | L1 | refusal | `cb984d101b30` `71458cfc782e` | `cb984d101b30` `71458cfc782e` | 47 |
| compiler-h-gnuc-gt-4 | L1..L3 | refusal | `f153b82121b0` `cb984d101b30` | `f153b82121b0` `cb984d101b30` | 221 |
| arm-gcc48-pr58854 | L1 | blacklist | `7fc150543c73` |  | 0 |
| gcc-min-46 | L1 | refusal | `cafa0010cd51` | `cafa0010cd51` | 0 |
| gcc-min-49 | L1 | refusal | `6ec4476ac825` | `6ec4476ac825` | 0 |
| gcc-min-51 | L1 | refusal | `76ae847497bc` | `76ae847497bc` | 48 |
| gcc-min-81 | L1 | refusal | `a3e8fe814ad1` `118c40b7b503` | `a3e8fe814ad1` `118c40b7b503` | 8 |
| asm-goto-miscompile | L5..L7 | miscompile | `3f0116c` `a9f180345f53` |  | 2 |
| gcc49-load-balance | L5..L7 | miscompile | `2062afb4f804` |  | 0 |
| gcc10-start-secondary | L5 | miscompile-by-assumption | `a9a3ed1eff36` |  | 0 |
| gcc15-counted-by-kunit | L7 | miscompile-by-assumption | `04e403e6627d` |  | 6 |
| gcc15-union-padding | L5..L7 | miscompile-by-assumption | `dce4aab8441d` |  | 1 |
| arm-gcc47-ice-migrate | L3 | ice | `6ec4476ac825` | `6ec4476ac825` | 0 |
| too-old-asm-goto | L3 | too-old | `e501ce957a78` |  | 0 |
| too-old-generic | L3 | too-old | `6ec4476ac825` | `6ec4476ac825` | 0 |
| tcg-rtc-kunit-stall | L8 | emulator | `95c46336ab47` |  | 7 |
| distro-ssp | L3 | default-change | `34c162f79e37` `eb2cafa1d902` | `eb2cafa1d902` | 3 |
| distro-pie | L3 | default-change | `8ae94224c9d7` `c6a385539175` | `8ae94224c9d7` | 2 |
| distro-cf-protection | L3 | default-change | `29be86d7f9cb` | `29be86d7f9cb` | 2 |
| fno-common | L3 | default-change | `e33a814e772c` |  | 20 |
| gnu11-extern-inline | L3 | default-change | `e2afe67453e5` `14bfc987e395` `8f375e10ee47` `51b97e354ba9` |  | 9 |
| gnu23-bool | L3 | default-change | `b3bee1e7c3f2` `ee2ab467bddf` `8ba14d9f490a` `3b8b80e99376` `947d5d036c78` `5a821e2d69e2` `7cbb015e2d3d` `0f4ae7c6ecb8` |  | 9 |
| gcc42-proxy-pda | L3 | new-error | `1bac3b383a93` | `1bac3b383a93` | 1 |
| gcc48-mutex-slowpath-unused | L3 | new-error | `7918baa55514` |  | 1 |
| gcc48-ptrace-asmregparm | L3 | new-error | `1b4ac2a935aa` | `1b4ac2a935aa` | 11 |
| gcc46-m-elf-i386 | L3 | new-error | `de2a8cf98ecd` |  | 24 |
| gcc3-vdso-plt-overlap | L3 | too-old | `c65916fe3586` | `c65916fe3586` | 1 |
| gcc34-this-ip-label | L3 | too-old | `4a92379bdfb4` | `4a92379bdfb4` | 19 |
| gcc32-directive-in-macro-args | L3 | too-old | `9cf4f298e29a` |  | 2 |
| gcc44-setup-dil | L3 | new-error | `811a0fff5d6e` | `811a0fff5d6e` | 3 |
| gcc43-no-unit-at-a-time | L3 | new-error | `9ab34fe76114` | `9ab34fe76114` | 12 |
| gcc7-format-werror | L3 | new-warning-werror | `bd664f6b3e37` |  | 0 |
| gcc8-packed-not-aligned | L3 | new-warning-werror | `321cb0308a9e` |  | 0 |
| gcc8-attribute-alias | L3 | new-warning-werror | `bee20031772a` |  | 0 |
| gcc8-stringop-truncation | L3 | new-warning-werror | `217c3e019675` |  | 0 |
| gcc9-missing-attributes | L3 | new-warning-werror | `c0d9782f5b6d` `a6e60d84989f` |  | 0 |
| gcc9-packed-member | L3 | new-warning-werror | `6f303d60534c` |  | 0 |
| gcc10-warnings-werror | L3 | new-warning-werror | `5c45de21a222` `5a76021c2eff` |  | 0 |
| gcc11-stringop-overread | L3 | new-warning-werror | `e7c6e405e171` |  | 0 |
| gcc12-array-bounds | L3 | new-warning-werror | `f0be87c42cbd` |  | 0 |
| gcc12-dangling-pointer | L3 | new-warning-werror | `49beadbd47c2` |  | 0 |
| gcc13-enum-type | L3 | new-error | `525ff9c29657` |  | 0 |
| gcc15-unterminated-string | L3 | new-warning-werror | `d5d45a7f2619` `9d7a0577c9db` |  | 0 |
| gcc16-unused-but-set | L3 | new-warning-werror | `97fb54d86d21` `729a2e8e9ac4` |  | 0 |
| gcc8-objtool-cold | L4..L8 | objtool | `13810435b9a7` |  | 0 |
| gcc8-objtool-switch | L4..L8 | objtool | `fd35c88b7441` |  | 0 |
| gcc16-modpost-ipa-init | L4 | modpost | `4c9ad387aa2d` |  | 0 |
| binutils-too-old-assert | L3 | binutils-too-old | `d2ba8b211bb8` |  | 6 |
| binutils-too-old-cfi | L3 | binutils-too-old | `d1526e2cda64` |  | 3 |
| binutils-too-old-multiline-macro | L3 | binutils-too-old | `6e3515352bdd` |  | 1 |
| binutils-216-user32-cs | L3 | binutils-too-old | `dd2897bf0f4d` |  | 2 |
| binutils-size-undefined | L3 | binutils-assembler | `ad2fc2cd9253` |  | 11 |
| binutils-segment-mov | L3 | binutils-assembler | `fd51f666fa59` |  | 109 |
| binutils-range-ok-cmp | L3 | binutils-assembler | `722f4f5b2600` | `722f4f5b2600` | 27 |
| binutils-plt32 | L4..L5 | binutils-relocation | `b21ebf2fb4cd` |  | 0 |
| binutils-separate-code | L5 | binutils-layout | `e3d03598e8ae` |  | 0 |
| binutils-got32x | L5 | binutils-relocation | `6d92bc9d483a` |  | 0 |
| binutils-objtool-symtab | L3 | binutils-objtool | `1d489151e9f9` |  | 2 |
| binutils-riscv-zicsr | L3 | binutils-isa | `6df2a016c0c8` |  | 0 |
| binutils-rwx-warning | L4 | binutils-warning | `0d362be5b142` `ffcf9c5700e4` |  | 0 |
| binutils-loongarch-relax | L5..L7 | binutils-relocation | `03c53eb90c0c` |  | 0 |
| binutils-ppc-ztext | L4 | binutils-link | `97f902dd4c99` |  | 0 |

## Runs

| Range | GCC | Platform | Judged by | Steps | First commit | Subject | In | Signature |
|---|---|---|---|--:|---|---|---|---|
| 2.6.15..2.6.16 | gcc-2.95.3 | i386 | `arch/i386/kernel/asm-offsets.s` | 13 | `fd285bb54d8a` | [PATCH] Abandon gcc-2.95.x | v2.6.16-rc1 | gcc-min-32, gcc-296-frame-pointer |
| 2.6.15..2.6.16 | gcc-4.3.5 | i386 | `arch/i386/kernel/apic.o` | 14 | `9ab34fe76114` | [PATCH] enable unit-at-a-time optimisations for gcc4 | v2.6.16-rc1 | gcc43-no-unit-at-a-time |
| 2.6.17..2.6.18 | gcc-4.4.7 | i386 | `fs/binfmt_aout.o` | 13 | `722f4f5b2600` | [PATCH] x86: fix __range_ok constraint | v2.6.18-rc1 | binutils-range-ok-cmp |
| 2.6.17..2.6.18 | ubuntu-hardy-gcc-4.2 | x86_64 | L3 | 12 | `eb2cafa1d902` | kbuild: -fno-stack-protector is not good | v2.6.18-rc4 | distro-ssp |
| 2.6.19..2.6.20 | gcc-4.3.5 | x86_64 | L3 | 12 | `1bac3b383a93` | [PATCH] x86: Work around gcc 4.2 over aggressive optimizer | v2.6.20-rc1 | gcc42-proxy-pda |
| 2.6.23..2.6.24 | gcc-3.3.6 | x86_64 | L3 | 13 | `c65916fe3586` | x86: vdso linker script cleanup | v2.6.24-rc1 | gcc3-vdso-plt-overlap |
| 2.6.24..2.6.25 | gcc-4.5.4 | i386 | `arch/x86/boot/video.o` | 14 | `811a0fff5d6e` | x86 setup: fix constraints in segment accessor functions | v2.6.25-rc1 | gcc44-setup-dil |
| 2.6.26..2.6.27 | gcc-3.2.3 | x86_64 | `arch/x86/kernel/smpboot.o` | 14 | `1c5b0eb66d74` | x86: fix readb() et al compile error with gcc-3.2.3 | v2.6.27-rc4 | none |
| 2.6.28..2.6.29 | gcc-5.5.0 | x86_64 | L1 | 14 | `f153b82121b0` | Sanitize gcc version header includes | v2.6.29-rc1 | compiler-h-gnuc-gt-4 |
| 2.6.37..2.6.39 | gcc-3.4.6 | i386 | L3 | 15 | `a45b0616e7ee` | Merge branch 'slab/next' into for-linus | v2.6.38-rc1 | none |
| 2.6.39..3.0 | gcc-4.8.5 | i386 | `arch/x86/kernel/ptrace.o` | 13 | `1b4ac2a935aa` | x86: Get rid of asmregparm | v3.0-rc1 | gcc48-ptrace-asmregparm |
| 3.17..3.18 | gcc-5.5.0 | x86_64 | L1 | 14 | `71458cfc782e` | kernel: add support for gcc 5 | v3.18-rc1 | no-compiler-header |
| 4.1..4.2 | gcc-6.5.0 | x86_64 | L1 | 13 | `cb984d101b30` | compiler-gcc: integrate the various compiler-gcc[345].h files | v4.2-rc1 | no-compiler-header, compiler-h-gnuc-gt-4 |
| 4.8..4.9 | debian-stretch-gcc-6 | x86_64 | `kernel/bounds.s` | 15 | `8ae94224c9d7` | kbuild: add -fno-PIE | v4.9-rc6 | distro-pie |
| 4.18..4.19 | gcc-4.5.4 | x86_64 | L1 | 14 | `cafa0010cd51` | Raise the minimum required gcc version to 4.6 | v4.19-rc1 | gcc-min-46 |
| 5.2..5.3 | ubuntu-eoan-gcc-9 | x86_64 | `scripts/mod/devicetable-offsets.s` | 14 | `29be86d7f9cb` | kbuild: add -fcf-protection=none when using retpoline flags | v5.3-rc1 | distro-cf-protection |
| 5.7..5.8 | gcc-4.8.5 | arm64 | L1 | 15 | `6ec4476ac825` | Raise gcc version requirement to 4.9 | v5.8-rc5 | gcc-min-49, arm-gcc47-ice-migrate, too-old-generic |
| 5.14..5.15 | gcc-4.9.4 | x86_64 | L1 | 14 | `76ae847497bc` | Documentation: raise minimum supported version of GCC to 5.1 | v5.15-rc2 | gcc-min-51 |
| 6.14..6.15 | gcc-5.5.0 | x86_64 | L1 | 14 | `a3e8fe814ad1` | x86/build: Raise the minimum GCC version to 8.1 | v6.15-rc1 | gcc-min-81 |
| 6.15..6.16 | gcc-5.5.0 | arm64 | L1 | 13 | `118c40b7b503` | kbuild: require gcc-8 and binutils-2.30 | v6.16-rc1 | gcc-min-81 |
