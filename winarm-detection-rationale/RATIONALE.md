# Windows AArch64 feature-detection rationale

This branch (`winarm-feature-detection`) replaces
`library/std_detect/src/detect/os/windows/aarch64.rs` with a version that:

1. Probes every `PF_ARM_*` constant defined in Windows SDK 10.0.26100.0
   (`winnt.h`) — 56 constants, mapping to 36 stdarch feature names.
2. Derives `rdm` from `PF_ARM_V82_DP` *or* `PF_ARM_V81_ATOMIC` via the ARM ARM
   architectural rule (D17.2.91), matching what .NET 10 ships in production.
3. Performs **zero registry reads, zero ID-register decoding, zero regex,
   zero string parsing**. Each probe is one `IsProcessorFeaturePresent`
   syscall returning a `BOOL`.

`detect_features()` issues exactly **39 IPFP calls** at first probe (verified
via `cargo rustc --emit=asm --target aarch64-pc-windows-msvc -C opt-level=3`)
and is cached by the existing `cache::Initializer` mechanism. No new
dependencies; the file is alloc-free.

## Trust hierarchy

Every feature bit set by this file traces to one of three authority tiers,
each backed by a primary source:

| Tier | Source | Used for |
|---|---|---|
| **T0 — Microsoft IPFP** | `IsProcessorFeaturePresent(PF_ARM_*)` | 36 features directly |
| **T1 — ARM ARM strict implication** | ARM ARM K.a §B2.2 / §D17.2.91 | `rdm` derived from `dotprod`/`lse` |
| **T2 — Industry-consensus inference** | .NET 10 `cpufeatures.c:549-563` | Same `rdm` derivation; precedent established by `dotnet/runtime#109493` |

We **deliberately do not use**:

- **Tier 4 — Architecture-level mandatory inference** (e.g. `lse2 → dpb`).
  Falsifiable by chips that backport features early. .NET PR #109493 review
  (a74nh) explicitly considered and rejected this category.
- **Tier 5 — Windows-version baseline** ("Win11 ⇒ X"). Same .NET PR review
  rejected this approach: conflates OS version with hardware capability and
  can break on hypothetical future SKUs.

For features that genuinely require ID-register decoding (the ~30 stdarch
names with no `PF_ARM_*` constant — `fhm`, `fcma`, `frintts`, `paca`/`pacg`,
`bti`, `dpb`/`dpb2`, `mte`, `mops`, `dit`, `sb`, `ssbs`, `flagm`/`flagm2`,
`rand`, `cssc`, `wfxt`, `hbc`, `sm4`, `rcpc2`/`rcpc3`, `pauth_lr`, `lse128`,
`tme`, `ecv`, `lut`, `faminmax`, `fp8*`, `fpmr`), we leave them at the
upstream `false` default. A separate crate
([`winarm-cpufeatures`](https://github.com/imazen/winarm-cpufeatures))
offers them behind an opt-in `registry` Cargo feature for callers that
specifically need them. Same trust tier as LLVM's
[`llvm-project#151596`](https://github.com/llvm/llvm-project/pull/151596).

## Why `rdm` specifically

Microsoft has never defined a `PF_ARM_RDM_*` constant in any Windows SDK,
including the latest 10.0.26100.0. `dotnet/runtime#74778` ("RCPC, DC ZVA and
probably RDM ISAs are never detected on win-arm64", 2022-08-29) records the
gap; the resolution PR
[`dotnet/runtime#109493`](https://github.com/dotnet/runtime/pull/109493)
documents that *"Windows has no plans to expose FEAT_RDM detection"*.

Since FEAT_RDM is **mandatory** in ARMv8.1-A whenever FEAT_AdvSIMD is
implemented (ARM ARM K.a §D17.2.91), and FEAT_AdvSIMD is universal on
Windows-on-ARM SKUs, any IPFP probe that confirms a v8.1-A or higher feature
also confirms `rdm`. The two cleanest such markers are:

- `PF_ARM_V81_ATOMIC` (#34) ⇒ FEAT_LSE — v8.1-A mandatory
- `PF_ARM_V82_DP` (#43) ⇒ FEAT_DotProd — v8.2-A (which subsumes v8.1-A)

We OR both. Either confirms `rdm` with an architecturally-mandated implication.

This is the exact same inference Microsoft adopted in their own .NET runtime
for the same reason — see the .NET 10 reference document next to this file.

## Files in this directory

- `RATIONALE.md` (this file) — design rationale and trust hierarchy
- `dotnet10_reference.md` — verbatim source extraction from
  `dotnet/runtime` v10.0.0 GA (commit `60629d14`), file-by-file +
  per-feature trace tables. Treat as authoritative for "how does
  a major Microsoft-maintained runtime handle ARM detection."

## Status

This branch is a draft pushed to the fork for review. **No upstream PR has
been opened.** The patch is ready for hardware validation on a real
Windows-on-ARM machine — clone, apply via `x.py build library/std`, and run
your AArch64 test suite.

## How to build a custom toolchain

```bash
git clone https://github.com/imazen/rust --branch winarm-feature-detection
cd rust
./x.py build library/std --target aarch64-pc-windows-msvc
rustup toolchain link winarm build/host/stage1
cargo +winarm test --target aarch64-pc-windows-msvc
```

Or for a quicker iteration without a full toolchain build, copy
`library/std_detect/src/detect/os/windows/aarch64.rs` over the same path
in `$(rustc +nightly --print sysroot)/lib/rustlib/src/rust/` and use
`-Z build-std=std,panic_abort` with `cargo +nightly`.
