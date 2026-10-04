# Configuration differential

What the kernel's configuration notices about each GCC on the Current set (spec 11.3). For every kernel, the `.config` of each column is compared with the one of the next column that has a cell, and the symbols that change are gathered per step. `CC_VERSION_TEXT` and `GCC_VERSION` change at every step and are left out. A persona's `config-divergences.toml` in rucc-kernel must be a subset of these lines. Written by `gk publish`.

## arm64 defconfig+gk

| Step | Kernels | Symbols that change |
|---|--:|--:|
| 8.5.0 to 9.5.0 | 1 | 11 |
| 9.5.0 to 10.5.0 | 1 | 2 |
| 10.5.0 to 11.5.0 | 1 | 11 |
| 11.5.0 to 12.2.0 | 1 | 17 |
| 12.2.0 to 12.5.0 | 1 | 9 |
| 12.5.0 to 13.5.0 | 1 | 3 |
| 13.5.0 to 14.2.0 | 1 | 6 |
| 14.2.0 to 14.4.0 | 1 | 4 |
| 14.4.0 to 15.3.0 | 1 | 2 |
| 15.3.0 to 16.1.0 | 1 | 1 |
| 16.1.0 to 16.2.0 | 1 | 0 |

### 8.5.0 to 9.5.0

Over 7.2.8.

| Symbol | Kernels | 8.5.0 | 9.5.0 |
|---|---|---|---|
| ARM64_ERRATUM_4193714 | all | (absent) | y |
| ARM64_SME | all | (absent) | y |
| AS_HAS_MOPS | all | (absent) | y |
| AS_HAS_SME | all | (absent) | y |
| AS_VERSION | all | 23700 | 23900 |
| CC_HAS_BRANCH_PROT_PAC_RET | all | (absent) | y |
| CC_HAS_BRANCH_PROT_PAC_RET_BTI | all | (absent) | y |
| CC_HAVE_STACKPROTECTOR_SYSREG | all | (absent) | y |
| CC_NO_ARRAY_BOUNDS | all | (absent) | y |
| LD_VERSION | all | 23700 | 23900 |
| STACKPROTECTOR_PER_TASK | all | (absent) | y |

### 9.5.0 to 10.5.0

Over 7.2.8.

| Symbol | Kernels | 9.5.0 | 10.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 24100 |
| LD_VERSION | all | 23900 | 24100 |

### 10.5.0 to 11.5.0

Over 7.2.8.

| Symbol | Kernels | 10.5.0 | 11.5.0 |
|---|---|---|---|
| AS_VERSION | all | 24100 | 24301 |
| CC_HAS_ASM_GOTO_OUTPUT | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | (absent) | y |
| CC_HAS_KASAN_SW_TAGS | all | (absent) | y |
| CC_HAS_ZERO_CALL_USED_REGS | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | y | (absent) |
| HAVE_KCSAN_COMPILER | all | (absent) | y |
| LD_VERSION | all | 24100 | 24301 |
| RELR | all | (absent) | y |
| TOOLS_SUPPORT_RELR | all | (absent) | y |
| ZERO_CALL_USED_REGS | all | (absent) | n |

### 11.5.0 to 12.2.0

Over 7.2.8.

| Symbol | Kernels | 11.5.0 | 12.2.0 |
|---|---|---|---|
| ARCH_SUPPORTS_SHADOW_CALL_STACK | all | (absent) | y |
| AS_VERSION | all | 24301 | 23900 |
| CC_HAS_ASM_GOTO_OUTPUT | all | y | (absent) |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | y | (absent) |
| CC_HAS_AUTO_VAR_INIT_PATTERN | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO_BARE | all | (absent) | y |
| CC_HAVE_SHADOW_CALL_STACK | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | (absent) | y |
| INIT_STACK_ALL_PATTERN | all | (absent) | n |
| INIT_STACK_ALL_ZERO | all | (absent) | y |
| INIT_STACK_NONE | all | y | n |
| KCOV | all | (absent) | n |
| LD_VERSION | all | 24301 | 23900 |
| RELR | all | y | (absent) |
| SHADOW_CALL_STACK | all | (absent) | n |
| TOOLS_SUPPORT_RELR | all | y | (absent) |

