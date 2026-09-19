# cineharbor-core Current State

Target: CineHarbor **1.0.0 public release**, with release gates enforced. **RELEASE_READY = false; PUBLIC_RELEASE_EXECUTED = false.** Current release authority and acceptance scope live in `CineHarbor/cineharbor/docs/releases/1.0.0/`.

## Implemented surfaces

- `cineharbor-core`: pure model, sync types, transport abstraction, addon dispatch/merge and storage abstraction; optional native HTTP/storage implementations.
- `cineharbor-core-web`: wasm-bindgen bridge, FetchHttpClient and addon manifest/catalog/meta/streams entrypoints.
- Native storage, sync, profile, download, addon-host and local-service crates remain in this workspace.
- ADR-0006 target is in-process native/WASM core plus remote addons, not a mandatory native RPC daemon for Web content.

## Final pre-version validation

Main `e2bb2c6cab5cf620ab4eef34e05b254b26dc3139` passed the complete Core CI matrix twice: push run `35420232092` and clean workflow-dispatch run `35420280323`. Those runs include fmt, check, native tests/doc tests, strict Clippy and WASM compilation.

## 1.0.0 version alignment

The release branch changes the Core workspace package version from `0.1.0` to `1.0.0` and updates exactly the eight owned workspace package entries in `Cargo.lock`: addon-host, core-web, core, download, local-service, profile, storage and sync. The sibling addon protocol/SDK lock entries are deliberately not rewritten because they come from the independently pinned sibling repository.

This is a release-metadata change, so predecessor CI does not certify it. Complete PR CI and two post-merge main executions on one immutable SHA are required. Downstream Addon SDK/Web/Desktop pins must be updated only after the new Core main SHA is verified.

See `.agnir/evidence/2026-09-19-version-1.0.0.md`.

## Continuity

Project `urn:cineharbor:project:cineharbor-core`; lineage `urn:cineharbor:lineage:cineharbor-core`. Agnir Core/Profile 1.0 / repository-filesystem/1.0, operations v1.0.2 at `b5626394ec40a5cb7a28c01892acde07cc0adc8e`, remain unchanged. License baseline: CC-BY-NC-SA-4.0.
