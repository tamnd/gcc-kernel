# Persona surface

What the kernel notices about each GCC release (spec 11.3), over the Current set: 5.10.271, 5.15.222, 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9. A persona that stands for a GCC has to give the same answers to all of it. Written by `gk report persona-surface`.

Three things are listed per GCC column. The version gates are the places the sources test the version outright, in `#if` lines, Makefiles and Kconfig, read from the trees and placed at the first pinned column that gets the other answer. The Kconfig symbols are the `.config` lines that differ from the column before that has a cell, from the newest cells, with `CC_VERSION_TEXT` and `GCC_VERSION` left out since they change at every step. The flags are the ones kbuild starts or stops passing to the units, from the cells that kept their `compile.jsonl`. A column with nothing in it is left out of the sections.

| Column | Gates | Kconfig symbols | Flags |
|---|--:|--:|--:|
| gcc-2.5.8 | 6 | 0 | 0 |
| gcc-3.0.4 | 6 | 0 | 0 |
| gcc-3.1.1 | 1 | 0 | 0 |
| gcc-3.4.6 | 4 | 0 | 0 |
| gcc-4.0.4 | 13 | 0 | 0 |
| gcc-4.3.5 | 1 | 0 | 0 |
| gcc-4.4.7 | 3 | 0 | 0 |
| gcc-4.5.4 | 1 | 0 | 0 |
| gcc-4.6.4 | 2 | 0 | 0 |
| gcc-4.8.5 | 3 | 0 | 0 |
| gcc-4.9.2 | 6 | 0 | 0 |
| gcc-4.9.4 | 1 | 0 | 0 |
| gcc-5.5.0 | 9 | 0 | 0 |
| gcc-6.3.0 | 3 | 0 | 0 |
| gcc-7.5.0 | 21 | 0 | 0 |
| gcc-8.3.0 | 9 | 0 | 0 |
| gcc-8.5.0 | 1 | 0 | 0 |
| gcc-9.5.0 | 13 | 12 | 2 |
| gcc-10.5.0 | 8 | 2 | 3 |
| gcc-11.5.0 | 13 | 14 | 2 |
| gcc-12.2.0 | 8 | 17 | 2 |
| gcc-12.5.0 | 1 | 9 | 0 |
| gcc-13.5.0 | 4 | 3 | 1 |
| gcc-14.2.0 | 5 | 7 | 2 |
| gcc-14.4.0 | 0 | 4 | 0 |
| gcc-15.3.0 | 2 | 2 | 2 |
| gcc-16.1.0 | 2 | 1 | 1 |
| gcc-16.2.0 | 0 | 0 | 0 |

## gcc-2.5.8

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 2.0.0 | lib/crypto/mpi/longlong.h | `#else /* __GNUC__ >= 2 */` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 2.0.0 | lib/crypto/mpi/longlong.h | `#endif /* __GNUC__ < 2 */` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 2.0.0 | lib/crypto/mpi/longlong.h | `#if __GNUC__ < 2` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 2.0.0 | lib/mpi/longlong.h | `#else /* __GNUC__ >= 2 */` | 5.10.271, 5.15.222 and 6.1.189 |
| 2.0.0 | lib/mpi/longlong.h | `#endif /* __GNUC__ < 2 */` | 5.10.271, 5.15.222 and 6.1.189 |
| 2.0.0 | lib/mpi/longlong.h | `#if __GNUC__ < 2` | 5.10.271, 5.15.222 and 6.1.189 |

## gcc-3.0.4

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 3.0.0 | include/linux/zstd_lib.h | `#  elif (__GNUC__ >= 3)` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 3.0.0 | lib/zstd/common/bitstream.h | `#   if (__GNUC__ >= 3)   /* Use GCC Intrinsic */` | 6.1.189, 6.6.158 and 6.12.112 |
| 3.0.0 | lib/zstd/common/entropy_common.c | `#   if (__GNUC__ >= 3)   /* GCC Intrinsic */` | 6.1.189, 6.6.158 and 6.12.112 |
| 3.0.0 | lib/zstd/common/zstd_internal.h | `#       if (__GNUC__ >= 3)` | 6.6.158 and 6.12.112 |
| 3.0.0 | lib/zstd/common/zstd_internal.h | `#   if (__GNUC__ >= 3)   /* GCC Intrinsic */` | 6.1.189, 6.6.158 and 6.12.112 |
| 3.0.0 | lib/zstd/compress/zstd_compress_internal.h | `#       if (__GNUC__ >= 3)` | 6.1.189, 6.6.158 and 6.12.112 |

## gcc-3.1.1

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 3.1.0 | lib/zstd/common/compiler.h | `#if ( (__GNUC__ >= 4) \|\| ( (__GNUC__ == 3) && (__GNUC_MINOR__ >= 1) ) )` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |

