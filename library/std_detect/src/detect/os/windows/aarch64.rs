//! Run-time feature detection for Aarch64 on Windows.
//!
//! Pure `IsProcessorFeaturePresent` (IPFP) backend — no registry reads,
//! no ID-register decoding, no regex, no string parsing. Every probe is
//! a single Win32 syscall returning a `BOOL`.
//!
//! ## Coverage (SDK 26100 / Windows 11 24H2)
//!
//! All 56 `PF_ARM_*` constants Microsoft defines in `winnt.h` are wired
//! below, covering 36 stdarch feature names directly:
//!
//! ```text
//!   fp, asimd, crc, aes, pmull, sha2, sha3,
//!   lse, lse2, rcpc, dotprod, jsconv,
//!   fp16, bf16, i8mm, sve, sve2, sve2p1,
//!   sve2_aes, sve2_bitperm, sve2_sha3, sve2_sm4, sve_b16b16,
//!   f32mm, f64mm,
//!   sme, sme2, sme2p1,
//!   sme_b16b16, sme_f16f16, sme_f64f64, sme_f8f16, sme_f8f32,
//!   sme_fa64, sme_i16i64, sme_lutv2,
//!   ssve_fp8dot2, ssve_fp8dot4, ssve_fp8fma
//! ```
//!
//! Plus one architecturally-derived feature:
//!
//! ```text
//!   rdm
//! ```
//!
//! ## The `rdm` derivation
//!
//! Microsoft has never defined a `PF_ARM_RDM_*` constant — confirmed by
//! `dotnet/runtime#74778` ("RCPC, DC ZVA and probably RDM ISAs are never
//! detected on win-arm64", 2022) and the resolution in `dotnet/runtime#109493`
//! ("Enable Arm64 RDM on Windows", merged 2025-01).
//!
//! .NET 10 GA (`v10.0.0`, commit `60629d14`) ships the same architectural
//! inference used here at `src/native/minipal/cpufeatures.c` lines 549-563:
//! if `PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE` is set, FEAT_RDM is set.
//! Justification (ARM ARM K.a §D17.2.91): *"In an ARMv8.1 implementation,
//! if FEAT_AdvSIMD is implemented, FEAT_RDM is implemented."* FEAT_DotProd
//! is v8.2-A → implies v8.1-A baseline → implies FEAT_RDM (since AdvSIMD
//! is universally implemented on Windows-on-ARM hardware).
//!
//! Same reasoning lets `PF_ARM_V81_ATOMIC` (FEAT_LSE, also v8.1-A
//! mandatory) imply FEAT_RDM. We apply both as belt-and-suspenders;
//! either one catches the bit.
//!
//! ## Features stdarch knows but Microsoft does not surface
//!
//! ~30 stdarch feature names have no `PF_ARM_*` constant in any Windows
//! SDK. They report `false` here — same as upstream stdarch and same as
//! .NET 10. A separate crate (`winarm-cpufeatures` on crates.io) offers
//! optional registry-based ID-register decoding for callers that need
//! these specialized names (`fhm`, `fcma`, `frintts`, `paca`, `pacg`,
//! `bti`, `dpb`, `dpb2`, `mte`, `mops`, `dit`, `sb`, `ssbs`, `flagm`,
//! `flagm2`, `rand`, `cssc`, `wfxt`, `hbc`, `rcpc2`, `rcpc3`, `pauth_lr`,
//! `lse128`, `tme`, `ecv`, `lut`, `faminmax`, `fp8*`, `fpmr`, `sm4`).

use crate::detect::{Feature, cache};

