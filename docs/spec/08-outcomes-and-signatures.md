# Outcomes, classification and the failure catalog

A verdict says how far a cell got. That alone does not make the matrix useful. What makes it useful is knowing why a cell stopped, whether that reason is the same one seen in a hundred other cells, and which kernel commit made it go away. This document fixes how a failure is reduced to a first error, how first errors are classified into signatures, what the catalog contains at the start, and how holes are handled.

## 8.1 From a log to a first error

`gk-run` records, for every failed rung:
- **the rung**, L1 to L8;
- **the failing units**, from `compile.jsonl` (every `gk-cc` invocation with its exit status) and from `make`'s own error lines for units that are not C (assembler files, linker steps, host programs, scripts);
- **the first error**, the first diagnostic of severity error (or fatal, or an ICE) in the first failing unit in build order, normalized: paths made relative to the tree, line and column numbers kept, addresses and temporary file names removed;
- for L5 to L8, **the first bad console line**: the first line matching the splat patterns (`Kernel panic`, `BUG:`, `Oops`, `general protection fault`, `WARNING: CPU:`, `Unable to handle kernel`, `stack-protector:`, `Kernel stack is corrupted`, `KASAN:`, `UBSAN:`, `rcu: INFO: rcu_.* detected stalls`, `watchdog: BUG: soft lockup`) or, for a hang, the last line and the time it was printed.

L1 failures are special. They are the kernel refusing the compiler, and the first error is the refusal: an `#error` text from a compiler header, the `cc-version.sh` message, or a missing `compiler-gccN.h`.

## 8.2 Signatures

A signature is a named pattern over the first error and its context. `signatures.toml` holds them:

```toml
[[signature]]
class = "no-compiler-header"
rung = "L1"
match = { first-error = 'linux/compiler-gcc(\d+)\.h: No such file or directory' }
kind = "refusal"
summary = "Kernels 2.6.29 to 4.1 include one header per GCC major version and have none for this one"
fixed-by = [
  { commit = "cb984d101b30", first-tag = "v4.2-rc1", stable = ["3.10"] },
  { commit = "71458cfc782e", first-tag = "v3.18-rc1", note = "adds compiler-gcc5.h only", stable = ["3.10", "3.13"] },
]
gcc = ">=5"

[[signature]]
class = "gcc10-start-secondary"
rung = "L5"
match = { console = 'stack-protector: Kernel stack is corrupted in: start_secondary' }
kind = "miscompile-by-assumption"
summary = "GCC 10 tail-calls cpu_startup_entry() after the stack canary is set up"
fixed-by = [ { commit = "a9a3ed1eff36", first-tag = "v5.7", stable = ["4.4", "4.9", "4.14", "4.19", "5.4"] } ]
gcc = ">=10"
platform = ["x86_64", "i386"]
```

Fields:
- `class`, a short name, unique, used everywhere a failure is named;
- `rung`, where it shows;
- `match`, regular expressions over `first-error`, `unit`, `console`, `config` or `command`, all of which must match;
- `kind`, one of the kinds of 8.3;
- `summary`, one sentence;
- `fixed-by`, the kernel commits that made it go away, with first tag and stable branches, as in 01.4;
- `gcc`, `binutils`, `platform`, `kernel`, ranges that limit where the signature can apply. A match outside them is flagged as a new finding, not classified, when no other signature classifies the cell.

`gk classify` runs every signature over every failed cell and writes `classes` into `cell.json`. Where a cell stored a failing unit with no error line, as cells run before gas's `Error:` lines counted did, and its compile.jsonl now finds one, it writes errors.jsonl again too, so a copy of the store without compile.jsonl still has the line. A cell may have more than one class when its `-k` run shows several (09.5), but its first error has exactly one or none. `gk triage` lists unclassified first errors, clustered by normalized text, largest cluster first. `gk explain K G P` prints the cell's class, the signature's summary and fix, and whether the fix is in the tree: a fix that is in the tree but the cell still fails means the signature is wrong or there is a second problem.

## 8.3 Kinds

