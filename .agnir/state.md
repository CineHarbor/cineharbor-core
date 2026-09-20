# cineharbor-core Current State

Target: **1.0.0 release-ready** under the canonical facade scope. **RELEASE_READY=false; PUBLIC_RELEASE_EXECUTED=false.** This checkpoint does not authorize public publication.

## Repository boundary

Pure model/transport/addon dispatch and the browser WASM bridge live here, alongside native storage, sync, profile, download, addon-host and local-service. ADR-0006 keeps Web's content path independent of a mandatory native RPC daemon. Remaining native helper/consumer retirement still needs evidence.

## Media-client integration — 2026-09-20

Base main is `d51414bba4964dd7cee02780e41c73e35190ce76` (owned crates already 1.0.0). The current candidate pins Addon SDK to `3ab4ff8fcc38a6f0849389a7c6b66291f3ca341d`, whose complete main Rust matrix was independently inspected twice at runs `35501487450` and `35501530721`. Exactly the two sibling protocol/SDK Cargo.lock entries are aligned from 0.1.0 to 1.0.0; no third-party dependency or owned package version changes.

The real WASM FetchHttpClient now uses `cache: no-store`, `credentials: omit`, and `referrerPolicy: no-referrer`. Addon meta/stream requests carrying expiring resource capabilities must be renewed at the source without browser HTTP-cache reuse or ambient credentials. Applications still must bypass persistent Service Worker metadata caches and preserve server-issued media URLs.

Local exact-source validation passed owned fmt, locked all-target check, all-target/all-feature native tests (161 passed; no failures or ignores), doc-test execution, strict Clippy, pure-core WASM check and release WASM bridge compilation. Running the generated bridge under Node confirmed three actual transport calls and unchanged signed stream bytes. Restoring the predecessor Fetch implementation caused the no-store assertion to fail; restoring the repair and rerunning all local gates passed.

This candidate still requires PR CI and two successful full main runs after merge before downstream pins move. Browser/player/production/installed native/updater and final release acceptance are not certified by these local checks. See `.agnir/evidence/2026-09-20-media-client-alignment.md`.

## Continuity

Project `urn:cineharbor:project:cineharbor-core`; lineage `urn:cineharbor:lineage:cineharbor-core`. Agnir Core/Profile 1.0 / repository-filesystem/1.0 and operations 1.0.2 at `b5626394ec40a5cb7a28c01892acde07cc0adc8e` are unchanged. License baseline CC-BY-NC-SA-4.0.
