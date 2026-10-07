# Bisections

Every `fixed-by` commit of the failure catalog is confirmed by bisecting between a kernel the signature matches and one it does not, with the signature's GCC (spec 03.5 and 08.2). When the bisection finds another commit, the bisected commit wins and the signature is corrected. A signature can name several commits, one for each place the kernel had to change, and a bisection confirms the one its range and platform reach. Written by `gk publish` from `signatures.toml` and the bisections in the result store.

5 bisections in the store. 3 of the 53 signatures with a fixing commit have at least one of their commits confirmed.

## Signatures

| Signature | Rung | Kind | Fixing commits | Confirmed |
|---|---|---|---|---|
| gcc-min-32 | L1..L3 | refusal | `a136564` |  |
| gcc-296-frame-pointer | L1 | refusal | `a136564` |  |
| gcc3-0-1-refused | L1 | refusal | `6680598` |  |
| arm-gcc3-below-33 | L1 | refusal | `a136564` |  |
| gcc41-weak | L1 | blacklist | `f9d1425` |  |
| no-compiler-header | L1 | refusal | `cb984d101b30` `71458cfc782e` | `cb984d101b30` `71458cfc782e` |
| compiler-h-gnuc-gt-4 | L1..L3 | refusal | `f153b82121b0` `cb984d101b30` | `f153b82121b0` `cb984d101b30` |
| arm-gcc48-pr58854 | L1 | blacklist | `7fc150543c73` |  |
| gcc-min-46 | L1 | refusal | `cafa0010cd51` | `cafa0010cd51` |
| gcc-min-49 | L1 | refusal | `6ec4476ac825` |  |
| gcc-min-51 | L1 | refusal | `76ae847497bc` |  |
| gcc-min-81 | L1 | refusal | `a3e8fe814ad1` `118c40b7b503` |  |
| asm-goto-miscompile | L5..L7 | miscompile | `3f0116c` `a9f180345f53` |  |
| gcc49-load-balance | L5..L7 | miscompile | `2062afb4f804` |  |
| gcc10-start-secondary | L5 | miscompile-by-assumption | `a9a3ed1eff36` |  |
| gcc15-union-padding | L5..L7 | miscompile-by-assumption | `dce4aab8441d` |  |
| arm-gcc47-ice-migrate | L3 | ice | `6ec4476ac825` |  |
| too-old-asm-goto | L3 | too-old | `e501ce957a78` |  |
| too-old-generic | L3 | too-old | `6ec4476ac825` |  |
| distro-ssp | L3 | default-change | `34c162f79e37` `eb2cafa1d902` |  |
| distro-pie | L3 | default-change | `8ae94224c9d7` `c6a385539175` |  |
| distro-cf-protection | L3 | default-change | `29be86d7f9cb` |  |
| fno-common | L3 | default-change | `e33a814e772c` |  |
| gnu11-extern-inline | L3 | default-change | `14bfc987e395` `8f375e10ee47` `51b97e354ba9` |  |
| gnu23-bool | L3 | default-change | `b3bee1e7c3f2` `ee2ab467bddf` `8ba14d9f490a` `3b8b80e99376` `947d5d036c78` `5a821e2d69e2` `7cbb015e2d3d` `0f4ae7c6ecb8` |  |
| gcc46-m-elf-i386 | L3 | new-error | `de2a8cf98ecd` |  |
| gcc44-setup-dil | L3 | new-error | `811a0fff5d6e` |  |
| gcc7-format-werror | L3 | new-warning-werror | `bd664f6b3e37` |  |
| gcc8-packed-not-aligned | L3 | new-warning-werror | `321cb0308a9e` |  |
| gcc8-attribute-alias | L3 | new-warning-werror | `bee20031772a` |  |
| gcc8-stringop-truncation | L3 | new-warning-werror | `217c3e019675` |  |
| gcc9-missing-attributes | L3 | new-warning-werror | `c0d9782f5b6d` `a6e60d84989f` |  |
| gcc9-packed-member | L3 | new-warning-werror | `6f303d60534c` |  |
| gcc10-warnings-werror | L3 | new-warning-werror | `5c45de21a222` `5a76021c2eff` |  |
| gcc11-stringop-overread | L3 | new-warning-werror | `e7c6e405e171` |  |
| gcc12-array-bounds | L3 | new-warning-werror | `f0be87c42cbd` |  |
| gcc12-dangling-pointer | L3 | new-warning-werror | `49beadbd47c2` |  |
| gcc13-enum-type | L3 | new-error | `525ff9c29657` |  |
| gcc15-unterminated-string | L3 | new-warning-werror | `d5d45a7f2619` `9d7a0577c9db` |  |
| gcc16-unused-but-set | L3 | new-warning-werror | `97fb54d86d21` `729a2e8e9ac4` |  |
| gcc8-objtool-cold | L4..L8 | objtool | `13810435b9a7` |  |
| gcc8-objtool-switch | L4..L8 | objtool | `fd35c88b7441` |  |
| gcc16-modpost-ipa-init | L4 | modpost | `4c9ad387aa2d` |  |
| binutils-segment-mov | L3 | binutils-assembler | `fd51f666fa59` |  |
| binutils-range-ok-cmp | L3 | binutils-assembler | `722f4f5b2600` |  |
| binutils-plt32 | L4..L5 | binutils-relocation | `b21ebf2fb4cd` |  |
| binutils-separate-code | L5 | binutils-layout | `e3d03598e8ae` |  |
| binutils-got32x | L5 | binutils-relocation | `6d92bc9d483a` |  |
| binutils-objtool-symtab | L3 | binutils-objtool | `1d489151e9f9` |  |
| binutils-riscv-zicsr | L3 | binutils-isa | `6df2a016c0c8` |  |
| binutils-rwx-warning | L4 | binutils-warning | `0d362be5b142` `ffcf9c5700e4` |  |
| binutils-loongarch-relax | L5..L7 | binutils-relocation | `03c53eb90c0c` |  |
| binutils-ppc-ztext | L4 | binutils-link | `97f902dd4c99` |  |

## Runs

| Range | GCC | Platform | Judged by | Steps | First commit | Subject | In | Signature |
|---|---|---|---|--:|---|---|---|---|
| 2.6.28..2.6.29 | gcc-5.5.0 | x86_64 | L1 | 14 | `f153b82121b0` | Sanitize gcc version header includes | v2.6.29-rc1 | compiler-h-gnuc-gt-4 |
| 3.17..3.18 | gcc-5.5.0 | x86_64 | L1 | 14 | `71458cfc782e` | kernel: add support for gcc 5 | v3.18-rc1 | no-compiler-header |
| 4.1..4.2 | gcc-6.5.0 | x86_64 | L1 | 13 | `cb984d101b30` | compiler-gcc: integrate the various compiler-gcc[345].h files | v4.2-rc1 | no-compiler-header, compiler-h-gnuc-gt-4 |
| 4.18..4.19 | gcc-4.5.4 | x86_64 | L1 | 14 | `cafa0010cd51` | Raise the minimum required gcc version to 4.6 | v4.19-rc1 | gcc-min-46 |
| 5.2..5.3 | ubuntu-eoan-gcc-9 | x86_64 | L3 | 14 | `46f5c0cc3af0` | Merge branch 'perf-urgent-for-linus' of git://git.kernel.org/pub/scm/linux/kernel/git/tip/tip | v5.3-rc1 | none |
