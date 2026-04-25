# .NET 10 AArch64 (ARM64) CPU Feature Detection — Authoritative Reference

## Header

- **Tagged release:** `v10.0.0` (".NET 10.0.0" GA / RTM)
- **Commit SHA:** `60629d14374c56f1cb51819049ad1fa529307f8d`
- **Tag date:** 2025-11-11 (commit author date 2025-10-22)
- **Release URL:** <https://github.com/dotnet/runtime/releases/tag/v10.0.0>
- **Tree at SHA:** <https://github.com/dotnet/runtime/tree/60629d14374c56f1cb51819049ad1fa529307f8d>

> Verification: `gh api repos/dotnet/runtime/git/refs/tags/v10.0.0` returns object SHA `60629d14374c56f1cb51819049ad1fa529307f8d`. Every permalink in this report uses that SHA, not a branch name.

### Architecture summary

.NET 10's ARM64 feature detection is layered:

1. **Native probe (one function, all OSes):** `minipal_getcpufeatures()` in `src/native/minipal/cpufeatures.c` returns an `int` bitmask whose bits are defined by `ARM64IntrinsicConstants_*` macros in `src/native/minipal/cpufeatures.h`. The function picks one of three OS-specific paths via `#if`:
   - **Windows** — `IsProcessorFeaturePresent(PF_ARM_*)`.
   - **Linux/Android/FreeBSD with `<sys/auxv.h>` and `<asm/hwcap.h>`** — `getauxval(AT_HWCAP)` and `getauxval(AT_HWCAP2)` against `HWCAP_*`/`HWCAP2_*` bits.
   - **macOS / iOS / tvOS (and other Darwin) — and any Unix without HWCAP** — `sysctlbyname("hw.optional.arm.FEAT_*")` and `sysctlbyname("hw.optional.armv8_*")`.
2. **CoreCLR consumer:** `EEJitManager::SetCpuInfo()` in `src/coreclr/vm/codeman.cpp` calls `minipal_getcpufeatures()`, then translates each `ARM64IntrinsicConstants_*` bit (gated by an `EnableArm64*` config knob) to a JIT `InstructionSet_*` flag (`InstructionSet_Aes`, `InstructionSet_Rdm`, …).
3. **NativeAOT consumer:** `src/coreclr/nativeaot/Runtime/startup.cpp` calls `minipal_getcpufeatures()` once at startup to populate `g_cpuFeatures`.
4. **JIT enum:** `InstructionSet_ARM64_*` values are defined in `src/coreclr/inc/corinfoinstructionset.h` and the matching managed enum lives in `src/coreclr/tools/Common/JitInterface/CorInfoInstructionSet.cs`. AOT mapping back to stdarch-style ISA names (`"aes"`, `"crc"`, `"dotprod"`, `"rdma"`, `"sha1"`, `"sha2"`, `"lse"`, `"rcpc"`, `"rcpc2"`, `"sve"`, `"sve2"`) lives in `src/coreclr/tools/Common/Compiler/HardwareIntrinsicHelpers.cs`.
5. **Managed surface:** `System.Runtime.Intrinsics.Arm.{ArmBase, AdvSimd, Aes, Crc32, Dp, Rdm, Sha1, Sha256, Sve, Sve2}` and their nested `.Arm64` types each expose `IsSupported`. Each `IsSupported` is `[Intrinsic]` and is rewritten by the JIT to a constant true/false using the `InstructionSet_*` flag set in step 2. There is no managed `Atomics`/`Rcpc`/`Rcpc2` class — `Atomics` (LSE) is consumed only by codegen lowering of `System.Threading.Interlocked.*` (gated through `g_arm64_atomics_present` and `ARM64_ATOMICS_FEATURE_FLAG_BIT`). `Rcpc`/`Rcpc2` are JIT codegen flags only.
6. **Mono runtime** has its own, simpler probe in `src/mono/mono/utils/mono-hwcap-arm64.c` (Apple-only `sysctlbyname`; on Linux Mono uses generic `getauxval` plumbing in `mono-hwcap.c`).
7. **Special non-CPUID system register reads:** Two leaf functions in `src/coreclr/vm/arm64/asmhelpers.S` (and the `.asm` MASM equivalent for Windows): `GetDataCacheZeroIDReg` reads `DCZID_EL0` to gate `Dczva`, and `GetSveLengthFromOS` executes `RDVL` to discover the SVE vector length (only useful when `Sve` was already detected via the OS).

## File-by-file index