## gcc-3.4.6

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 3.4.0 | arch/alpha/include/uapi/asm/compiler.h | `# if __GNUC__ == 3 && __GNUC_MINOR__ >= 4 \|\| __GNUC__ > 3` | all |
| 3.4.0 | arch/alpha/include/uapi/asm/compiler.h | `#if __GNUC__ == 3 && __GNUC_MINOR__ >= 4 \|\| __GNUC__ > 3` | all |
| 3.4.0 | arch/ia64/include/uapi/asm/gcc_intrin.h | `#if __GNUC__ >= 4 \|\| (__GNUC__ == 3 && __GNUC_MINOR__ >= 4)` | 5.10.271, 5.15.222, 6.1.189 and 6.6.158 |
| 3.4.0 | lib/zstd/compress/zstd_lazy.c | `#   if (defined(__GNUC__) && ((__GNUC__ > 3) \|\| ((__GNUC__ == 3) && (__GNUC_MINOR__ >= 4))))` | 6.6.158 and 6.12.112 |

## gcc-4.0.4

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 4.0.0 | arch/arm/include/asm/div64.h | `#define __div64_const32_is_OK (__GNUC__ >= 4)` | 5.10.271 |
| 4.0.0 | arch/arm/include/asm/unified.h | `#if __GNUC__ < 4` | 5.10.271 |
| 4.0.0 | arch/microblaze/kernel/module.c | `#if __GNUC__ < 4` | 5.10.271 |
| 4.0.0 | include/asm-generic/div64.h | `#define __div64_const32_is_OK (__GNUC__ >= 4)` | 5.10.271 |
| 4.0.0 | include/linux/zstd_errors.h | `#  if (__GNUC__ >= 4) && !defined(__MINGW32__)` | 6.18.55 and 7.2.9 |
| 4.0.0 | include/linux/zstd_lib.h | `#  if (__GNUC__ >= 4) && !defined(__MINGW32__)` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 4.0.0 | lib/zstd/common/bits.h | `#if (__GNUC__ >= 4)` | 6.18.55 and 7.2.9 |
| 4.0.0 | lib/zstd/common/bits.h | `#if (__GNUC__ >= 4) && defined(__LP64__)` | 6.18.55 and 7.2.9 |
| 4.0.0 | lib/zstd/common/compiler.h | `#if !defined(__clang__) && defined(__GNUC__) && __GNUC__ >= 4 && __GNUC_MINOR__ >= 8 && __GNUC__ < 5` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 4.0.0 | lib/zstd/common/fse.h | `#if defined(FSE_DLL_EXPORT) && (FSE_DLL_EXPORT==1) && defined(__GNUC__) && (__GNUC__ >= 4)` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 4.0.0 | lib/zstd/common/huf.h | `#if defined(FSE_DLL_EXPORT) && (FSE_DLL_EXPORT==1) && defined(__GNUC__) && (__GNUC__ >= 4)` | 6.1.189, 6.6.158 and 6.12.112 |
| 4.0.0 | lib/zstd/common/zstd_internal.h | `#       if (__GNUC__ >= 4)` | 6.6.158 and 6.12.112 |
| 4.0.0 | lib/zstd/compress/zstd_compress_internal.h | `#       if (__GNUC__ >= 4)` | 6.1.189, 6.6.158 and 6.12.112 |

## gcc-4.3.5

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 4.3.0 | arch/arm/kernel/unwind.c | `#elif (__GNUC__ == 4 && __GNUC_MINOR__ <= 2) && !defined(__clang__)` | 5.10.271 |

## gcc-4.4.7

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 4.4.0 | lib/zstd/common/compiler.h | `#  if (__GNUC__ == 4 && __GNUC_MINOR__ > 3) \|\| (__GNUC__ >= 5)` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 4.4.0 | scripts/dtc/util.h | `#elif __GNUC__ >= 5 \|\| (__GNUC__ == 4 && __GNUC_MINOR__ >= 4)` | 6.12.112, 6.18.55 and 7.2.9 |
| 4.4.0 | scripts/dtc/util.h | `#if __GNUC__ >= 5 \|\| (__GNUC__ == 4 && __GNUC_MINOR__ >= 4)` | 6.1.189 and 6.6.158 |

## gcc-4.5.4

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 4.5.0 | lib/zstd/common/compiler.h | `#if __has_builtin(__builtin_unreachable) \|\| (defined(__GNUC__) && (__GNUC__ > 4 \|\| (__GNUC__ == 4 && __GNUC_MINOR__ >= 5)))` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |

## gcc-4.6.4

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 4.6.0 | arch/arm/lib/xor-neon.c | `#if __GNUC__ > 4 \|\| (__GNUC__ == 4 && __GNUC_MINOR__ >= 6)` | 5.10.271 and 5.15.222 |
| 4.6.0 | include/uapi/linux/v4l2-dv-timings.h | `#if __GNUC__ < 4 \|\| (__GNUC__ == 4 && (__GNUC_MINOR__ < 6))` | all |