| Kind | Meaning | Example |
|---|---|---|
| `refusal` | the kernel refuses the compiler on purpose | `#error` for GCC 3.0 in 2.6.29, `cc-version.sh` below 5.1 in 5.15 |
| `blacklist` | a refusal naming a known bad version | GCC 4.1.0 `__weak`, arm 4.8.0 to 4.8.2 |
| `new-error` | a new GCC turns something into an error | GCC 14's six errors, GCC 15's `bool` keyword |
| `new-warning-werror` | a new GCC warning, fatal because the unit uses `-Werror` | GCC 7's format warnings under an arch `-Werror` |
| `default-change` | a changed compiler default | `-fno-common` (GCC 10), `-std=gnu23` (GCC 15), PIE (distribution GCC 6) |
| `miscompile` | GCC generates wrong code, the kernel builds and misbehaves | 4.9.0 `load_balance()`, 4.8 `asm goto` |
| `miscompile-by-assumption` | GCC generates correct code, the kernel relied on something GCC never promised | GCC 10 `start_secondary`, GCC 15 union `{0}` padding |
| `ice` | the compiler crashes | arm 4.7 in `mm/migrate.c` |
| `objtool` | objtool rejects the object | GCC 8 `.cold` subfunctions, GCC 14.2 `ENDBR` |
| `modpost` | section or symbol checks fail | GCC 16.1 IPA inlining of `__init` callees |
| `too-old` | an old GCC lacks a feature the kernel uses without a probe | `asm goto` before 4.5, `_Generic` before 4.9 |
| `binutils-*` | the assembler or linker, not GCC | binutils 2.39 RWX segment warnings |
| `host-tool` | a host program failed. Should not happen in the matrix (05.1). A cell with this kind means the era host is wrong | make 3.82 on 2.6.35 |
| `emulator` | the boot failed for a reason the L5k twin removes | wrong `-cpu` for the kernel's age |
| `timeout` | over budget with no other signal | |

The kind decides what a failure means for rucc (11.6). A `refusal` or `blacklist` must be reproduced under the persona. A `miscompile` must not be. A `new-error` or `default-change` must be reproduced at the version it appeared. A `host-tool` or `emulator` kind is a bug in gcc-kernel, not a finding.

## 8.4 The seed catalog

The catalog starts with every breakage of document 01 as a signature. G2's exit requires them all, with regular expressions tested against real cells. The seed, grouped by GCC:

