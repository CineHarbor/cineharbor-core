# cineharbor-core Next Actions

1. Verify the published lint-fix revision through every independent CI lane and its automatic second clean main run. Local strict clippy and native tests passed; only observed remote runs count as current main evidence.
2. Audit local-service addon_douban/live/vod, content search/detail, proxies and prefetch consumers. Move remaining production consumers to core/remote addons before deleting duplicates.
3. Validate pure-core/native/WASM compatibility and protocol/browser consumers after each cross-repository change; deliberately update pinned dependency revisions when required.
4. Produce and verify the release sidecar for macOS arm64/x64 and Windows x64; validate startup, version/health, crash/shutdown and desktop integration.
5. Complete security/dependency/license checks, version alignment and accurate evidence-bound checkpoint. No release-ready claim until all release gates (including real desktop upgrade and production smoke) pass.

Continue automatically under the Principal's 2026-09-19 release authorization. No public release is authorized by this preparation run. Agnir identity/lineage and historical evidence remain unchanged.