| File | Purpose | OS coverage | Permalink |
|---|---|---|---|
| `src/native/minipal/cpufeatures.h` | Bit-flag definitions for `ARM64IntrinsicConstants_*` shared between native code and AOT compilers | All | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.h> |
| `src/native/minipal/cpufeatures.c` | The single native ARM64 detection entry point (`minipal_getcpufeatures`) with three per-OS branches | Windows / Linux / Android / FreeBSD / macOS / iOS | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c> |
| `src/native/minipal/configure.cmake` | Sets `HAVE_AUXV_HWCAP_H` / `HAVE_SYSCTLBYNAME` / `HAVE_HWPROBE_H` build-time switches | All non-Windows | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/configure.cmake> |
| `src/coreclr/vm/codeman.cpp` (`EEJitManager::SetCpuInfo`, lines ~1171–1403) | CoreCLR translates the native bitmask to JIT flags + applies `EnableArm64*` env vars + Sve gating | All | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/vm/codeman.cpp#L1171-L1403> |
| `src/coreclr/nativeaot/Runtime/startup.cpp` (line 179) | NativeAOT populates `g_cpuFeatures` at runtime startup | All | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/nativeaot/Runtime/startup.cpp#L179> |
| `src/coreclr/nativeaot/Runtime/AsmOffsets.h` | Exposes `ARM64_ATOMICS_FEATURE_FLAG_BIT` (= 6) for use by hand-written ASM helpers (e.g. interlocked codegen) | All | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/nativeaot/Runtime/AsmOffsets.h#L40-L44> |
| `src/coreclr/vm/arm64/asmhelpers.S` (lines 19–31) | `GetDataCacheZeroIDReg` (`mrs x0, dczid_el0`) and `GetSveLengthFromOS` (`rdvl x0, 1`) — sole architectural-register reads in detection | Linux/macOS arm64 | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/vm/arm64/asmhelpers.S#L19-L31> |
| `src/coreclr/vm/arm64/asmhelpers.asm` | MASM equivalent of the above for Windows arm64 | Windows | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/vm/arm64/asmhelpers.asm> |
| `src/coreclr/inc/corinfoinstructionset.h` | Auto-generated `InstructionSet_ARM64_*` enum values used by JIT | All | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/inc/corinfoinstructionset.h> |
| `src/coreclr/tools/Common/JitInterface/CorInfoInstructionSet.cs` | Managed mirror of the JIT enum + propagation rules between the regular and `_Arm64` variants | All (used by ILC/crossgen2) | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/tools/Common/JitInterface/CorInfoInstructionSet.cs#L144-L175> |
| `src/coreclr/tools/Common/Compiler/HardwareIntrinsicHelpers.cs` | Maps the native `ARM64IntrinsicConstants_*` bits to stdarch-style ISA names (`"aes"`, `"crc"`, `"dotprod"`, `"rdma"`, …) for AOT | All AOT | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/tools/Common/Compiler/HardwareIntrinsicHelpers.cs#L223-L302> |
| `src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/*.cs` | Managed `Arm*.IsSupported` properties (intrinsified by JIT) | All | <https://github.com/dotnet/runtime/tree/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm> |
| `src/mono/mono/utils/mono-hwcap-arm64.c` | Mono runtime ARM64 detection (Apple `sysctlbyname` only; Linux uses generic Mono HWCAP plumbing) | macOS / iOS (Mono) | <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/mono/mono/utils/mono-hwcap-arm64.c> |

## Detection method per OS

The single entry point is `int minipal_getcpufeatures(void)`. Below is each OS's verbatim block.

### Windows (HOST_WINDOWS)

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L531-L582>

```c
#if defined(HOST_WINDOWS)
    if (IsProcessorFeaturePresent(PF_ARM_V8_CRYPTO_INSTRUCTIONS_AVAILABLE))
    {
        result |= ARM64IntrinsicConstants_Aes;
        result |= ARM64IntrinsicConstants_Sha1;
        result |= ARM64IntrinsicConstants_Sha256;
    }

    if (IsProcessorFeaturePresent(PF_ARM_V8_CRC32_INSTRUCTIONS_AVAILABLE))
    {
        result |= ARM64IntrinsicConstants_Crc32;
    }

    if (IsProcessorFeaturePresent(PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE))
    {
        result |= ARM64IntrinsicConstants_Atomics;
    }

    if (IsProcessorFeaturePresent(PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE))
    {
        result |= ARM64IntrinsicConstants_Dp;

        // IsProcessorFeaturePresent does not have a dedicated flag for RDM, so we enable it by implication.
        // 1) DP is an optional instruction set for Armv8.2, which may be included only in processors implementing at least Armv8.1.
        // 2) Armv8.1 requires RDM when AdvSIMD is implemented, and AdvSIMD is a baseline requirement of .NET.
        //
        // Therefore, by documented standard, DP cannot exist here without RDM. In practice, there is only one CPU supported
        // by Windows that includes RDM without DP, so this implication also has little practical chance of a false negative.
        //
        // See: https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/Understanding%20the%20Armv8.x%20extensions.pdf
        //      https://developer.arm.com/documentation/109697/2024_09/Feature-descriptions/The-Armv8-1-architecture-extension
        result |= ARM64IntrinsicConstants_Rdm;
    }

    if (IsProcessorFeaturePresent(PF_ARM_V83_LRCPC_INSTRUCTIONS_AVAILABLE))
    {
        result |= ARM64IntrinsicConstants_Rcpc;
    }

    // TODO: IsProcessorFeaturePresent doesn't support LRCPC2 yet.

    if (IsProcessorFeaturePresent(PF_ARM_SVE_INSTRUCTIONS_AVAILABLE))
    {
        result |= ARM64IntrinsicConstants_Sve;
    }

    if (IsProcessorFeaturePresent(PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE))
    {
        result |= ARM64IntrinsicConstants_Sve2;
    }

#endif // HOST_WINDOWS
```

`PF_ARM_SVE_INSTRUCTIONS_AVAILABLE = 46` and `PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE = 47` are forward-declared at lines 18–24 in case the SDK headers are too old:

```c
#ifndef PF_ARM_SVE_INSTRUCTIONS_AVAILABLE
#define PF_ARM_SVE_INSTRUCTIONS_AVAILABLE (46)
#endif

#ifndef PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE
#define PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE (47)
#endif
```

(<https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L18-L24>)

Per-detection annotations:

| `IsProcessorFeaturePresent` flag | `ARM64IntrinsicConstants_*` set | stdarch name(s) | Notes |
|---|---|---|---|
| `PF_ARM_V8_CRYPTO_INSTRUCTIONS_AVAILABLE` | `Aes`, `Sha1`, `Sha256` | `aes` (+ `pmull`), `sha1`, `sha2` | Windows reports the AArch64 v8 crypto extension as one bit; .NET decomposes it to all three managed surfaces. |
| `PF_ARM_V8_CRC32_INSTRUCTIONS_AVAILABLE` | `Crc32` | `crc` | |
| `PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE` | `Atomics` | `lse` | |
| `PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE` | `Dp` **and** `Rdm` (by implication) | `dotprod`, `rdma` | RDM is **inferred** because Windows has no PF flag for it. See the RDM deep-dive below. |
| `PF_ARM_V83_LRCPC_INSTRUCTIONS_AVAILABLE` | `Rcpc` | `rcpc` | |
| (none) | `Rcpc2` | `rcpc2` | Comment: `IsProcessorFeaturePresent doesn't support LRCPC2 yet.` |
| `PF_ARM_SVE_INSTRUCTIONS_AVAILABLE` (46) | `Sve` | `sve` | |
| `PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE` (47) | `Sve2` | `sve2` | |

### Linux / Android / FreeBSD (HAVE_AUXV_HWCAP_H)

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L455-L494>

```c
#if HAVE_AUXV_HWCAP_H
    unsigned long hwCap = getauxval(AT_HWCAP);

    assert(hwCap & HWCAP_ASIMD);

    if (hwCap & HWCAP_AES)
        result |= ARM64IntrinsicConstants_Aes;

    if (hwCap & HWCAP_ATOMICS)
        result |= ARM64IntrinsicConstants_Atomics;

    if (hwCap & HWCAP_CRC32)
        result |= ARM64IntrinsicConstants_Crc32;

    if (hwCap & HWCAP_ASIMDDP)
        result |= ARM64IntrinsicConstants_Dp;

    if (hwCap & HWCAP_LRCPC)
        result |= ARM64IntrinsicConstants_Rcpc;

    if (hwCap & HWCAP_ILRCPC)
        result |= ARM64IntrinsicConstants_Rcpc2;

    if (hwCap & HWCAP_SHA1)
        result |= ARM64IntrinsicConstants_Sha1;

    if (hwCap & HWCAP_SHA2)
        result |= ARM64IntrinsicConstants_Sha256;

    if (hwCap & HWCAP_ASIMDRDM)
        result |= ARM64IntrinsicConstants_Rdm;

    if (hwCap & HWCAP_SVE)
        result |= ARM64IntrinsicConstants_Sve;

    unsigned long hwCap2 = getauxval(AT_HWCAP2);

    if (hwCap2 & HWCAP2_SVE2)
        result |= ARM64IntrinsicConstants_Sve2;
```

The HWCAP bits that .NET back-fills for old kernel headers (the portable build still has to compile against Ubuntu 18.04-era headers):

```c
#ifndef HWCAP_ASIMDRDM
#define HWCAP_ASIMDRDM  (1 << 12)
#endif
#ifndef HWCAP_LRCPC
#define HWCAP_LRCPC     (1 << 15)
#endif
#ifndef HWCAP_ILRCPC
#define HWCAP_ILRCPC    (1 << 26)
#endif
#ifndef HWCAP_ASIMDDP
#define HWCAP_ASIMDDP   (1 << 20)
#endif
#ifndef HWCAP_SVE
#define HWCAP_SVE   (1 << 22)
#endif
#ifndef HWCAP2_SVE2
#define HWCAP2_SVE2   (1 << 1)
#endif
```

(<https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L36-L53>)

Annotations: each `HWCAP_*` bit is the standard Linux `<asm/hwcap.h>` mapping. AArch64 `HWCAP_ASIMDRDM` → `rdm`, `HWCAP_ASIMDDP` → `dotprod`, `HWCAP_LRCPC` → `rcpc`, `HWCAP_ILRCPC` → `rcpc2`, `HWCAP_SHA1`/`HWCAP_SHA2` → `sha1`/`sha2`, `HWCAP_ATOMICS` → `lse`. `assert(hwCap & HWCAP_ASIMD)` is the architectural baseline (`AdvSimd` is required by .NET).

The `HAVE_AUXV_HWCAP_H` macro is set by `src/native/minipal/configure.cmake` line 6: `check_include_files("sys/auxv.h;asm/hwcap.h" HAVE_AUXV_HWCAP_H)`. This is true for Linux (glibc/musl) and Android NDK r21+; it is false on FreeBSD (which falls into the `sysctlbyname` branch below, and on FreeBSD `sysctlbyname` returns nothing useful for ARM64, so detection effectively returns the empty set there) and on Apple platforms.

### macOS / iOS / tvOS (HAVE_SYSCTLBYNAME)

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L495-L527>