## gcc-4.8.5

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 4.8.0 | drivers/gpu/drm/amd/display/dc/dml2/cmntypes.h | `#if __GNUC__ == 4 && __GNUC_MINOR__ > 7` | 6.12.112 and 6.18.55 |
| 4.8.0 | drivers/gpu/drm/amd/display/dc/dml2_0/cmntypes.h | `#if __GNUC__ == 4 && __GNUC_MINOR__ > 7` | 7.2.9 |
| 4.8.0 | lib/zstd/common/compiler.h | `&& (__GNUC__ >= 5 \|\| (__GNUC__ == 4 && __GNUC_MINOR__ >= 8)))) \` | 6.1.189 |

## gcc-4.9.2

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 4.9.0 | arch/mips/include/asm/compiler.h | `(__GNUC__ > 4) \|\| (__GNUC__ == 4 && __GNUC_MINOR__ >= 9)` | 5.10.271 |
| 4.9.0 | arch/powerpc/Makefile | `ifneq ($(call gcc-min-version, 40900),y)` | 5.10.271 |
| 4.9.0 | include/asm-generic/vmlinux.lds.h | `#if __GNUC__ == 4 && __GNUC_MINOR__ == 9` | 5.10.271 |
| 4.9.0 | include/linux/compiler-gcc.h | `#if GCC_VERSION < 40900` | 5.10.271 |
| 4.9.0 | kernel/gcov/gcc_4_7.c | `#elif __GNUC__ == 4 && __GNUC_MINOR__ >= 9` | 5.10.271 |
| 4.9.2 | include/linux/compiler-gcc.h | `#elif GCC_VERSION >= 40902` | 5.10.271 |

## gcc-4.9.4

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 4.9.3 | mm/ksm.c | `#if defined(GCC_VERSION) && GCC_VERSION >= 40903` | 5.10.271 |

## gcc-5.5.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 5.0.0 | Makefile | `ifneq ($(call gcc-min-version, 50000),y)` | 5.10.271 |
| 5.0.0 | arch/arm64/Kconfig | `select ARCH_SUPPORTS_INT128 if CC_HAS_INT128 && (GCC_VERSION >= 50000 \|\| CC_IS_CLANG)` | 5.10.271 |
| 5.0.0 | arch/powerpc/include/asm/asm-const.h | `#if defined(GCC_VERSION) && GCC_VERSION < 50000` | 5.10.271 |
| 5.0.0 | arch/riscv/Kconfig | `select ARCH_SUPPORTS_INT128 if CC_HAS_INT128 && GCC_VERSION >= 50000` | 5.10.271 |
| 5.0.0 | include/linux/compiler-gcc.h | `#elif GCC_VERSION >= 50000` | 5.10.271 |
| 5.0.0 | lib/zstd/common/compiler.h | `#if !defined(__clang__) && defined(__GNUC__) && __GNUC__ >= 4 && __GNUC_MINOR__ >= 8 && __GNUC__ < 5` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 5.1.0 | include/linux/compiler-gcc.h | `#elif defined(CONFIG_ARM64) && GCC_VERSION < 50100` | 5.10.271 |
| 5.1.0 | kernel/gcov/gcc_4_7.c | `#elif (__GNUC__ > 5) \|\| (__GNUC__ == 5 && __GNUC_MINOR__ >= 1)` | 5.10.271, 5.15.222, 6.1.189, 6.6.158 and 6.12.112 |
| 5.2.0 | arch/powerpc/Kconfig | `select HAVE_GCC_PLUGINS			if GCC_VERSION >= 50200   # plugin support on gcc <= 5.1 is buggy on PPC` | 5.10.271, 5.15.222, 6.1.189, 6.6.158 and 6.12.112 |

## gcc-6.3.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 6.0.0 | arch/arc/include/uapi/asm/elf.h | `#if __GNUC__ < 6` | all |
| 6.0.0 | arch/arm/mach-rpc/Kconfig | `depends on !CC_IS_CLANG && GCC_VERSION < 90100 && GCC_VERSION >= 60000` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 6.1.0 | arch/um/Makefile | `LINK-$(call gcc-min-version, 60100)$(CONFIG_CC_IS_CLANG) += -no-pie` | 6.1.189, 6.6.158 and 6.12.112 |

