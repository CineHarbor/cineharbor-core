# Owned Rust formatting checkpoint

Baseline: `be9c7126093d24fdc7a28fa8454e2933d6670884`

Normalized all owned workspace members with the pinned toolchain and verified `scripts/format-owned.py --check`. Sibling sources were not modified. This is formatting evidence only; compile, tests, security and release acceptance remain separate gates. Project identity and selected lineage are unchanged.

Changed sources:
- `crates/cineharbor-addon-host/src/lib.rs`
- `crates/cineharbor-core-web/src/fetch.rs`
- `crates/cineharbor-core-web/src/lib.rs`
- `crates/cineharbor-core/src/addons.rs`
- `crates/cineharbor-core/src/lib.rs`
- `crates/cineharbor-core/src/model.rs`
- `crates/cineharbor-core/src/storage.rs`
- `crates/cineharbor-core/src/sync.rs`
- `crates/cineharbor-core/src/transport.rs`
- `crates/cineharbor-sync/src/lib.rs`