```c
#else // !HAVE_AUXV_HWCAP_H

#if HAVE_SYSCTLBYNAME
    int64_t valueFromSysctl = 0;
    size_t sz = sizeof(valueFromSysctl);

    if ((sysctlbyname("hw.optional.arm.FEAT_AES", &valueFromSysctl, &sz, NULL, 0) == 0) && (valueFromSysctl != 0))
        result |= ARM64IntrinsicConstants_Aes;

    if ((sysctlbyname("hw.optional.armv8_crc32", &valueFromSysctl, &sz, NULL, 0) == 0) && (valueFromSysctl != 0))
        result |= ARM64IntrinsicConstants_Crc32;

    if ((sysctlbyname("hw.optional.arm.FEAT_DotProd", &valueFromSysctl, &sz, NULL, 0) == 0) && (valueFromSysctl != 0))
        result |= ARM64IntrinsicConstants_Dp;

    if ((sysctlbyname("hw.optional.arm.FEAT_RDM", &valueFromSysctl, &sz, NULL, 0) == 0) && (valueFromSysctl != 0))
        result |= ARM64IntrinsicConstants_Rdm;

    if ((sysctlbyname("hw.optional.arm.FEAT_SHA1", &valueFromSysctl, &sz, NULL, 0) == 0) && (valueFromSysctl != 0))
        result |= ARM64IntrinsicConstants_Sha1;

    if ((sysctlbyname("hw.optional.arm.FEAT_SHA256", &valueFromSysctl, &sz, NULL, 0) == 0) && (valueFromSysctl != 0))
        result |= ARM64IntrinsicConstants_Sha256;

    if ((sysctlbyname("hw.optional.armv8_1_atomics", &valueFromSysctl, &sz, NULL, 0) == 0) && (valueFromSysctl != 0))
        result |= ARM64IntrinsicConstants_Atomics;

    if ((sysctlbyname("hw.optional.arm.FEAT_LRCPC", &valueFromSysctl, &sz, NULL, 0) == 0) && (valueFromSysctl != 0))
        result |= ARM64IntrinsicConstants_Rcpc;

    if ((sysctlbyname("hw.optional.arm.FEAT_LRCPC2", &valueFromSysctl, &sz, NULL, 0) == 0) && (valueFromSysctl != 0))
        result |= ARM64IntrinsicConstants_Rcpc2;
#endif // HAVE_SYSCTLBYNAME
#endif // HAVE_AUXV_HWCAP_H
```

Note: SVE/SVE2 are **not** detected on macOS/iOS — Apple silicon does not implement SVE, and there is no `hw.optional.*` key, so the bits are left clear.

### Android

Android uses the same Linux `getauxval(AT_HWCAP)` path. The only Android-specific code is a CMake glue line in `src/native/minipal/CMakeLists.txt` that links `liblog`; the detection logic is identical to the Linux block.

### FreeBSD

FreeBSD aarch64 has `<sys/auxv.h>` but no `<asm/hwcap.h>`, so `HAVE_AUXV_HWCAP_H` is false and `HAVE_SYSCTLBYNAME` is true. It thus enters the `sysctlbyname` branch, but the FreeBSD kernel does not export the Apple-specific keys (`hw.optional.arm.FEAT_*`), so all queries fail and the result is zero — i.e. the JIT sees only the architectural baseline (`ArmBase` + `AdvSimd`). This is a known gap; .NET does not officially support FreeBSD as a Tier 1 platform on ARM64.

## CoreCLR consumer — `EEJitManager::SetCpuInfo`

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/vm/codeman.cpp#L1171-L1403>

```cpp
void EEJitManager::SetCpuInfo()
{
    // ...
    int cpuFeatures = minipal_getcpufeatures();
    // ...
#elif defined(TARGET_ARM64)
    CPUCompileFlags.Set(InstructionSet_VectorT128);

    if (CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableHWIntrinsic))
    {
        CPUCompileFlags.Set(InstructionSet_ArmBase);
        CPUCompileFlags.Set(InstructionSet_AdvSimd);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Aes) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Aes))
    {
        CPUCompileFlags.Set(InstructionSet_Aes);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Atomics) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Atomics))
    {
        CPUCompileFlags.Set(InstructionSet_Atomics);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Rcpc) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Rcpc))
    {
        CPUCompileFlags.Set(InstructionSet_Rcpc);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Rcpc2) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Rcpc2))
    {
        CPUCompileFlags.Set(InstructionSet_Rcpc2);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Crc32) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Crc32))
    {
        CPUCompileFlags.Set(InstructionSet_Crc32);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Dp) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Dp))
    {
        CPUCompileFlags.Set(InstructionSet_Dp);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Rdm) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Rdm))
    {
        CPUCompileFlags.Set(InstructionSet_Rdm);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Sha1) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Sha1))
    {
        CPUCompileFlags.Set(InstructionSet_Sha1);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Sha256) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Sha256))
    {
        CPUCompileFlags.Set(InstructionSet_Sha256);
    }

    if (((cpuFeatures & ARM64IntrinsicConstants_Sve) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Sve))
    {
        uint32_t maxVectorTLength = (maxVectorTBitWidth / 8);
        uint64_t sveLengthFromOS = GetSveLengthFromOS();

        // For now, enable SVE only when the system vector length is 16 bytes (128-bits)
        // TODO: https://github.com/dotnet/runtime/issues/101477
        if (sveLengthFromOS == 16)
        // if ((maxVectorTLength >= sveLengthFromOS) || (maxVectorTBitWidth == 0))
        {
            CPUCompileFlags.Set(InstructionSet_Sve);

            if (((cpuFeatures & ARM64IntrinsicConstants_Sve2) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Sve2))
            {
                CPUCompileFlags.Set(InstructionSet_Sve2);
            }
        }
    }

    // DCZID_EL0<4> (DZP) indicates whether use of DC ZVA instructions is permitted (0) or prohibited (1).
    // DCZID_EL0<3:0> (BS) specifies Log2 of the block size in words.
    //
    // We set the flag when the instruction is permitted and the block size is 64 bytes.
    if ((GetDataCacheZeroIDReg() == 4) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Dczva))
    {
        CPUCompileFlags.Set(InstructionSet_Dczva);
    }

    if ((cpuFeatures & ARM64IntrinsicConstants_Atomics) != 0)
    {
        g_arm64_atomics_present = true;
    }
```

Two extras visible only here:

