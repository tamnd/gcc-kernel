# Configuration differential

What the kernel's configuration notices about each GCC on the Current set (spec 11.3). For every kernel, the `.config` of each column is compared with the one of the next column that has a cell, and the symbols that change are gathered per step. `CC_VERSION_TEXT` and `GCC_VERSION` change at every step and are left out. A persona's `config-divergences.toml` in rucc-kernel must be a subset of these lines. Written by `gk publish`.

## arm64 defconfig+gk

No kernel has two columns to compare yet.

## arm64 tinyconfig+gk

No kernel has two columns to compare yet.

## i386 defconfig+gk

| Step | Kernels | Symbols that change |
|---|--:|--:|
| 12.2.0 to 16.2.0 | 1 | 3 |
| 14.2.0 to 16.2.0 | 1 | 4 |

### 12.2.0 to 16.2.0

Over 6.1.189.

| Symbol | Kernels | 12.2.0 | 16.2.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 20285426 |
| GCC_ASM_GOTO_OUTPUT_WORKAROUND | all | y | (absent) |
| LD_VERSION | all | 23900 | 20285426 |

### 14.2.0 to 16.2.0

Over 6.18.55.

| Symbol | Kernels | 14.2.0 | 16.2.0 |
|---|---|---|---|
| AS_VERSION | all | 24301 | 20285426 |
| CC_HAS_COUNTED_BY | all | (absent) | y |
| CC_HAS_MULTIDIMENSIONAL_NONSTRING | all | (absent) | y |
| LD_VERSION | all | 24301 | 20285426 |

## i386 tinyconfig+gk

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
| 14.2.0 to 16.2.0 | 1 | 4 |
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

### 14.2.0 to 16.2.0

Over 6.18.55.

| Symbol | Kernels | 14.2.0 | 16.2.0 |
|---|---|---|---|
| AS_VERSION | all | 24301 | 20285426 |
| CC_HAS_COUNTED_BY | all | (absent) | y |
| CC_HAS_MULTIDIMENSIONAL_NONSTRING | all | (absent) | y |
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
| 5.5.0 to 6.3.0 | 2 | 8 |
| 6.3.0 to 6.5.0 | 2 | 9 |
| 6.5.0 to 7.5.0 | 2 | 5 |
| 7.5.0 to 8.3.0 | 2 | 9 |
| 8.3.0 to 8.5.0 | 4 | 2 |
| 8.5.0 to 9.5.0 | 4 | 4 |
| 9.5.0 to 10.5.0 | 4 | 3 |
| 10.5.0 to 11.5.0 | 4 | 10 |
| 11.5.0 to 12.2.0 | 4 | 12 |
| 12.2.0 to 12.5.0 | 4 | 6 |
| 12.5.0 to 13.5.0 | 4 | 3 |
| 13.5.0 to 14.2.0 | 2 | 5 |
| 14.2.0 to 14.4.0 | 2 | 2 |
| 14.4.0 to 15.3.0 | 2 | 2 |
| 15.3.0 to 16.1.0 | 2 | 1 |
| 16.1.0 to 16.2.0 | 2 | 0 |

### 5.5.0 to 6.3.0

Over 6.6.158 and 6.12.112.

| Symbol | Kernels | 5.5.0 | 6.3.0 |
|---|---|---|---|
| AS_VERSION | all | 22901 | 22800 |
| AS_WRUSS | all | y | (absent) |
| CC_HAS_NAMED_AS | 6.12.112 | (absent) | y |
| CC_HAS_SANCOV_TRACE_PC | all | (absent) | y |
| KCOV | all | (absent) | n |
| LD_VERSION | all | 22901 | 22800 |
| USE_X86_SEG_SUPPORT | 6.12.112 | (absent) | y |
| X86_USER_SHADOW_STACK | all | n | (absent) |

### 6.3.0 to 6.5.0

Over 6.6.158 and 6.12.112.

| Symbol | Kernels | 6.3.0 | 6.5.0 |
|---|---|---|---|
| AS_GFNI | all | (absent) | y |
| AS_TPAUSE | all | (absent) | y |
| AS_VAES | 6.12.112 | (absent) | y |
| AS_VERSION | all | 22800 | 23101 |
| AS_VPCLMULQDQ | 6.12.112 | (absent) | y |
| AS_WRUSS | all | (absent) | y |
| CC_HAS_RETURN_THUNK | all | (absent) | y |
| LD_VERSION | all | 22800 | 23101 |
| X86_USER_SHADOW_STACK | all | (absent) | n |

### 6.5.0 to 7.5.0

Over 6.6.158 and 6.12.112.

| Symbol | Kernels | 6.5.0 | 7.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23101 | 23400 |
| CC_HAS_ASM_INLINE | all | (absent) | y |
| CC_HAS_NO_PROFILE_FN_ATTR | all | (absent) | y |
| CC_IMPLICIT_FALLTHROUGH | all | (absent) | "-Wimplicit-fallthrough=5" |
| LD_VERSION | all | 23101 | 23400 |

### 7.5.0 to 8.3.0

Over 6.6.158 and 6.12.112.

| Symbol | Kernels | 7.5.0 | 8.3.0 |
|---|---|---|---|
| AS_VERSION | all | 23400 | 23200 |
| CC_HAS_ENTRY_PADDING | all | (absent) | y |
| CC_HAS_IBT | all | (absent) | y |
| CC_HAS_SANE_STACKPROTECTOR | all | (absent) | y |
| CC_HAS_WORKING_NOSANITIZE_ADDRESS | all | (absent) | y |
| HAVE_STACKPROTECTOR | all | (absent) | y |
| LD_VERSION | all | 23400 | 23200 |
| STACKPROTECTOR | all | (absent) | n |
| X86_KERNEL_IBT | all | (absent) | n |