## gcc-7.5.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 7.0.0 | drivers/gpu/drm/i915/i915_utils.h | `#if defined(GCC_VERSION) && GCC_VERSION >= 70000` | 5.10.271, 5.15.222, 6.1.189 and 6.6.158 |
| 7.0.0 | include/linux/compiler-gcc.h | `#if GCC_VERSION >= 70000` | all |
| 7.0.0 | kernel/gcov/gcc_4_7.c | `#elif (__GNUC__ >= 7)` | 5.10.271, 5.15.222, 6.1.189, 6.6.158 and 6.12.112 |
| 7.0.0 | lib/zstd/decompress/zstd_decompress_block.c | `#  if __GNUC__ != 7` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 7.0.0 | lib/zstd/decompress/zstd_decompress_block.c | `#  if __GNUC__ >= 7` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 7.0.0 | lib/zstd/zstd_internal.h | `#if defined(GCC_VERSION) && GCC_VERSION >= 70000 && GCC_VERSION < 70200` | 5.10.271 and 5.15.222 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/calcs/Makefile | `ifneq ($(call gcc-min-version, 70100),y)` | 5.10.271 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn20/Makefile | `ifneq ($(call gcc-min-version, 70100), y)` | 5.15.222 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn20/Makefile | `ifneq ($(call gcc-min-version, 70100),y)` | 5.10.271 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn21/Makefile | `ifneq ($(call gcc-min-version, 70100), y)` | 5.15.222 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn21/Makefile | `ifneq ($(call gcc-min-version, 70100),y)` | 5.10.271 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn30/Makefile | `ifneq ($(call gcc-min-version, 70100), y)` | 5.15.222 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn30/Makefile | `ifneq ($(call gcc-min-version, 70100),y)` | 5.10.271 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn301/Makefile | `ifneq ($(call gcc-min-version, 70100), y)` | 5.15.222 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn302/Makefile | `ifneq ($(call gcc-min-version, 70100), y)` | 5.15.222 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn303/Makefile | `ifneq ($(call gcc-min-version, 70100), y)` | 5.15.222 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dcn31/Makefile | `ifneq ($(call gcc-min-version, 70100), y)` | 5.15.222 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dml/Makefile | `ifneq ($(call gcc-min-version, 70100),y)` | 5.10.271, 5.15.222, 6.1.189 and 6.6.158 |
| 7.1.0 | drivers/gpu/drm/amd/display/dc/dsc/Makefile | `ifneq ($(call gcc-min-version, 70100),y)` | 5.10.271 |
| 7.1.0 | scripts/Kbuild.include | `# Usage: cflags-$(call gcc-min-version, 70100) += -foo` | 5.10.271 |
| 7.1.0 | scripts/Makefile.compiler | `# Usage: cflags-$(call gcc-min-version, 70100) += -foo` | 5.15.222, 6.1.189, 6.6.158 and 6.12.112 |

## gcc-8.3.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 8.0.0 | drivers/net/ethernet/intel/i40e/i40e_xsk.h | `#elif __GNUC__ >= 8` | 5.15.222, 6.1.189, 6.6.158 and 6.12.112 |
| 8.0.0 | drivers/net/ethernet/intel/ice/ice_xsk.h | `#elif __GNUC__ >= 8` | 6.1.189, 6.6.158 and 6.12.112 |
| 8.0.0 | fs/btrfs/messages.h | `#if defined(CONFIG_CC_IS_CLANG) \|\| GCC_VERSION >= 80000` | 6.18.55 |
| 8.0.0 | include/linux/compiler-gcc.h | `#if GCC_VERSION >= 80000` | all |
| 8.0.0 | lib/Kconfig.ubsan | `depends on !CC_IS_GCC \|\| GCC_VERSION >= 80000` | 6.12.112 |
| 8.0.0 | lib/test_fortify/Makefile | `always-$(call gcc-min-version, 80000) += test_fortify.log` | 6.12.112 |
| 8.0.0 | lib/zstd/common/compiler.h | `#    if !defined(__clang__) && defined(__GNUC__) && __GNUC__ < 8` | 6.18.55 and 7.2.9 |
| 8.0.0 | lib/zstd/decompress/zstd_decompress_block.c | `#    if __GNUC__ == 8 \|\| __GNUC__ == 10` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 8.3.0 | lib/Kconfig.kasan | `def_bool !CC_IS_GCC \|\| GCC_VERSION >= 80300` | all |

## gcc-8.5.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 8.5.0 | arch/arm64/Kconfig | `default y if CC_IS_GCC && (GCC_VERSION >=  80500) && (GCC_VERSION <  90000)` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |

## gcc-9.5.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 9.0.0 | arch/arm64/Kconfig | `default y if CC_IS_GCC && (GCC_VERSION >=  80500) && (GCC_VERSION <  90000)` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 9.0.0 | arch/s390/Kconfig | `def_bool !(CC_IS_GCC && GCC_VERSION < 90000)` | 7.2.9 |
| 9.0.0 | arch/s390/include/asm/jump_label.h | `#elif __GNUC__ < 9` | all |
| 9.0.0 | init/Kconfig | `default y if CC_IS_GCC && GCC_VERSION >= 90000 && GCC10_NO_ARRAY_BOUNDS` | 6.12.112, 6.18.55 and 7.2.9 |
| 9.1.0 | Makefile | `KBUILD_CFLAGS-$(call gcc-min-version, 90100) += -Wno-alloc-size-larger-than` | 5.15.222 and 6.1.189 |
| 9.1.0 | arch/arm/mach-rpc/Kconfig | `depends on !CC_IS_CLANG && GCC_VERSION < 90100 && GCC_VERSION >= 60000` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 9.1.0 | arch/arm64/Kconfig | `depends on LD_IS_LLD \|\| LD_VERSION >= 23301 \|\| (CC_IS_GCC && GCC_VERSION < 90100)` | 5.15.222, 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 9.1.0 | arch/arm64/Kconfig | `depends on LD_IS_LLD \|\| LD_VERSION >= 233010000 \|\| (CC_IS_GCC && GCC_VERSION < 90100)` | 5.10.271 |
| 9.1.0 | include/linux/compiler-gcc.h | `#if GCC_VERSION < 90100` | 5.15.222, 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 9.1.0 | scripts/Makefile.extrawarn | `KBUILD_CFLAGS-$(call gcc-min-version, 90100) += -Wno-alloc-size-larger-than` | 6.6.158, 6.12.112 and 6.18.55 |
| 9.1.0 | scripts/Makefile.warn | `KBUILD_CFLAGS-$(call gcc-min-version, 90100) += -Wno-alloc-size-larger-than` | 7.2.9 |
| 9.3.0 | arch/mips/Kconfig | `depends on !(CPU_MICROMIPS && CC_IS_GCC && GCC_VERSION < 90300)` | 7.2.9 |
| 9.4.0 | arch/arm64/Kconfig | `default y if CC_IS_GCC && (GCC_VERSION >=  90400) && (GCC_VERSION < 100000)` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| ARM64_ERRATUM_4193714 | gcc-8.5.0 | arm64 | 7.2.8 | (absent) | y |
| ARM64_SME | gcc-8.5.0 | arm64 | 7.2.8 | (absent) | y |
| AS_HAS_MOPS | gcc-8.5.0 | arm64 | 7.2.8 | (absent) | y |
| AS_HAS_SME | gcc-8.5.0 | arm64 | 7.2.8 | (absent) | y |
| AS_VERSION | gcc-8.5.0 | arm64, i386, x86_64 | all | 23700 | 23900 |
| CC_HAS_BRANCH_PROT_PAC_RET | gcc-8.5.0 | arm64 | 7.2.8 | (absent) | y |
| CC_HAS_BRANCH_PROT_PAC_RET_BTI | gcc-8.5.0 | arm64 | 7.2.8 | (absent) | y |
| CC_HAVE_STACKPROTECTOR_SYSREG | gcc-8.5.0 | arm64 | 7.2.8 | (absent) | y |
| CC_NO_ARRAY_BOUNDS | gcc-8.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| LD_VERSION | gcc-8.5.0 | arm64, i386, x86_64 | all | 23700 | 23900 |
| STACKPROTECTOR_PER_TASK | gcc-8.5.0 | arm64 | 7.2.8 | (absent) | y |
| TOOLS_SUPPORT_RELR | gcc-8.5.0 | i386, x86_64 | all | (absent) | y |

### Flags

| Flag | Change | From | Platforms | Kernels |
|---|---|---|---|---|
| `-Wno-alloc-size-larger-than` | starts | gcc-8.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |
| `-Wno-array-bounds` | starts | gcc-8.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |

## gcc-10.5.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 10.0.0 | arch/arm64/Kconfig | `default y if CC_IS_GCC && (GCC_VERSION >=  90400) && (GCC_VERSION < 100000)` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 10.0.0 | arch/mips/lib/multi3.c | `#if defined(CONFIG_64BIT) && defined(CONFIG_CPU_MIPSR6) && (__GNUC__ < 10)` | all |
| 10.0.0 | init/Kconfig | `default y if CC_IS_GCC && GCC_VERSION >= 100000 && GCC10_NO_ARRAY_BOUNDS` | 6.1.189 and 6.6.158 |
| 10.0.0 | kernel/gcov/gcc_4_7.c | `#elif (__GNUC__ >= 10)` | all |
| 10.0.0 | lib/zstd/decompress/zstd_decompress_block.c | `#    if __GNUC__ == 8 \|\| __GNUC__ == 10` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 10.1.0 | arch/arm64/Kconfig | `depends on !CC_IS_GCC \|\| GCC_VERSION >= 100100` | all |
| 10.1.0 | drivers/gpu/drm/xe/xe_args.h | `#if defined(CONFIG_CC_IS_CLANG) \|\| GCC_VERSION >= 100100` | 7.2.9 |
| 10.2.0 | arch/arm64/Kconfig | `default y if CC_IS_GCC && (GCC_VERSION >= 100200) && (GCC_VERSION < 110000)` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| AS_VERSION | gcc-9.5.0 | arm64, i386, x86_64 | all | 23900 | 24100 |
| LD_VERSION | gcc-9.5.0 | arm64, i386, x86_64 | all | 23900 | 24100 |