| Class | Kind | GCC | Rung | Fixed by |
|---|---|---|---|---|
| `gcc-too-old-295` | refusal | below 2.95 | L1 to L3 | none, the refusal is the policy (2.6.0 to 2.6.15) |
| `gcc-296-frame-pointer` | refusal | 2.96 | L1 | removed with gcc-2 checks, a136564 |
| `gcc-min-32` | refusal | below 3.2 on 2.6.16 to 2.6.28 | L1 to L3 | policy from a136564 |
| `gcc-below-min-on-26` | too-old | below 2.95 on 2.6.0 to 2.6.15, below 3.2 from 2.6.16 | L3 | none, it stops before the refusal is compiled |
| `gcc-pre-27-on-20` | too-old | below 2.7 on 2.0 | L3 | none |
| `ifdef-0-in-10` | new-error | 3 and later on 1.0 | L3 | none, gone by 1.2 |
| `gcc3-0-1-refused` | refusal | 3.0, 3.1 | L1 | policy from 6680598 |
| `arm-gcc3-below-33` | refusal | 3.0 to 3.2, arm | L1 | policy from a136564 |
| `gcc4-incomplete-array` | new-error | 4 and later on 2.6.0 to 2.6.8, x86 | L3 | none, fixed before git in 2.6.9 |
| `gcc4-byteorder` | new-error | 4 and later on 2.6.0 and 2.6.1, x86 | L1 to L3 | none, fixed before git in 2.6.2 |
| `gcc41-weak` | blacklist | 4.1.0, 4.1.1 | L1 | policy from f9d1425 |
| `no-compiler-header` | refusal | 5 and later on 2.6.29 to 4.1 | L1 | cb984d101b30, 71458cfc782e |
| `compiler-h-gnuc-gt-4` | refusal | 5 and later on 2.6.12 to 2.6.28 | L1 to L3 | f153b82 changes the form, cb984d101b30 ends it |
| `asm-goto-miscompile` | miscompile | 4.5 to 4.8.1 | L5 to L7 | 3f0116c, a9f180345f53 |
| `gcc46-m-elf-i386` | new-error | 4.6 and later on kernels before 2.6.37, i386 and x86_64 | L3 | de2a8cf98ecd |
| `gcc44-setup-dil` | new-error | 4.4 and later on 2.6.23 and 2.6.24 | L3 | 811a0fff5d6e |
| `gcc43-no-unit-at-a-time` | new-error | 4.3 and later, i386, kernels before 2.6.16 | L3 | 9ab34fe76114 |
| `arm-gcc48-pr58854` | blacklist | 4.8.0 to 4.8.2, arm | L1 | policy from 7fc150543c73, removed in 5.8 |
| `arm-gcc47-ice-migrate` | ice | 4.7 to 4.8, arm | L3 | `ICE_noinline`, removed in 5.8 |
| `gcc49-load-balance` | miscompile | 4.9.0, 4.9.1 | L5 to L7 | 2062afb4f804 |
| `gcc-min-46` | refusal | below 4.6 | L1 | policy from cafa0010cd51, 4.19 |
| `gcc-min-49` | refusal | below 4.9 | L1 | policy from 6ec4476ac825, 5.8 |
| `gcc-min-51` | refusal | below 5.1 | L1 | policy from 76ae847497bc, 5.15 |
| `gcc-min-81` | refusal | below 8.1 | L1 | policy from a3e8fe814ad1 (x86, 6.15), 118c40b7b503 (all, 6.16) |
| `gnu11-extern-inline` | default-change | 5, kernels before 3.11 | L3 | 14bfc987e395 for tty in 2.6.28, 8f375e10ee47 for i915, 51b97e354ba9 for the whole tree |
| `distro-ssp` | default-change | Ubuntu 4.1 and later, kernels before 2.6.18 | L3 | eb2cafa1d902, after 34c162f79e37 missed Ubuntu |
| `distro-pie` | default-change | Debian and Ubuntu 6 and later | L3 | 8ae94224c9d7, c6a385539175 |
| `gcc7-format-werror` | new-warning-werror | 7 | L3 | bd664f6b3e37 |
| `gcc8-packed-not-aligned` | new-warning-werror | 8 | L3 | 321cb0308a9e |
| `gcc8-attribute-alias` | new-warning-werror | 8 | L3 | bee20031772a |
| `gcc8-stringop-truncation` | new-warning-werror | 8 | L3 | 217c3e019675 |
| `gcc8-objtool-cold` | objtool | 8 | L4 | 13810435b9a7 |
| `gcc8-objtool-switch` | objtool | 8 | L4 | fd35c88b7441 |
| `gcc9-missing-attributes` | new-warning-werror | 9 | L3 | c0d9782f5b6d, a6e60d84989f |
| `gcc9-packed-member` | new-warning-werror | 9 | L3 | 6f303d60534c |
| `fno-common` | default-change | 10, host side | L3 | e33a814e772c, and the tools fixes of 01.4 |
| `gcc10-start-secondary` | miscompile-by-assumption | 10 | L5 | a9a3ed1eff36 |
| `gcc10-warnings-werror` | new-warning-werror | 10 | L3 | 5c45de21a222, 5a76021c2eff |
| `gcc11-stringop-overread` | new-warning-werror | 11 | L3 | e7c6e405e171 |
| `gcc12-array-bounds` | new-warning-werror | 12 | L3 | f0be87c42cbd |
| `gcc12-dangling-pointer` | new-warning-werror | 12 | L3 | 49beadbd47c2 |
| `gcc13-enum-type` | new-error | 13, 32-bit | L3 | 525ff9c29657 |
| `gcc14-new-errors` | new-error | 14 | L3 | per unit |
| `gcc14-objtool-endbr` | objtool | 14.2 | L4 | none in the kernel, GCC PR 116174 |
| `gnu23-bool` | default-change | 15 | L3 | b3bee1e7c3f2, ee2ab467bddf, 8ba14d9f490a, 3b8b80e99376, 947d5d036c78, 5a821e2d69e2, 7cbb015e2d3d, 0f4ae7c6ecb8 |
| `gcc15-union-padding` | miscompile-by-assumption | 15 | L5 to L7, if anywhere | dce4aab8441d, c15253494fd9 |
| `gcc15-unterminated-string` | new-warning-werror | 15 | L3 | d5d45a7f2619, 9d7a0577c9db |
| `gcc16-modpost-ipa-init` | modpost | 16 | L4 | 4c9ad387aa2d |
| `gcc16-unused-but-set` | new-warning-werror | 16 | L3 | 97fb54d86d21, 729a2e8e9ac4 |
| `binutils-too-old-lfence` | binutils-* | binutils below 2.12, i386 | L3 | none, the kernel asks for 2.12 |
| `binutils-too-old-sysexit` | binutils-* | binutils 2.14, x86_64 | L3 | none |
| `binutils-too-old-bignum` | binutils-* | binutils 2.14, i386, s2io and aic79xx | L3 | none |
| `binutils-x86-64-macro-redefined` | binutils-* | binutils 2.16 and later, x86_64, kernels before 2.6.11 | L3 | none, fixed before git |
| `binutils-segment-mov` | binutils-* | binutils 2.17 and later, x86, kernels before 2.6.12 | L3 | fd51f666fa59 |
| `binutils-range-ok-cmp` | binutils-* | binutils 2.20 and later, i386, kernels before 2.6.18 | L3 | 722f4f5b2600 |
| `binutils-plt32` | binutils-* | binutils 2.31 and later, x86_64 | L4 to L5 | b21ebf2fb4cd |
| `binutils-separate-code` | binutils-* | binutils 2.31 and later, x86_64 | L5 | e3d03598e8ae |
| `binutils-got32x` | binutils-* | binutils 2.26 and later, i386 | L5 | 6d92bc9d483a |
| `binutils-objtool-symtab` | binutils-* | binutils 2.36 and later, x86, kernels before 5.11 | L3 | 1d489151e9f9 |
| `binutils-objtool-align` | binutils-* | binutils 2.45 and later, x86_64, kernels built on the buster and bullseye hosts | L3 | none yet, the host libelf |
| `binutils-riscv-zicsr` | binutils-* | binutils 2.38 and later, riscv | L3 | 6df2a016c0c8 |
| `binutils-rwx-warning` | binutils-* | binutils 2.39 and later | L4, fatal with `CONFIG_WERROR` from 6.18 | 0d362be5b142, ffcf9c5700e4 |
| `binutils-loongarch-relax` | binutils-* | binutils 2.41 and later, loongarch | L5 to L7 (module load) | 03c53eb90c0c |
| `binutils-ppc-ztext` | binutils-* | binutils 2.46 and later, ppc | L4 | 97f902dd4c99 |
| `too-old-asm-goto` | too-old | below 4.5 | L3 | the kernel's `CC_HAS_ASM_GOTO` probe and, from 4.20 on x86, the requirement |
| `too-old-generic` | too-old | below 4.9 | L3 | policy from 6ec4476ac825 |

