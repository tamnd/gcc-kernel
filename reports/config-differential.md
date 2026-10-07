# Configuration differential

What the kernel's configuration notices about each GCC on the Current set (spec 11.3). For every kernel, the `.config` of each column is compared with the one of the next column that has a cell, and the symbols that change are gathered per step. `CC_VERSION_TEXT` and `GCC_VERSION` change at every step and are left out. A persona's `config-divergences.toml` in rucc-kernel must be a subset of these lines. Written by `gk publish`.

## arm64 defconfig+gk

No kernel has two columns to compare yet.

## arm64 tinyconfig+gk

No kernel has two columns to compare yet.

## i386 defconfig+gk

No kernel has two columns to compare yet.

## x86_64 defconfig+gk

| Step | Kernels | Symbols that change |
|---|--:|--:|
| 8.5.0 to 9.5.0 | 1 | 4 |
| 9.5.0 to 10.5.0 | 1 | 2 |
| 10.5.0 to 11.5.0 | 1 | 11 |
| 10.5.0 to 16.2.0 | 2 | 17 |
| 11.5.0 to 12.2.0 | 1 | 11 |
| 12.2.0 to 12.5.0 | 1 | 5 |
| 12.2.0 to 16.2.0 | 3 | 10 |
| 12.5.0 to 13.5.0 | 1 | 3 |
| 13.5.0 to 14.2.0 | 1 | 5 |
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
| KCSAN | all | (absent) | n |
| LD_VERSION | all | 24100 | 24301 |
| MITIGATION_SLS | all | (absent) | n |
| ZERO_CALL_USED_REGS | all | (absent) | n |

### 10.5.0 to 16.2.0

Over 5.10.271 and 5.15.222.

| Symbol | Kernels | 10.5.0 | 16.2.0 |
|---|---|---|---|
| AS_VERSION | all | 24100 | 20285426 |
| CC_HAS_ASM_GOTO_OUTPUT | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_PATTERN | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO_BARE | all | (absent) | y |
| CC_HAS_KASAN_SW_TAGS | all | (absent) | y |
| CC_HAS_SLS | all | (absent) | y |
| CC_HAS_ZERO_CALL_USED_REGS | 5.15.222 | (absent) | y |
| HAVE_KCSAN_COMPILER | all | (absent) | y |
| INIT_STACK_ALL_PATTERN | all | (absent) | n |
| INIT_STACK_ALL_ZERO | all | (absent) | y |
| INIT_STACK_NONE | all | y | n |
| KCSAN | all | (absent) | n |
| LD_VERSION | all | varies | varies |
| SLS | all | (absent) | n |
| ZERO_CALL_USED_REGS | 5.15.222 | (absent) | n |

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

### 12.2.0 to 16.2.0

Over 6.1.189, 6.6.158 and 6.12.112.

| Symbol | Kernels | 12.2.0 | 16.2.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 20285426 |
| CC_HAS_ASM_GOTO_OUTPUT | 6.12.112 | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | 6.12.112 | (absent) | y |
| CC_HAS_COUNTED_BY | 6.6.158 and 6.12.112 | (absent) | y |
| CC_HAS_KASAN_SW_TAGS | all | (absent) | y |
| CC_HAS_MIN_FUNCTION_ALIGNMENT | 6.12.112 | (absent) | y |
| CC_HAS_SANE_FUNCTION_ALIGNMENT | 6.12.112 | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | 6.12.112 | y | (absent) |
| GCC_ASM_GOTO_OUTPUT_WORKAROUND | 6.1.189 and 6.6.158 | y | (absent) |
| LD_VERSION | all | 23900 | 20285426 |

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
| CC_HAS_KASAN_SW_TAGS | all | (absent) | y |
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

## x86_64 tinyconfig+gk

| Step | Kernels | Symbols that change |
|---|--:|--:|
| 8.5.0 to 9.5.0 | 1 | 4 |
| 9.5.0 to 10.5.0 | 1 | 2 |
| 10.5.0 to 11.5.0 | 1 | 10 |
| 11.5.0 to 12.2.0 | 1 | 11 |
| 12.2.0 to 12.5.0 | 1 | 5 |
| 12.5.0 to 13.5.0 | 1 | 3 |
| 13.5.0 to 14.2.0 | 1 | 5 |

### 8.5.0 to 9.5.0

Over 7.2.9.

| Symbol | Kernels | 8.5.0 | 9.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23700 | 23900 |
| CC_NO_ARRAY_BOUNDS | all | (absent) | y |
| LD_VERSION | all | 23700 | 23900 |
| TOOLS_SUPPORT_RELR | all | (absent) | y |

### 9.5.0 to 10.5.0

Over 7.2.9.

| Symbol | Kernels | 9.5.0 | 10.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 24100 |
| LD_VERSION | all | 23900 | 24100 |

### 10.5.0 to 11.5.0

Over 7.2.9.

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
| ZERO_CALL_USED_REGS | all | (absent) | n |

### 11.5.0 to 12.2.0

Over 7.2.9.

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

Over 7.2.9.

| Symbol | Kernels | 12.2.0 | 12.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 24500 |
| CC_HAS_ASM_GOTO_OUTPUT | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | all | y | (absent) |
| LD_VERSION | all | 23900 | 24500 |

### 12.5.0 to 13.5.0

Over 7.2.9.

| Symbol | Kernels | 12.5.0 | 13.5.0 |
|---|---|---|---|
| AS_VERSION | all | 24500 | 20285426 |
| CC_HAS_ASSUME | all | (absent) | y |
| LD_VERSION | all | 24500 | 20285426 |

### 13.5.0 to 14.2.0

Over 7.2.9.

| Symbol | Kernels | 13.5.0 | 14.2.0 |
|---|---|---|---|
| AS_VERSION | all | 20285426 | 24301 |
| CC_HAS_KASAN_SW_TAGS | all | (absent) | y |
| CC_HAS_MIN_FUNCTION_ALIGNMENT | all | (absent) | y |
| CC_HAS_SANE_FUNCTION_ALIGNMENT | all | (absent) | y |
| LD_VERSION | all | 20285426 | 24301 |