### Flags

| Flag | Change | From | Platforms | Kernels |
|---|---|---|---|---|
| `--param=allow-store-data-races=0` | stops | gcc-9.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |
| `-Wenum-conversion` | starts | gcc-9.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |
| `-fno-allow-store-data-races` | starts | gcc-9.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |

## gcc-11.5.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 11.0.0 | arch/arm64/Kconfig | `default y if CC_IS_GCC && (GCC_VERSION >= 100200) && (GCC_VERSION < 110000)` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 11.0.0 | arch/x86/Makefile.um | `ifeq ($(call gcc-min-version, 110000)$(CONFIG_CC_IS_CLANG),y)` | 6.12.112, 6.18.55 and 7.2.9 |
| 11.0.0 | drivers/acpi/acpica/tbprint.c | `#if defined(__GNUC__) && __GNUC__ >= 11` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 11.0.0 | lib/zstd/common/portability_macros.h | `&& (__GNUC__ >= 11))) \` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 11.0.0 | lib/zstd/decompress/zstd_decompress_block.c | `#  elif __GNUC__ >= 11` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 11.1.0 | arch/arm64/Kconfig | `default y if CC_IS_GCC && (GCC_VERSION >= 110100)` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 11.1.0 | arch/powerpc/Kconfig | `select HAVE_LD_DEAD_CODE_DATA_ELIMINATION if HAVE_OBJTOOL_MCOUNT && (!ARCH_USING_PATCHABLE_FUNCTION_ENTRY \|\| (!CC_IS_GCC \|\| GCC_VERSION >= 110100))` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 11.1.0 | scripts/Makefile.compiler | `# Usage: cflags-$(call gcc-min-version, 110100) += -foo` | 6.18.55 and 7.2.9 |
| 11.2.0 | arch/s390/Kconfig | `def_bool EXPOLINE && CC_IS_GCC && GCC_VERSION >= 110200 && \` | 6.12.112, 6.18.55 and 7.2.9 |
| 11.2.0 | arch/s390/Kconfig | `depends on CC_IS_GCC && GCC_VERSION >= 110200` | 6.1.189 and 6.6.158 |
| 11.3.0 | arch/riscv/Kconfig | `depends on (CC_IS_CLANG && CLANG_VERSION < 170000) \|\| (CC_IS_GCC && GCC_VERSION < 110300)` | 6.1.189, 6.6.158, 6.12.112 and 6.18.55 |
| 11.3.0 | arch/riscv/Kconfig | `depends on CC_IS_GCC && GCC_VERSION < 110300` | 7.2.9 |
| 11.5.0 | init/Kconfig | `default y if GCC_VERSION < 110500` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| AS_VERSION | gcc-10.5.0 | arm64, i386, x86_64 | all | 24100 | 24301 |
| CC_HAS_ASM_GOTO_OUTPUT | gcc-10.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | gcc-10.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| CC_HAS_KASAN_SW_TAGS | gcc-10.5.0 | arm64 | 7.2.8 | (absent) | y |
| CC_HAS_SLS | gcc-10.5.0 | i386, x86_64 | all | (absent) | y |
| CC_HAS_ZERO_CALL_USED_REGS | gcc-10.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | gcc-10.5.0 | arm64, i386, x86_64 | all | y | (absent) |
| HAVE_KCSAN_COMPILER | gcc-10.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| KCSAN | gcc-10.5.0 | x86_64 | all | (absent) | n |
| LD_VERSION | gcc-10.5.0 | arm64, i386, x86_64 | all | 24100 | 24301 |
| MITIGATION_SLS | gcc-10.5.0 | x86_64 | all | (absent) | n |
| RELR | gcc-10.5.0 | arm64 | 7.2.8 | (absent) | y |
| TOOLS_SUPPORT_RELR | gcc-10.5.0 | arm64 | 7.2.8 | (absent) | y |
| ZERO_CALL_USED_REGS | gcc-10.5.0 | arm64, i386, x86_64 | all | (absent) | n |

### Flags

| Flag | Change | From | Platforms | Kernels |
|---|---|---|---|---|
| `-Wno-stringop-overread` | starts | gcc-10.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |
| `-mindirect-branch-cs-prefix` | starts | gcc-10.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |

## gcc-12.2.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 12.0.0 | arch/s390/Makefile | `ifeq ($(call gcc-min-version, 120000), y)` | 5.10.271 and 5.15.222 |
| 12.0.0 | arch/xtensa/Kconfig | `select HAVE_GCC_PLUGINS if GCC_VERSION >= 120000` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 12.0.0 | drivers/acpi/acpica/utdebug.c | `#if defined(__GNUC__) && __GNUC__ >= 12` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 12.0.0 | init/Kconfig | `default y if GCC_VERSION >= 120000 && GCC_VERSION < 120400` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 12.0.0 | kernel/gcov/gcc_4_7.c | `#if (__GNUC__ >= 12)` | all |
| 12.0.0 | lib/Kconfig.debug | `GCC_VERSION >= 120000 \|\| CC_IS_CLANG` | 6.12.112, 6.18.55 and 7.2.9 |
| 12.0.0 | lib/Kconfig.debug | `GCC_VERSION >= 120000 \|\| CLANG_VERSION >= 130000` | 6.1.189 and 6.6.158 |
| 12.0.0 | lib/Kconfig.debug | `depends on !RISCV \|\| GCC_VERSION >= 120000` | 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| ARCH_SUPPORTS_SHADOW_CALL_STACK | gcc-11.5.0 | arm64 | 7.2.8 | (absent) | y |
| AS_VERSION | gcc-11.5.0 | arm64, i386, x86_64 | all | 24301 | 23900 |
| CC_HAS_ASM_GOTO_OUTPUT | gcc-11.5.0 | arm64, i386, x86_64 | all | y | (absent) |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | gcc-11.5.0 | arm64, i386, x86_64 | all | y | (absent) |
| CC_HAS_AUTO_VAR_INIT_PATTERN | gcc-11.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO | gcc-11.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| CC_HAS_AUTO_VAR_INIT_ZERO_BARE | gcc-11.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| CC_HAVE_SHADOW_CALL_STACK | gcc-11.5.0 | arm64 | 7.2.8 | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | gcc-11.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| INIT_STACK_ALL_PATTERN | gcc-11.5.0 | arm64, i386, x86_64 | all | (absent) | n |
| INIT_STACK_ALL_ZERO | gcc-11.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| INIT_STACK_NONE | gcc-11.5.0 | arm64, i386, x86_64 | all | y | n |
| KCOV | gcc-11.5.0 | arm64 | 7.2.8 | (absent) | n |
| LD_VERSION | gcc-11.5.0 | arm64, i386, x86_64 | all | 24301 | 23900 |
| RELR | gcc-11.5.0 | arm64 | 7.2.8 | y | (absent) |
| SHADOW_CALL_STACK | gcc-11.5.0 | arm64 | 7.2.8 | (absent) | n |
| TOOLS_SUPPORT_RELR | gcc-11.5.0 | arm64 | 7.2.8 | y | (absent) |

