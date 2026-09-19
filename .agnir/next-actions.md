# cineharbor-core Next Actions

1. Run the 1.0.0 version-alignment branch through every independent CI lane. After merge, require two complete successful runs on the exact new main SHA.
2. Publish the verified new Core SHA to the release matrix and update Addon SDK's pinned Core revision; do not update downstream pins to an unverified candidate.
3. Complete the remaining local-service consumer/duplicate audit under ADR-0006 and retain only proven control/runtime surfaces.
4. Validate the final Core/native/WASM revision through Web product-path and Desktop sidecar integration after downstream pins move.
5. Complete security/dependency/license checks and evidence-bound checkpoint. Public release is authorized only after every hard seven-repository gate passes.

Continue autonomously under the Principal's 2026-09-19 release authorization.
