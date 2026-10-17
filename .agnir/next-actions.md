# cineharbor-core Next Actions — 2026-10-18

1. Require SDK 10ce024c9f5b1ea739fca9e239488ee4411f403c to pass two complete main CI executions, then verify this Core lock/pin candidate PR across fmt/check/test/clippy/WASM lanes and two final main runs.
2. Advance Web to Core final SHA and SDK 10ce024c9f5b1ea739fca9e239488ee4411f403c without changing media protocol/security; run full Web production/browser/Jest/CI twice. Advance Desktop last, preserving real installed smoke; run three-platform native/install CI twice at final SHA.
3. Resolve remaining Web Next/Swiper/PostCSS and Desktop rustls/unsound dependencies and complete runtime/production/security/license gates. Do not weaken checks.
4. Public signing/updater deferred to public release by Principal; release_ready=false and public_release_executed=false.