pub(crate) fn detect_features() -> cache::Initializer {
    type DWORD = u32;
    type BOOL = i32;
    const FALSE: BOOL = 0;

    // Verbatim from `winnt.h`, Windows SDK 10.0.26100.0 (Win11 24H2).
    // Numbered comments mark the SDK series each batch first appeared in.

    // ── ARMv8.0 baseline (SDK 17134 / Win10 RS4) ────────────────────────
    const PF_ARM_VFP_32_REGISTERS_AVAILABLE: DWORD = 18;
    const PF_ARM_NEON_INSTRUCTIONS_AVAILABLE: DWORD = 19;
    const PF_ARM_V8_CRYPTO_INSTRUCTIONS_AVAILABLE: DWORD = 30;
    const PF_ARM_V8_CRC32_INSTRUCTIONS_AVAILABLE: DWORD = 31;
    // ── ARMv8.1 atomics (SDK 17763 / Win10 RS5) ─────────────────────────
    const PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE: DWORD = 34;
    // ── ARMv8.2/8.3 (SDK 19041 / Win10 20H1) ────────────────────────────
    const PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE: DWORD = 43;
    const PF_ARM_V83_JSCVT_INSTRUCTIONS_AVAILABLE: DWORD = 44;
    const PF_ARM_V83_LRCPC_INSTRUCTIONS_AVAILABLE: DWORD = 45;
    // ── SVE family (SDK 26100 / Win11 24H2) ─────────────────────────────
    const PF_ARM_SVE_INSTRUCTIONS_AVAILABLE: DWORD = 46;
    const PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE: DWORD = 47;
    const PF_ARM_SVE2_1_INSTRUCTIONS_AVAILABLE: DWORD = 48;
    const PF_ARM_SVE_AES_INSTRUCTIONS_AVAILABLE: DWORD = 49;
    const PF_ARM_SVE_PMULL128_INSTRUCTIONS_AVAILABLE: DWORD = 50;
    const PF_ARM_SVE_BITPERM_INSTRUCTIONS_AVAILABLE: DWORD = 51;
    // 52 PF_ARM_SVE_BF16 / 53 PF_ARM_SVE_EBF16 — SVE-form BF16; stdarch
    //    `bf16` semantics are the AdvSIMD form (PF_ARM_V86_BF16, #68).
    const PF_ARM_SVE_B16B16_INSTRUCTIONS_AVAILABLE: DWORD = 54;
    const PF_ARM_SVE_SHA3_INSTRUCTIONS_AVAILABLE: DWORD = 55;
    const PF_ARM_SVE_SM4_INSTRUCTIONS_AVAILABLE: DWORD = 56;
    // 57 PF_ARM_SVE_I8MM — SVE-form; AdvSIMD i8mm is V82_I8MM (#66).
    const PF_ARM_SVE_F32MM_INSTRUCTIONS_AVAILABLE: DWORD = 58;
    const PF_ARM_SVE_F64MM_INSTRUCTIONS_AVAILABLE: DWORD = 59;
    // ── AdvSIMD additions + SME (SDK 26100 / Win11 24H2) ────────────────
    const PF_ARM_LSE2_AVAILABLE: DWORD = 62;
    const PF_ARM_SHA3_INSTRUCTIONS_AVAILABLE: DWORD = 64;
    const PF_ARM_SHA512_INSTRUCTIONS_AVAILABLE: DWORD = 65;
    const PF_ARM_V82_I8MM_INSTRUCTIONS_AVAILABLE: DWORD = 66;
    const PF_ARM_V82_FP16_INSTRUCTIONS_AVAILABLE: DWORD = 67;
    const PF_ARM_V86_BF16_INSTRUCTIONS_AVAILABLE: DWORD = 68;
    // 69 PF_ARM_V86_EBF16 — no stdarch name today.
    const PF_ARM_SME_INSTRUCTIONS_AVAILABLE: DWORD = 70;
    const PF_ARM_SME2_INSTRUCTIONS_AVAILABLE: DWORD = 71;
    const PF_ARM_SME2_1_INSTRUCTIONS_AVAILABLE: DWORD = 72;
    // 73..=77 SME2_2 / SME_AES / SME_SBITPERM / SME_SF8MM4 / SME_SF8MM8 —
    //         no stdarch feature names yet.
    const PF_ARM_SME_SF8DP2_INSTRUCTIONS_AVAILABLE: DWORD = 78;
    const PF_ARM_SME_SF8DP4_INSTRUCTIONS_AVAILABLE: DWORD = 79;
    const PF_ARM_SME_SF8FMA_INSTRUCTIONS_AVAILABLE: DWORD = 80;
    const PF_ARM_SME_F8F32_INSTRUCTIONS_AVAILABLE: DWORD = 81;
    const PF_ARM_SME_F8F16_INSTRUCTIONS_AVAILABLE: DWORD = 82;
    const PF_ARM_SME_F16F16_INSTRUCTIONS_AVAILABLE: DWORD = 83;
    const PF_ARM_SME_B16B16_INSTRUCTIONS_AVAILABLE: DWORD = 84;
    const PF_ARM_SME_F64F64_INSTRUCTIONS_AVAILABLE: DWORD = 85;
    const PF_ARM_SME_I16I64_INSTRUCTIONS_AVAILABLE: DWORD = 86;
    #[allow(non_upper_case_globals)] // matches winnt.h spelling exactly
    const PF_ARM_SME_LUTv2_INSTRUCTIONS_AVAILABLE: DWORD = 87;
    const PF_ARM_SME_FA64_INSTRUCTIONS_AVAILABLE: DWORD = 88;

    unsafe extern "system" {
        fn IsProcessorFeaturePresent(ProcessorFeature: DWORD) -> BOOL;
    }

    let mut value = cache::Initializer::default();
    let mut enable = |f: Feature, on: bool| {
        if on {
            value.set(f as u32);
        }
    };

    // SAFETY: IsProcessorFeaturePresent takes a DWORD by value, returns
    // a BOOL. Pure, no pointers, no out-parameters, no reentrancy.
    unsafe {
        let p = |c: DWORD| IsProcessorFeaturePresent(c) != FALSE;

        // ── Baseline ────────────────────────────────────────────────────
        enable(Feature::fp, p(PF_ARM_VFP_32_REGISTERS_AVAILABLE));
        enable(Feature::asimd, p(PF_ARM_NEON_INSTRUCTIONS_AVAILABLE));
        enable(Feature::crc, p(PF_ARM_V8_CRC32_INSTRUCTIONS_AVAILABLE));

        // V8_CRYPTO covers AES + SHA1/SHA2 + PMULL together.
        let crypto = p(PF_ARM_V8_CRYPTO_INSTRUCTIONS_AVAILABLE);
        enable(Feature::aes, crypto);
        enable(Feature::pmull, crypto);
        enable(Feature::sha2, crypto);

        // ── Atomics & memory ordering ───────────────────────────────────
        let lse = p(PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE);
        enable(Feature::lse, lse);
        enable(Feature::lse2, p(PF_ARM_LSE2_AVAILABLE));
        enable(Feature::rcpc, p(PF_ARM_V83_LRCPC_INSTRUCTIONS_AVAILABLE));

        // ── ARMv8.2+ AdvSIMD ────────────────────────────────────────────
        let dotprod = p(PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE);
        enable(Feature::dotprod, dotprod);
        enable(Feature::jsconv, p(PF_ARM_V83_JSCVT_INSTRUCTIONS_AVAILABLE));
        enable(Feature::fp16, p(PF_ARM_V82_FP16_INSTRUCTIONS_AVAILABLE));
        enable(Feature::i8mm, p(PF_ARM_V82_I8MM_INSTRUCTIONS_AVAILABLE));
        enable(Feature::bf16, p(PF_ARM_V86_BF16_INSTRUCTIONS_AVAILABLE));

        // stdarch `sha3` is documented as "FEAT_SHA512 & FEAT_SHA3" — both
        // must be present. Microsoft exposes them as two separate flags.
        enable(
            Feature::sha3,
            p(PF_ARM_SHA3_INSTRUCTIONS_AVAILABLE)
                && p(PF_ARM_SHA512_INSTRUCTIONS_AVAILABLE),
        );

        // ── SVE family ──────────────────────────────────────────────────
        enable(Feature::sve, p(PF_ARM_SVE_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sve2, p(PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sve2p1, p(PF_ARM_SVE2_1_INSTRUCTIONS_AVAILABLE));
        // sve2-aes per ARM ARM = SVE_AES + SVE_PMULL128 together.
        enable(
            Feature::sve2_aes,
            p(PF_ARM_SVE_AES_INSTRUCTIONS_AVAILABLE)
                && p(PF_ARM_SVE_PMULL128_INSTRUCTIONS_AVAILABLE),
        );
        enable(Feature::sve2_bitperm, p(PF_ARM_SVE_BITPERM_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sve_b16b16, p(PF_ARM_SVE_B16B16_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sve2_sha3, p(PF_ARM_SVE_SHA3_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sve2_sm4, p(PF_ARM_SVE_SM4_INSTRUCTIONS_AVAILABLE));
        enable(Feature::f32mm, p(PF_ARM_SVE_F32MM_INSTRUCTIONS_AVAILABLE));
        enable(Feature::f64mm, p(PF_ARM_SVE_F64MM_INSTRUCTIONS_AVAILABLE));

        // ── SME family ──────────────────────────────────────────────────
        enable(Feature::sme, p(PF_ARM_SME_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme2, p(PF_ARM_SME2_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme2p1, p(PF_ARM_SME2_1_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme_b16b16, p(PF_ARM_SME_B16B16_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme_f16f16, p(PF_ARM_SME_F16F16_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme_f64f64, p(PF_ARM_SME_F64F64_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme_f8f16, p(PF_ARM_SME_F8F16_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme_f8f32, p(PF_ARM_SME_F8F32_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme_fa64, p(PF_ARM_SME_FA64_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme_i16i64, p(PF_ARM_SME_I16I64_INSTRUCTIONS_AVAILABLE));
        enable(Feature::sme_lutv2, p(PF_ARM_SME_LUTv2_INSTRUCTIONS_AVAILABLE));
        // SF8 = streaming-SVE-mode FP8.
        enable(Feature::ssve_fp8dot2, p(PF_ARM_SME_SF8DP2_INSTRUCTIONS_AVAILABLE));
        enable(Feature::ssve_fp8dot4, p(PF_ARM_SME_SF8DP4_INSTRUCTIONS_AVAILABLE));
        enable(Feature::ssve_fp8fma, p(PF_ARM_SME_SF8FMA_INSTRUCTIONS_AVAILABLE));

        // ── FEAT_RDM via architectural inference ────────────────────────
        // ARM ARM K.a §D17.2.91: "In an ARMv8.1 implementation, if
        // FEAT_AdvSIMD is implemented, FEAT_RDM is implemented." Both
        // FEAT_DotProd (v8.2-A) and FEAT_LSE (v8.1-A) are v8.1+ markers
        // and require v8.1-A baseline conformance. AdvSIMD is universally
        // implemented on every Windows-on-ARM SKU.
        //
        // Same inference adopted by .NET 10 (PR #109493, merged 2025-01,
        // shipped in `v10.0.0` at `src/native/minipal/cpufeatures.c:549`).
        enable(Feature::rdm, dotprod || lse);
    }
    value
}