- **Dczva** (`DC ZVA` allowed, with 64-byte block size) is gated by reading `DCZID_EL0` directly via assembly leaf `GetDataCacheZeroIDReg`.
- **Sve / Sve2** are gated not just on the OS reporting the bit, but also on `RDVL #1` returning exactly 16 (i.e. 128-bit vector length), per [issue #101477](https://github.com/dotnet/runtime/issues/101477).

Permalinks for the assembly probes (Linux/macOS variant):

```asm
// DWORD64 __stdcall GetDataCacheZeroIDReg(void)
LEAF_ENTRY GetDataCacheZeroIDReg, _TEXT
    mrs     x0, dczid_el0
    and     x0, x0, 31
    ret     lr
LEAF_END GetDataCacheZeroIDReg, _TEXT

// uint64_t GetSveLengthFromOS(void);
.arch_extension sve
    LEAF_ENTRY GetSveLengthFromOS, _TEXT
        rdvl    x0, 1
        ret     lr
    LEAF_END GetSveLengthFromOS, _TEXT
```

(<https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/vm/arm64/asmhelpers.S#L19-L31>)

## NativeAOT consumer — `src/coreclr/nativeaot/Runtime/startup.cpp`

`g_cpuFeatures = minipal_getcpufeatures();` at line 179 — single statement, identical bitmask format.

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/nativeaot/Runtime/startup.cpp#L179>

The `ARM64_ATOMICS_FEATURE_FLAG_BIT` constant from `cpufeatures.h` is re-exposed to ASM helpers via `AsmOffsets.h`:

```c
#if defined(HOST_ARM64)
// Bit position for the ARM64IntrinsicConstants_Atomics flags, to be used with tbz / tbnz instructions
// ARM64IntrinsicConstants_Atomics = 0x0040
ASM_CONST(     6,     6, ARM64_ATOMICS_FEATURE_FLAG_BIT)
#endif
```

(<https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/nativeaot/Runtime/AsmOffsets.h#L40-L44>)

## AOT name mapping — `HardwareIntrinsicHelpers.cs`

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/tools/Common/Compiler/HardwareIntrinsicHelpers.cs#L223-L262>

```cs
// Keep these enumerations in sync with cpufeatures.h in the minipal.
private static class Arm64IntrinsicConstants
{
    public const int Aes = (1 << 0);
    public const int Crc32 = (1 << 1);
    public const int Dp = (1 << 2);
    public const int Rdm = (1 << 3);
    public const int Sha1 = (1 << 4);
    public const int Sha256 = (1 << 5);
    public const int Atomics = (1 << 6);
    public const int Rcpc = (1 << 7);
    public const int Rcpc2 = (1 << 8);
    public const int Sve = (1 << 9);
    public const int Sve2 = (1 << 10);

    public static void AddToBuilder(InstructionSetSupportBuilder builder, int flags)
    {
        if ((flags & Aes) != 0)
            builder.AddSupportedInstructionSet("aes");
        if ((flags & Crc32) != 0)
            builder.AddSupportedInstructionSet("crc");
        if ((flags & Dp) != 0)
            builder.AddSupportedInstructionSet("dotprod");
        if ((flags & Rdm) != 0)
            builder.AddSupportedInstructionSet("rdma");
        if ((flags & Sha1) != 0)
            builder.AddSupportedInstructionSet("sha1");
        if ((flags & Sha256) != 0)
            builder.AddSupportedInstructionSet("sha2");
        if ((flags & Atomics) != 0)
            builder.AddSupportedInstructionSet("lse");
        if ((flags & Rcpc) != 0)
            builder.AddSupportedInstructionSet("rcpc");
        if ((flags & Rcpc2) != 0)
            builder.AddSupportedInstructionSet("rcpc2");
        if ((flags & Sve) != 0)
            builder.AddSupportedInstructionSet("sve");
        if ((flags & Sve2) != 0)
            builder.AddSupportedInstructionSet("sve2");
    }
```

This is the canonical table .NET 10 uses to translate its own bitmask into the stdarch-style names recognised by `cc -march=armv8-a+<feature>`.

## Per-feature trace table (Windows path)

Each row shows the chain `Managed IsSupported → JIT enum → ARM64IntrinsicConstants_ → cpufeatures.c line(s)`. All managed `IsSupported` accessors share the JIT-rewriting idiom `public static bool IsSupported { get => IsSupported; }` — the recursive call is replaced at compile time by a constant true/false. References below point to the property declaration.

| Feature (stdarch) | Managed property | JIT ISA enum value | Native bit | Native detection (Windows) | Justification (verbatim if present) |
|---|---|---|---|---|---|
| `AdvSimd` / `asimd` | [`AdvSimd.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/AdvSimd.cs#L19) | `InstructionSet_ARM64.AdvSimd = 2` ([CorInfoInstructionSet.cs L149](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/tools/Common/JitInterface/CorInfoInstructionSet.cs#L149)) | (none — baseline) | Architectural baseline, not detected. Set unconditionally by [`SetCpuInfo` L1323-L1324](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/vm/codeman.cpp#L1323-L1324) when `EnableHWIntrinsic` is on. The Linux path additionally `assert(hwCap & HWCAP_ASIMD)`. | "AdvSIMD is a baseline requirement of .NET" |
| `ArmBase` / (root) | [`ArmBase.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/ArmBase.cs#L18) | `InstructionSet_ARM64.ArmBase = 1` | (none — baseline) | Same as AdvSimd. | — |
| `Aes` / `aes`, `pmull` | [`Aes.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/Aes.cs#L18) | `InstructionSet_ARM64.Aes = 3` | `ARM64IntrinsicConstants_Aes (1<<0)` | [cpufeatures.c L532-L537](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L532-L537) → `IsProcessorFeaturePresent(PF_ARM_V8_CRYPTO_INSTRUCTIONS_AVAILABLE)` | — |
| `Crc32` / `crc` | [`Crc32.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/Crc32.cs#L18) | `InstructionSet_ARM64.Crc32 = 4` | `ARM64IntrinsicConstants_Crc32 (1<<1)` | [cpufeatures.c L539-L542](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L539-L542) → `IsProcessorFeaturePresent(PF_ARM_V8_CRC32_INSTRUCTIONS_AVAILABLE)` | — |
| `Dp` / `dotprod` | [`Dp.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/Dp.cs#L19) | `InstructionSet_ARM64.Dp = 5` | `ARM64IntrinsicConstants_Dp (1<<2)` | [cpufeatures.c L549-L551](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L549-L551) → `IsProcessorFeaturePresent(PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE)` | — |
| `Rdm` / `rdma` | [`Rdm.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/Rdm.cs#L19) | `InstructionSet_ARM64.Rdm = 6` | `ARM64IntrinsicConstants_Rdm (1<<3)` | [cpufeatures.c L549-L563](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L549-L563) — **inferred** from DP (PR #109493) | "DP cannot exist here without RDM" — see RDM deep-dive |
| `Sha1` / `sha1` | [`Sha1.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/Sha1.cs#L18) | `InstructionSet_ARM64.Sha1 = 7` | `ARM64IntrinsicConstants_Sha1 (1<<4)` | [cpufeatures.c L532-L537](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L532-L537) (bundled with Aes under the V8 crypto bit) | — |
| `Sha256` / `sha2` | [`Sha256.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/Sha256.cs#L18) | `InstructionSet_ARM64.Sha256 = 8` | `ARM64IntrinsicConstants_Sha256 (1<<5)` | [cpufeatures.c L532-L537](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L532-L537) | — |
| Atomics / `lse` | (no managed type — used by JIT lowering of `Interlocked.*`) | `InstructionSet_ARM64.Atomics = 9` | `ARM64IntrinsicConstants_Atomics (1<<6)` | [cpufeatures.c L544-L547](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L544-L547) → `IsProcessorFeaturePresent(PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE)` | The bit-position 6 is also exposed to ASM via `ARM64_ATOMICS_FEATURE_FLAG_BIT` for `tbz/tbnz` |
| Rcpc / `rcpc` | (no managed type) | `InstructionSet_ARM64.Rcpc = 13` | `ARM64IntrinsicConstants_Rcpc (1<<7)` | [cpufeatures.c L565-L568](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L565-L568) → `IsProcessorFeaturePresent(PF_ARM_V83_LRCPC_INSTRUCTIONS_AVAILABLE)` | — |
| Rcpc2 / `rcpc2` | (no managed type) | `InstructionSet_ARM64.Rcpc2 = 15` | `ARM64IntrinsicConstants_Rcpc2 (1<<8)` | **Never set on Windows.** [cpufeatures.c L570](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L570) | "TODO: IsProcessorFeaturePresent doesn't support LRCPC2 yet." |
| `Sve` / `sve` | [`Sve.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/Sve.cs#L22) | `InstructionSet_ARM64.Sve = 16` | `ARM64IntrinsicConstants_Sve (1<<9)` | [cpufeatures.c L572-L575](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L572-L575) → `IsProcessorFeaturePresent(PF_ARM_SVE_INSTRUCTIONS_AVAILABLE)` (PF #46). Then re-gated on `GetSveLengthFromOS() == 16` in `SetCpuInfo`. | "For now, enable SVE only when the system vector length is 16 bytes (128-bits)" |
| `Sve2` / `sve2` | [`Sve2.IsSupported`](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/libraries/System.Private.CoreLib/src/System/Runtime/Intrinsics/Arm/Sve2.cs#L21) | `InstructionSet_ARM64.Sve2 = 17` | `ARM64IntrinsicConstants_Sve2 (1<<10)` | [cpufeatures.c L577-L580](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L577-L580) → `IsProcessorFeaturePresent(PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE)` (PF #47) | — |
| `Dczva` (DC ZVA, 64-byte block) | (no managed type) | `InstructionSet_ARM64.Dczva = 12` | (none — read directly) | `GetDataCacheZeroIDReg()` (`mrs x0, dczid_el0`) compared to `4` in [codeman.cpp L1395-L1398](https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/vm/codeman.cpp#L1395-L1398) | "DCZID_EL0<4> (DZP) indicates whether use of DC ZVA instructions is permitted (0) or prohibited (1). DCZID_EL0<3:0> (BS) specifies Log2 of the block size in words. We set the flag when the instruction is permitted and the block size is 64 bytes." |

Notes:

- `Atomics`, `Rcpc`, `Rcpc2`, `Dczva` deliberately have no managed type. `Atomics` is consumed only by the JIT's `Interlocked.*` lowering (and `g_arm64_atomics_present`). `Rcpc`/`Rcpc2` enable the JIT to emit `ldapr*` opcodes for volatile reads.
- The `_Arm64` enum twins (`Aes_Arm64`, `Rdm_Arm64`, …) are an internal convention used by the JIT to mark sub-APIs that are exposed only on 64-bit (i.e. all ARM64 in .NET 10). The propagation rule in `CorInfoInstructionSet.cs` lines 449–504 mirrors them automatically, so the detection table above does not need to track them separately.

## RDM deep-dive

### The PR

**PR:** [dotnet/runtime#109493 — "Enable Arm64 RDM on Windows"](https://github.com/dotnet/runtime/pull/109493)
**State:** Merged 2025-01-10 16:19 UTC, merge commit `96613ae650113f6d33e3b92433ddd43e3720b5d9`.

### Confirmation the PR is in `v10.0.0`

`gh api repos/dotnet/runtime/compare/96613ae650113f6d33e3b92433ddd43e3720b5d9...60629d14374c56f1cb51819049ad1fa529307f8d` returns `{ "status": "ahead", "ahead_by": 3218, "behind_by": 0 }` — i.e. `v10.0.0` is 3218 commits descended from `96613ae`, so the PR is contained in the release. The file's commit history at the tag also shows `96613ae` directly modifying `cpufeatures.c`:

```
2025-01-10  Enable Arm64 RDM on Windows (#109493)   96613ae650113f6d33e3b92433ddd43e3720b5d9
```

### The code in the release

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L549-L563>

```c
    if (IsProcessorFeaturePresent(PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE))
    {
        result |= ARM64IntrinsicConstants_Dp;

        // IsProcessorFeaturePresent does not have a dedicated flag for RDM, so we enable it by implication.
        // 1) DP is an optional instruction set for Armv8.2, which may be included only in processors implementing at least Armv8.1.
        // 2) Armv8.1 requires RDM when AdvSIMD is implemented, and AdvSIMD is a baseline requirement of .NET.
        //
        // Therefore, by documented standard, DP cannot exist here without RDM. In practice, there is only one CPU supported
        // by Windows that includes RDM without DP, so this implication also has little practical chance of a false negative.
        //
        // See: https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/Understanding%20the%20Armv8.x%20extensions.pdf
        //      https://developer.arm.com/documentation/109697/2024_09/Feature-descriptions/The-Armv8-1-architecture-extension
        result |= ARM64IntrinsicConstants_Rdm;
    }
```

### Pre-#109493 behaviour vs post

Before #109493 the Windows `if (IsProcessorFeaturePresent(PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE))` block set only `ARM64IntrinsicConstants_Dp`. Because Windows exposes no `PF_*` flag corresponding to RDM (it has `PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE` but no `PF_ARM_V81_RDM_INSTRUCTIONS_AVAILABLE`), `Rdm.IsSupported` always returned `false` on Windows ARM64 — even on hardware (e.g. Snapdragon X / Cobalt 100 / Ampere Altra running Win11 ARM) that fully implements RDM. This blocked any `System.Runtime.Intrinsics.Arm.Rdm.*` intrinsic and forced fallbacks to scalar / non-RDM AdvSIMD code paths.

Post-#109493 the RDM bit is **inferred from the DP bit**, on the basis that:

- DP (FEAT_DotProd) is part of the optional Armv8.2 extensions; any chip implementing DP also implements at least the Armv8.1 baseline.
- RDM (FEAT_RDM) is mandatory in Armv8.1 whenever AdvSIMD is implemented.
- AdvSIMD is a baseline requirement of .NET (the Linux path even `assert`s on it: line 458).

Therefore a chip that reports DP via `IsProcessorFeaturePresent` must have RDM. The comment also acknowledges the only theoretical false-negative case (an Armv8.1 chip that has RDM but no DP) is "extremely rare" among Windows-supported parts.

This is functionally identical to the inverse of the rule .NET would use on Linux: on Linux `HWCAP_ASIMDDP` and `HWCAP_ASIMDRDM` are exposed independently, so .NET reads them independently; on Apple, separate `hw.optional.arm.FEAT_DotProd` and `hw.optional.arm.FEAT_RDM` are queried; only Windows lacks the granularity, hence the implication.

### How RDM is consumed downstream

```cpp
if (((cpuFeatures & ARM64IntrinsicConstants_Rdm) != 0) && CLRConfig::GetConfigValue(CLRConfig::EXTERNAL_EnableArm64Rdm))
{
    CPUCompileFlags.Set(InstructionSet_Rdm);
}
```

(<https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/vm/codeman.cpp#L1357-L1360>)

`InstructionSet_Rdm` then propagates to `Rdm_Arm64` automatically (CorInfoInstructionSet.cs L470-L473) and the JIT rewrites `Rdm.IsSupported` and `Rdm.Arm64.IsSupported` to constant `true`.

The user-visible config knob to opt out is `DOTNET_EnableArm64Rdm=0`.

## Verbatim-quotation appendix

### A. `cpufeatures.h` — full ARM64 macro block

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.h#L35-L54>

```c
#if defined(HOST_ARM64)
#define ARM64IntrinsicConstants_Aes (1 << 0)
#define ARM64IntrinsicConstants_Crc32 (1 << 1)
#define ARM64IntrinsicConstants_Dp (1 << 2)
#define ARM64IntrinsicConstants_Rdm (1 << 3)
#define ARM64IntrinsicConstants_Sha1 (1 << 4)
#define ARM64IntrinsicConstants_Sha256 (1 << 5)
#define ARM64IntrinsicConstants_Atomics (1 << 6)
#define ARM64IntrinsicConstants_Rcpc (1 << 7)
#define ARM64IntrinsicConstants_Rcpc2 (1 << 8)
#define ARM64IntrinsicConstants_Sve (1 << 9)
#define ARM64IntrinsicConstants_Sve2 (1 << 10)

#include <assert.h>

// Bit position for the ARM64IntrinsicConstants_Atomics flags, to be used with tbz / tbnz instructions
#define ARM64_ATOMICS_FEATURE_FLAG_BIT 6
static_assert((1 << ARM64_ATOMICS_FEATURE_FLAG_BIT) == ARM64IntrinsicConstants_Atomics, "ARM64_ATOMICS_FEATURE_FLAG_BIT must match with ARM64IntrinsicConstants_Atomics");

#endif // HOST_ARM64
```

### B. `cpufeatures.c` — outer ARM64 dispatch (the `#if defined(HOST_ARM64)` block in full)

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/cpufeatures.c#L452-L584>

(Already fully quoted above in three OS-specific slices: lines 455–494 for Linux/HWCAP, 495–527 for sysctl, 531–582 for Windows. The `#if defined(HOST_ARM64)` opening at line 452 and the `#endif // HOST_ARM64` closing at line 584 wrap them together.)

### C. `mono-hwcap-arm64.c` — full file

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/mono/mono/utils/mono-hwcap-arm64.c>

```c
/**
 * \file
 * ARM64 hardware feature detection
 *
 * Copyright 2013 Xamarin Inc
 * Licensed under the MIT license. See LICENSE file in the project root for full license information.
 */

#ifdef __APPLE__
#include <sys/types.h>
#include <sys/sysctl.h>
#endif

#include "mono/utils/mono-hwcap.h"

void
mono_hwcap_arch_init (void)
{
#ifdef __APPLE__
    const char *prop;
    guint val [16];
    size_t val_len;
    int res;

    val_len = sizeof (val);
    prop = "hw.optional.armv8_crc32";
    res = sysctlbyname (prop, val, &val_len, NULL, 0);
    if (res == 0) {
        g_assert (val_len == 4);
        mono_hwcap_arm64_has_crc32 = *(int*)val;
    } else {
        mono_hwcap_arm64_has_crc32 = 0;
    }

    val_len = sizeof (val);
    prop = "hw.optional.arm.FEAT_RDM";
    res = sysctlbyname (prop, val, &val_len, NULL, 0);
    if (res == 0) {
        g_assert (val_len == 4);
        mono_hwcap_arm64_has_rdm = *(int*)val;
    } else {
        mono_hwcap_arm64_has_rdm = 0;
    }

    val_len = sizeof (val);
    prop = "hw.optional.arm.FEAT_DotProd";
    res = sysctlbyname (prop, val, &val_len, NULL, 0);
    if (res == 0) {
        g_assert (val_len == 4);
        mono_hwcap_arm64_has_dot = *(int*)val;
    } else {
        mono_hwcap_arm64_has_dot = 0;
    }

    val_len = sizeof (val);
    prop = "hw.optional.arm.FEAT_SHA1";
    res = sysctlbyname (prop, val, &val_len, NULL, 0);
    if (res == 0) {
        g_assert (val_len == 4);
        mono_hwcap_arm64_has_sha1 = *(int*)val;
    } else {
        mono_hwcap_arm64_has_sha1 = 0;
    }

    val_len = sizeof (val);
    prop = "hw.optional.arm.FEAT_SHA256";
    res = sysctlbyname (prop, val, &val_len, NULL, 0);
    if (res == 0) {
        g_assert (val_len == 4);
        mono_hwcap_arm64_has_sha256 = *(int*)val;
    } else {
        mono_hwcap_arm64_has_sha256 = 0;
    }

    val_len = sizeof (val);
    prop = "hw.optional.arm.FEAT_AES";
    res = sysctlbyname (prop, val, &val_len, NULL, 0);
    if (res == 0) {
        g_assert (val_len == 4);
        mono_hwcap_arm64_has_aes = *(int*)val;
    } else {
        mono_hwcap_arm64_has_aes = 0;
    }

#endif
}
```

Mono on Linux ARM64 uses the generic `mono-hwcap.c` infrastructure that reads `getauxval(AT_HWCAP)` once, identical in spirit to the minipal path but a separate code path; on macOS Mono's `mono_hwcap_arm64_has_*` globals are populated as above. There is no Mono Windows-ARM64 detection because Mono is not the Windows-ARM64 runtime — CoreCLR is.

### D. `CorInfoInstructionSet.cs` — ARM64 enum (managed mirror of the JIT enum)

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/coreclr/tools/Common/JitInterface/CorInfoInstructionSet.cs#L144-L175>

```cs
public enum InstructionSet_ARM64
{
    ILLEGAL = InstructionSet.ILLEGAL,
    NONE = InstructionSet.NONE,
    ArmBase = 1,
    AdvSimd = 2,
    Aes = 3,
    Crc32 = 4,
    Dp = 5,
    Rdm = 6,
    Sha1 = 7,
    Sha256 = 8,
    Atomics = 9,
    Vector64 = 10,
    Vector128 = 11,
    Dczva = 12,
    Rcpc = 13,
    VectorT128 = 14,
    Rcpc2 = 15,
    Sve = 16,
    Sve2 = 17,
    ArmBase_Arm64 = 18,
    AdvSimd_Arm64 = 19,
    Aes_Arm64 = 20,
    Crc32_Arm64 = 21,
    Dp_Arm64 = 22,
    Rdm_Arm64 = 23,
    Sha1_Arm64 = 24,
    Sha256_Arm64 = 25,
    Sve_Arm64 = 26,
    Sve2_Arm64 = 27,
}
```

### E. Build-time configure switches

Permalink: <https://github.com/dotnet/runtime/blob/60629d14374c56f1cb51819049ad1fa529307f8d/src/native/minipal/configure.cmake>

```cmake
check_include_files("sys/auxv.h;asm/hwcap.h" HAVE_AUXV_HWCAP_H)
check_include_files("asm/hwprobe.h" HAVE_HWPROBE_H)

check_function_exists(sysctlbyname HAVE_SYSCTLBYNAME)
```

The three macros (`HAVE_AUXV_HWCAP_H`, `HAVE_HWPROBE_H`, `HAVE_SYSCTLBYNAME`) are written into `minipalconfig.h` and consumed by the `#if` ladder at the top of `cpufeatures.c`. There is **no** `-march=` or `+feature` modifier set globally for ARM64 in `eng/native/configurecompiler.cmake` — .NET 10's portable build deliberately compiles its native components for the architectural baseline only, then dispatches at runtime.
