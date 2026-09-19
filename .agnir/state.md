# cineharbor-core Current State

Target: CineHarbor **1.0.0**, release preparation in progress; **RELEASE_READY = false**. Current release authority and complete acceptance scope live in `CineHarbor/cineharbor/docs/releases/1.0.0/`.

## Implemented surfaces under validation

- `cineharbor-core`: pure model, sync types, transport abstraction, addon dispatch/merge and storage abstraction; optional native HTTP/storage implementations.
- `cineharbor-core-web`: wasm-bindgen bridge, FetchHttpClient and addon manifest/catalog/meta/streams entrypoints.
- Native storage, sync, profile, download, addon-host and local-service crates remain in this workspace. Duplicate local-service content implementations must be audited against all Web/Desktop consumers before retirement.
- ADR-0006 target is in-process native/WASM core plus remote addons, not a mandatory native RPC daemon for Web content. Historical native-RPC descriptions are superseded.

## CI and reproducibility

`ci/dependency.json` pins the sibling SDK revision. `scripts/ci-checkout.py` creates its explicit sibling layout and verifies the immutable revision. Rust is pinned to 1.98.1 with rustfmt, clippy and wasm32. CI has independent fmt, check, test, clippy and WASM lanes with fail-fast disabled. Formatting selects every owned workspace member and never formats sibling dependencies. The maintenance workflow publishes only deterministic owned Rust formatting plus checkpoint evidence and dispatches CI on the resulting revision; formatting success is not build/runtime acceptance.

Baseline `05989fb6f2fe388d8eaeb4128445ac4c61662f5a`, run `35382333866`: fmt failed and downstream check/test/clippy were skipped. New gates and formatting repair are pending observed execution. No previous locally reported tests are promoted to current main release evidence.

## Continuity

Project `urn:cineharbor:project:cineharbor-core`; lineage `urn:cineharbor:lineage:cineharbor-core`. Agnir Core/Profile 1.0 / repository-filesystem/1.0, operations v1.0.2 at `b5626394ec40a5cb7a28c01892acde07cc0adc8e`, remain unchanged. License baseline: CC-BY-NC-SA-4.0. No developer-specific absolute cache path or uncommitted-initialization prerequisite is required.