### 8.3.0 to 8.5.0

Over 6.6.158, 6.12.112, 6.18.55 and 7.2.9.

| Symbol | Kernels | 8.3.0 | 8.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23200 | 23700 |
| LD_VERSION | all | 23200 | 23700 |

### 8.5.0 to 9.5.0

Over 6.6.158, 6.12.112, 6.18.55 and 7.2.9.

| Symbol | Kernels | 8.5.0 | 9.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23700 | 23900 |
| CC_NO_ARRAY_BOUNDS | 6.12.112, 6.18.55 and 7.2.9 | (absent) | y |
| LD_VERSION | all | 23700 | 23900 |
| TOOLS_SUPPORT_RELR | all | (absent) | y |

### 9.5.0 to 10.5.0

Over 6.6.158, 6.12.112, 6.18.55 and 7.2.9.

| Symbol | Kernels | 9.5.0 | 10.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 24100 |
| CC_NO_ARRAY_BOUNDS | 6.6.158 | (absent) | y |
| LD_VERSION | all | 23900 | 24100 |

### 10.5.0 to 11.5.0

Over 6.6.158, 6.12.112, 6.18.55 and 7.2.9.

| Symbol | Kernels | 10.5.0 | 11.5.0 |
|---|---|---|---|
| AS_VERSION | all | 24100 | 24301 |
| CC_HAS_ASM_GOTO_OUTPUT | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | all | (absent) | y |
| CC_HAS_SLS | all | (absent) | y |
| CC_HAS_ZERO_CALL_USED_REGS | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | 6.12.112, 6.18.55 and 7.2.9 | y | (absent) |
| HAVE_KCSAN_COMPILER | all | (absent) | y |
| KCSAN | all | (absent) | n |
| LD_VERSION | all | 24100 | 24301 |
| ZERO_CALL_USED_REGS | all | (absent) | n |

### 11.5.0 to 12.2.0

Over 6.6.158, 6.12.112, 6.18.55 and 7.2.9.

| Symbol | Kernels | 11.5.0 | 12.2.0 |
|---|---|---|---|
| AS_VERSION | all | 24301 | 23900 |
| CC_HAS_ASM_GOTO_OUTPUT | 6.12.112, 6.18.55 and 7.2.9 | y | (absent) |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | 6.12.112, 6.18.55 and 7.2.9 | y | (absent) |
| CC_HAS_AUTO_VAR_INIT_PATTERN | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO_BARE | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | 6.12.112, 6.18.55 and 7.2.9 | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_WORKAROUND | 6.6.158 | (absent) | y |
| INIT_STACK_ALL_PATTERN | all | (absent) | n |
| INIT_STACK_ALL_ZERO | all | (absent) | y |
| INIT_STACK_NONE | all | y | n |
| LD_VERSION | all | 24301 | 23900 |

### 12.2.0 to 12.5.0

Over 6.6.158, 6.12.112, 6.18.55 and 7.2.9.

| Symbol | Kernels | 12.2.0 | 12.5.0 |
|---|---|---|---|
| AS_VERSION | all | 23900 | 24500 |
| CC_HAS_ASM_GOTO_OUTPUT | 6.12.112, 6.18.55 and 7.2.9 | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | 6.12.112, 6.18.55 and 7.2.9 | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | 6.12.112, 6.18.55 and 7.2.9 | y | (absent) |
| GCC_ASM_GOTO_OUTPUT_WORKAROUND | 6.6.158 | y | (absent) |
| LD_VERSION | all | 23900 | 24500 |

### 12.5.0 to 13.5.0

Over 6.6.158, 6.12.112, 6.18.55 and 7.2.9.

| Symbol | Kernels | 12.5.0 | 13.5.0 |
|---|---|---|---|
| AS_VERSION | all | 24500 | 20285426 |
| CC_HAS_ASSUME | 6.18.55 and 7.2.9 | (absent) | y |
| LD_VERSION | all | 24500 | 20285426 |

### 13.5.0 to 14.2.0

Over 6.18.55 and 7.2.9.

| Symbol | Kernels | 13.5.0 | 14.2.0 |
|---|---|---|---|
| AS_VERSION | all | 20285426 | 24301 |
| CC_HAS_KASAN_SW_TAGS | all | (absent) | y |
| CC_HAS_MIN_FUNCTION_ALIGNMENT | all | (absent) | y |
| CC_HAS_SANE_FUNCTION_ALIGNMENT | all | (absent) | y |
| LD_VERSION | all | 20285426 | 24301 |

### 14.2.0 to 14.4.0

Over 6.18.55 and 7.2.9.

| Symbol | Kernels | 14.2.0 | 14.4.0 |
|---|---|---|---|
| AS_VERSION | all | 24301 | 20285426 |
| LD_VERSION | all | 24301 | 20285426 |

### 14.4.0 to 15.3.0

Over 6.18.55 and 7.2.9.

| Symbol | Kernels | 14.4.0 | 15.3.0 |
|---|---|---|---|
| CC_HAS_COUNTED_BY | all | (absent) | y |
| CC_HAS_MULTIDIMENSIONAL_NONSTRING | all | (absent) | y |

### 15.3.0 to 16.1.0

Over 6.18.55 and 7.2.9.

| Symbol | Kernels | 15.3.0 | 16.1.0 |
|---|---|---|---|
| CC_HAS_COUNTED_BY_PTR | 7.2.9 | (absent) | y |