### Flags

| Flag | Change | From | Platforms | Kernels |
|---|---|---|---|---|
| `-Wno-dangling-pointer` | starts | gcc-11.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |
| `-ftrivial-auto-var-init=zero` | starts | gcc-11.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |

## gcc-12.5.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 12.4.0 | init/Kconfig | `default y if GCC_VERSION >= 120000 && GCC_VERSION < 120400` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| ARM64_LSUI | gcc-12.2.0 | arm64 | 7.2.8 | (absent) | y |
| AS_HAS_LSUI | gcc-12.2.0 | arm64 | 7.2.8 | (absent) | y |
| AS_VERSION | gcc-12.2.0 | arm64, i386, x86_64 | all | 23900 | 24500 |
| CC_HAS_ASM_GOTO_OUTPUT | gcc-12.2.0 | arm64, i386, x86_64 | all | (absent) | y |
| CC_HAS_ASM_GOTO_TIED_OUTPUT | gcc-12.2.0 | arm64, i386, x86_64 | all | (absent) | y |
| GCC_ASM_GOTO_OUTPUT_BROKEN | gcc-12.2.0 | arm64, i386, x86_64 | all | y | (absent) |
| LD_VERSION | gcc-12.2.0 | arm64, i386, x86_64 | all | 23900 | 24500 |
| RELR | gcc-12.2.0 | arm64 | 7.2.8 | (absent) | y |
| TOOLS_SUPPORT_RELR | gcc-12.2.0 | arm64 | 7.2.8 | (absent) | y |

## gcc-13.5.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 13.0.0 | arch/s390/Makefile | `ifneq ($(call gcc-min-version, 130000), y)` | 5.10.271 and 5.15.222 |
| 13.0.0 | init/Kconfig | `default y if GCC_VERSION >= 130000 && GCC_VERSION < 130300` | 6.1.189, 6.6.158, 6.12.112, 6.18.55 and 7.2.9 |
| 13.1.0 | init/Kconfig | `default y if CC_IS_GCC && GCC_VERSION >= 130100` | 6.18.55 and 7.2.9 |
| 13.3.0 | arch/x86/Kconfig | `depends on !(KASAN \|\| KCSAN) \|\| GCC_VERSION >= 130300` | 6.12.112, 6.18.55 and 7.2.9 |

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| AS_VERSION | gcc-12.5.0 | arm64, i386, x86_64 | all | 24500 | 20285426 |
| CC_HAS_ASSUME | gcc-12.5.0 | arm64, i386, x86_64 | 6.18.54 and 7.2.8 | (absent) | y |
| LD_VERSION | gcc-12.5.0 | arm64, i386, x86_64 | all | 24500 | 20285426 |