The catalog grows as `gk triage` clusters are named. Every new signature needs a cell that it matches, and its `fixed-by`, when it has one, needs `gk bisect-kernel` to confirm it: the commit is the one the bisection finds, not the one a mailing list post claims.

## 8.5 Holes

A hole is a column inside W(K, P)'s range that is not **works** while its neighbours on both sides are. Holes are the interesting cells: they are where a single GCC release broke a kernel that the releases around it built and ran. Every hole:

1. turns its row dense for the run (09.3), because one hole suggests more;
2. is re-run three times from a fresh scratch directory, so that a non-deterministic build or an emulator hang is not recorded as a hole;
3. gets a `-k` run and full logs kept;
4. is classified or triaged like any failure;
5. is listed in `reports/holes.md`, with its class, and, where the class is `miscompile`, the GCC bug number when one is known.

Known holes that the sweep must find, as a check that it finds holes at all: 4.1.0 and 4.1.1 on 2.6.29 and later (refused), arm 4.8.0 to 4.8.2 on 3.18 to 5.7 (refused), and 4.9.0 and 4.9.1 on kernels before 3.16 (not refused, and whether the miscompile shows at run time is an open question the matrix answers). If the sweep does not find the first two, the search is broken.

## 8.6 What gets published per failure

In `matrix.json`, per failed cell: rung, class or `unclassified`, the first error truncated to 200 characters, and `fixed-by` from the signature. In the per-kernel and per-GCC reports, each edge is one line: "4.9.337 × GCC 15.3 on x86_64: fails L3, `gnu23-bool`, fixed in 6.14 by ee2ab467bddf, not backported to 4.9." That line is the unit of knowledge the repository produces.
