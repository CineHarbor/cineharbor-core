# Locked media-client integration — 2026-09-20

Base: `CineHarbor/cineharbor-core@d51414bba4964dd7cee02780e41c73e35190ce76`, tree `b822d68a7dbfdbf181f8b3fcec6b443634284bb0`. Latest owned source snapshot ZIP: `e10fe235fb775147f8d98c42ebb867c6b6872541fc2f3d566bdfdf5398bb2dc8`; Core archive tree matched its Git object tree. Agnir identity/lineage/profile/operations are unchanged.

SDK input: `3ab4ff8fcc38a6f0849389a7c6b66291f3ca341d`; mandatory fmt/check/test/strict-Clippy steps passed in both main runs `35501487450` and `35501530721`. The repeat-dispatch helper's expected skip in the second run is not a skipped product gate.

## Candidate code blobs

- `Cargo.lock`: `c59a2b08f6bba42761bec883e05071c27f90c404`
- `ci/dependency.json`: `1050fd1e23636825e75f5e50245c39365de5d2d1`
- `crates/cineharbor-core-web/src/fetch.rs`: `e67575968fede7647f592da64f8bccb414bef53e`

Only the two sibling owned SDK/protocol lock versions and SDK revision pin change. Fetch privacy options are explicit; signing secrets never enter the Web bridge.

## Observed local verification

Rust 1.98.1 and wasm-bindgen 0.2.126, offline locked registry from verified release-toolbox ZIP `7635b8d1784ce026ceed4ef5118eed28a2fbf8577127d5d75bf5b4bb80f1e7e5`.

- `python3 scripts/format-owned.py --check`
- `cargo check --workspace --all-targets --locked`
- `cargo test --workspace --all-targets --all-features --locked`: 161 native tests passed, 0 failed/ignored.
- `cargo test --workspace --doc --all-features --locked`: completed, no executable examples.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- pure-core no-default-feature WASM check and core-web locked release WASM build.
- Actual generated bridge under Node: meta + two repeated stream requests invoke fetch three times with no-store/omit/no-referrer and preserve the capability URL byte-for-byte. Prior Fetch implementation fails the no-store assertion (exit 1); restored code and all gates pass again. Web's required smoke suite will retain this integration check.

This is build/fixture transport evidence, not real-browser playback, deployed egress, native installed/updater or full release acceptance. No test/lint/workflow gate is suppressed. PR and repeated post-merge CI remain required. The complete candidate (code and these continuity files) is prepared before publication; publication must reject a changed base, verify its destination, and fresh-resolve the same lineage.