### 12.2.0 to 12.5.0

Over 7.2.8.

| Symbol | Kernels | 12.2.0 | 12.5.0 |
|---|---|---|---|
| ARM64_LSUI | all | (absent) | y |
| AS_HAS_LSUI | all | (absent) | y |
| AS_VERSION | all | 23900 | 24500 |
| CC_HAS_ASM_GOTO_OUTPUT | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | y | (absent) |
| LD_VERSION | all | 23900 | 24500 |
| RELR | all | (absent) | y |
| TOOLS_SUPPORT_RELR | all | (absent) | y |

### 12.5.0 to 13.5.0

Over 7.2.8.

| Symbol | Kernels | 12.5.0 | 13.5.0 |
|---|---|---|---|
| AS_VERSION | all | 24500 | 20285426 |
| CC_HAS_ASSUME | all | (absent) | y |
| LD_VERSION | all | 24500 | 20285426 |

### 13.5.0 to 14.2.0

Over 7.2.8.

| Symbol | Kernels | 13.5.0 | 14.2.0 |
|---|---|---|---|
| ARM64_LSUI | all | y | (absent) |
| AS_HAS_LSUI | all | y | (absent) |
| AS_VERSION | all | 20285426 | 24301 |
| CC_HAS_MIN_FUNCTION_ALIGNMENT | all | (absent) | y |
| CC_HAS_SANE_FUNCTION_ALIGNMENT | all | (absent) | y |
| LD_VERSION | all | 20285426 | 24301 |

### 14.2.0 to 14.4.0

Over 7.2.8.

| Symbol | Kernels | 14.2.0 | 14.4.0 |
|---|---|---|---|
| ARM64_LSUI | all | (absent) | y |
| AS_HAS_LSUI | all | (absent) | y |
| AS_VERSION | all | 24301 | 20285426 |
| LD_VERSION | all | 24301 | 20285426 |

### 14.4.0 to 15.3.0

Over 7.2.8.

| Symbol | Kernels | 14.4.0 | 15.3.0 |
|---|---|---|---|
| CC_HAS_COUNTED_BY | all | (absent) | y |
| CC_HAS_MULTIDIMENSIONAL_NONSTRING | all | (absent) | y |

### 15.3.0 to 16.1.0

Over 7.2.8.

| Symbol | Kernels | 15.3.0 | 16.1.0 |
|---|---|---|---|
| CC_HAS_COUNTED_BY_PTR | all | (absent) | y |

## arm64 tinyconfig+gk

No kernel has two columns to compare yet.

## i386 defconfig+gk

| Step | Kernels | Symbols that change |
|---|--:|--:|
| 8.5.0 to 9.5.0 | 1 | 4 |
| 9.5.0 to 10.5.0 | 1 | 2 |
| 10.5.0 to 11.5.0 | 1 | 9 |
| 11.5.0 to 12.2.0 | 1 | 11 |
| 12.2.0 to 12.5.0 | 1 | 5 |
| 12.5.0 to 13.5.0 | 1 | 3 |
| 13.5.0 to 14.2.0 | 1 | 4 |
| 14.2.0 to 14.4.0 | 1 | 2 |
| 14.4.0 to 15.3.0 | 1 | 2 |
| 15.3.0 to 16.1.0 | 1 | 1 |
| 16.1.0 to 16.2.0 | 1 | 0 |

### 8.5.0 to 9.5.0

Over 7.2.8.

| Symbol | Kernels | 8.5.0 | 9.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23700 | 23900 |
| CC_NO_ARRAY_BOUNDS | all | (absent) | y |
| LD_VERSION | all | 23700 | 23900 |
| TOOLS_SUPPORT_RELR | all | (absent) | y |

### 9.5.0 to 10.5.0

Over 7.2.8.

| Symbol | Kernels | 9.5.0 | 10.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 24100 |
| LD_VERSION | all | 23900 | 24100 |

### 10.5.0 to 11.5.0

Over 7.2.8.