### Flags

| Flag | Change | From | Platforms | Kernels |
|---|---|---|---|---|
| `-fstrict-flex-arrays=3` | starts | gcc-12.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |

## gcc-14.2.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 14.0.0 | include/linux/compiler-gcc.h | `#define CC_HAS_TYPEOF_UNQUAL (__GNUC__ >= 14)` | 6.18.55 and 7.2.9 |
| 14.0.0 | kernel/gcov/gcc_4_7.c | `#elif (__GNUC__ >= 14)` | all |
| 14.2.0 | arch/s390/Kconfig | `def_bool CC_IS_GCC && GCC_VERSION < 140200` | 6.18.55 and 7.2.9 |
| 14.2.0 | arch/s390/include/asm/atomic_ops.h | `#if defined(__GCC_ASM_FLAG_OUTPUTS__) && !(IS_ENABLED(CONFIG_CC_IS_GCC) && (GCC_VERSION < 140200))` | 6.12.112 |
| 14.2.0 | arch/x86/Kconfig | `depends on !(UBSAN_BOOL && KASAN) \|\| GCC_VERSION >= 140200` | 6.12.112, 6.18.55 and 7.2.9 |

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| ARM64_LSUI | gcc-13.5.0 | arm64 | 7.2.8 | y | (absent) |
| AS_HAS_LSUI | gcc-13.5.0 | arm64 | 7.2.8 | y | (absent) |
| AS_VERSION | gcc-13.5.0 | arm64, i386, x86_64 | all | 20285426 | 24301 |
| CC_HAS_KASAN_SW_TAGS | gcc-13.5.0 | x86_64 | all | (absent) | y |
| CC_HAS_MIN_FUNCTION_ALIGNMENT | gcc-13.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| CC_HAS_SANE_FUNCTION_ALIGNMENT | gcc-13.5.0 | arm64, i386, x86_64 | all | (absent) | y |
| LD_VERSION | gcc-13.5.0 | arm64, i386, x86_64 | all | 20285426 | 24301 |

### Flags

| Flag | Change | From | Platforms | Kernels |
|---|---|---|---|---|
| `-falign-functions=16` | stops | gcc-13.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |
| `-fmin-function-alignment=16` | starts | gcc-13.5.0 | x86_64 | 6.12.111, 6.18.54 and 7.2.8 |

## gcc-14.4.0

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| ARM64_LSUI | gcc-14.2.0 | arm64 | 7.2.8 | (absent) | y |
| AS_HAS_LSUI | gcc-14.2.0 | arm64 | 7.2.8 | (absent) | y |
| AS_VERSION | gcc-14.2.0 | arm64, i386, x86_64 | 6.18.54 and 7.2.8 | 24301 | 20285426 |
| LD_VERSION | gcc-14.2.0 | arm64, i386, x86_64 | 6.18.54 and 7.2.8 | 24301 | 20285426 |

## gcc-15.3.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 15.0.0 | kernel/gcov/gcc_4_7.c | `#if (__GNUC__ >= 15)` | all |
| 15.1.0 | init/Kconfig | `default y if CC_IS_GCC && GCC_VERSION >= 150100` | 6.18.55 and 7.2.9 |

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| CC_HAS_COUNTED_BY | gcc-14.4.0 | arm64, i386, x86_64 | 6.18.54 and 7.2.8 | (absent) | y |
| CC_HAS_MULTIDIMENSIONAL_NONSTRING | gcc-14.4.0 | arm64, i386, x86_64 | 6.18.54 and 7.2.8 | (absent) | y |

### Flags

| Flag | Change | From | Platforms | Kernels |
|---|---|---|---|---|
| `-Wno-unterminated-string-initialization` | starts | gcc-14.4.0 | x86_64 | 6.18.54 and 7.2.8 |
| `-fzero-init-padding-bits=all` | starts | gcc-14.4.0 | x86_64 | 6.18.54 and 7.2.8 |

## gcc-16.1.0

### Version gates

| Flips at | Where | Test | Kernels |
|---|---|---|---|
| 16.0.0 | arch/s390/Kconfig | `def_bool !(CC_IS_GCC && GCC_VERSION < 160000)` | 6.18.55 and 7.2.9 |
| 16.0.0 | init/Kconfig | `default y if CC_IS_GCC && GCC_VERSION >= 160000` | 7.2.9 |

### Kconfig symbols

| Symbol | From | Platforms | Kernels | Before | After |
|---|---|---|---|---|---|
| CC_HAS_COUNTED_BY_PTR | gcc-15.3.0 | arm64, i386, x86_64 | 7.2.8 | (absent) | y |

### Flags

| Flag | Change | From | Platforms | Kernels |
|---|---|---|---|---|
| `-fdiagnostics-show-context=2` | starts | gcc-15.3.0 | x86_64 | 7.2.8 |
