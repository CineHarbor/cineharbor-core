# cineharbor-core Next Actions

1. Require all five independent PR CI lanes, then two complete executions at the exact merged main SHA for this SDK lock/fetch-policy integration. Do not project predecessor results onto it.
2. Advance Web to the verified Core main plus SDK 3ab4ff8, migrate opaque media capabilities and cache/renewal behavior, then advance Desktop to the verified frontend/Core/SDK unit.
3. Finish ADR-0006 consumer audit without deleting live capabilities. Verify retained native control/service authentication and diagnostics.
4. Validate browser/native/download paths, security/dependencies/licenses and signed installed/updater/data-retention acceptance in the seven-repository release matrix.
5. Keep the canonical facade evidence current. Final public publication is outside the preparation scope; release_ready remains false until every mandatory acceptance gate is observed passing.