| Symbol | Kernels | 10.5.0 | 11.5.0 |
|---|---|---|---|
| AS_VERSION | all | 24100 | 24301 |
| CC_HAS_ASM_GOTO_OUTPUT | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | (absent) | y |
| CC_HAS_SLS | all | (absent) | y |
| CC_HAS_ZERO_CALL_USED_REGS | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | y | (absent) |
| HAVE_KCSAN_COMPILER | all | (absent) | y |
| LD_VERSION | all | 24100 | 24301 |
| ZERO_CALL_USED_REGS | all | (absent) | n |

### 11.5.0 to 12.2.0

Over 7.2.8.

| Symbol | Kernels | 11.5.0 | 12.2.0 |
|---|---|---|---|
| AS_VERSION | all | 24301 | 23900 |
| CC_HAS_ASM_GOTO_OUTPUT | all | y | (absent) |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | y | (absent) |
| CC_HAS_AUTO_VAR_INIT_PATTERN | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO_BARE | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | (absent) | y |
| INIT_STACK_ALL_PATTERN | all | (absent) | n |
| INIT_STACK_ALL_ZERO | all | (absent) | y |
| INIT_STACK_NONE | all | y | n |
| LD_VERSION | all | 24301 | 23900 |

### 12.2.0 to 12.5.0

Over 7.2.8.

| Symbol | Kernels | 12.2.0 | 12.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 24500 |
| CC_HAS_ASM_GOTO_OUTPUT | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | y | (absent) |
| LD_VERSION | all | 23900 | 24500 |

### 12.5.0 to 13.5.0

Over 7.2.8.

| Symbol | Kernels | 12.5.0 | 13.5.0 |
|---|---|---|---|
| AS_VERSION | all | 24500 | 20285426 |
| CC_HAS_ASSUME | all | (absent) | y |
| LD_VERSION | all | 24500 | 20285426 |

### 13.5.0 to 14.2.0

Over 7.2.8.

| Symbol | Kernels | 13.5.0 | 14.2.0 |
|---|---|---|---|
| AS_VERSION | all | 20285426 | 24301 |
| CC_HAS_MIN_FUNCTION_ALIGNMENT | all | (absent) | y |
| CC_HAS_SANE_FUNCTION_ALIGNMENT | all | (absent) | y |
| LD_VERSION | all | 20285426 | 24301 |

### 14.2.0 to 14.4.0

Over 7.2.8.

| Symbol | Kernels | 14.2.0 | 14.4.0 |
|---|---|---|---|
| AS_VERSION | all | 24301 | 20285426 |
| LD_VERSION | all | 24301 | 20285426 |

### 14.4.0 to 15.3.0

Over 7.2.8.

| Symbol | Kernels | 14.4.0 | 15.3.0 |
|---|---|---|---|
| CC_HAS_COUNTED_BY | all | (absent) | y |
| CC_HAS_MULTIDIMENSIONAL_NONSTRING | all | (absent) | y |

### 15.3.0 to 16.1.0

Over 7.2.8.

| Symbol | Kernels | 15.3.0 | 16.1.0 |
|---|---|---|---|
| CC_HAS_COUNTED_BY_PTR | all | (absent) | y |

## x86_64 defconfig+gk

| Step | Kernels | Symbols that change |
|---|--:|--:|
| 8.5.0 to 9.5.0 | 3 | 4 |
| 9.5.0 to 10.5.0 | 3 | 2 |
| 10.5.0 to 11.5.0 | 3 | 11 |
| 11.5.0 to 12.2.0 | 3 | 11 |
| 12.2.0 to 12.5.0 | 3 | 5 |
| 12.5.0 to 13.5.0 | 3 | 3 |
| 13.5.0 to 14.2.0 | 2 | 5 |
| 14.2.0 to 14.4.0 | 2 | 2 |
| 14.4.0 to 15.3.0 | 2 | 2 |
| 15.3.0 to 16.1.0 | 2 | 1 |
| 16.1.0 to 16.2.0 | 2 | 0 |

### 8.5.0 to 9.5.0

Over 6.12.111, 6.18.54 and 7.2.8.

| Symbol | Kernels | 8.5.0 | 9.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23700 | 23900 |
| CC_NO_ARRAY_BOUNDS | all | (absent) | y |
| LD_VERSION | all | 23700 | 23900 |
| TOOLS_SUPPORT_RELR | all | (absent) | y |

### 9.5.0 to 10.5.0

Over 6.12.111, 6.18.54 and 7.2.8.

| Symbol | Kernels | 9.5.0 | 10.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 24100 |
| LD_VERSION | all | 23900 | 24100 |

### 10.5.0 to 11.5.0

Over 6.12.111, 6.18.54 and 7.2.8.

| Symbol | Kernels | 10.5.0 | 11.5.0 |
|---|---|---|---|
| AS_VERSION | all | 24100 | 24301 |
| CC_HAS_ASM_GOTO_OUTPUT | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | (absent) | y |
| CC_HAS_SLS | all | (absent) | y |
| CC_HAS_ZERO_CALL_USED_REGS | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | y | (absent) |
| HAVE_KCSAN_COMPILER | all | (absent) | y |
| KCSAN | all | (absent) | n |
| LD_VERSION | all | 24100 | 24301 |
| MITIGATION_SLS | all | (absent) | n |
| ZERO_CALL_USED_REGS | all | (absent) | n |

### 11.5.0 to 12.2.0

Over 6.12.111, 6.18.54 and 7.2.8.

| Symbol | Kernels | 11.5.0 | 12.2.0 |
|---|---|---|---|
| AS_VERSION | all | 24301 | 23900 |
| CC_HAS_ASM_GOTO_OUTPUT | all | y | (absent) |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | y | (absent) |
| CC_HAS_AUTO_VAR_INIT_PATTERN | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO_BARE | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | (absent) | y |
| INIT_STACK_ALL_PATTERN | all | (absent) | n |
| INIT_STACK_ALL_ZERO | all | (absent) | y |
| INIT_STACK_NONE | all | y | n |
| LD_VERSION | all | 24301 | 23900 |

### 12.2.0 to 12.5.0

Over 6.12.111, 6.18.54 and 7.2.8.

| Symbol | Kernels | 12.2.0 | 12.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 24500 |
| CC_HAS_ASM_GOTO_OUTPUT | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | y | (absent) |
| LD_VERSION | all | 23900 | 24500 |

### 12.5.0 to 13.5.0

Over 6.12.111, 6.18.54 and 7.2.8.

| Symbol | Kernels | 12.5.0 | 13.5.0 |
|---|---|---|---|
| AS_VERSION | all | 24500 | 20285426 |
| CC_HAS_ASSUME | 6.18.54 and 7.2.8 | (absent) | y |
| LD_VERSION | all | 24500 | 20285426 |

### 13.5.0 to 14.2.0

Over 6.18.54 and 7.2.8.

| Symbol | Kernels | 13.5.0 | 14.2.0 |
|---|---|---|---|
| AS_VERSION | all | 20285426 | 24301 |
| CC_HAS_KASAN_SW_TAGS | all | (absent) | y |
| CC_HAS_MIN_FUNCTION_ALIGNMENT | all | (absent) | y |
| CC_HAS_SANE_FUNCTION_ALIGNMENT | all | (absent) | y |
| LD_VERSION | all | 20285426 | 24301 |

### 14.2.0 to 14.4.0

Over 6.18.54 and 7.2.8.

| Symbol | Kernels | 14.2.0 | 14.4.0 |
|---|---|---|---|
| AS_VERSION | all | 24301 | 20285426 |
| LD_VERSION | all | 24301 | 20285426 |

### 14.4.0 to 15.3.0

Over 6.18.54 and 7.2.8.

| Symbol | Kernels | 14.4.0 | 15.3.0 |
|---|---|---|---|
| CC_HAS_COUNTED_BY | all | (absent) | y |
| CC_HAS_MULTIDIMENSIONAL_NONSTRING | all | (absent) | y |

### 15.3.0 to 16.1.0

Over 6.18.54 and 7.2.8.

| Symbol | Kernels | 15.3.0 | 16.1.0 |
|---|---|---|---|
| CC_HAS_COUNTED_BY_PTR | 7.2.8 | (absent) | y |
