# 1.0.0 CI foundation checkpoint

Baseline: `05989fb6f2fe388d8eaeb4128445ac4c61662f5a`. The seven-repository source audit (`CineHarbor/cineharbor`, run `35417560069`, artifact `10576242656`) reproduced this revision; archive SHA-256 was verified before source inspection. Read AGENTS/AGNIR/AGNIR.yaml and selected state/next-actions/decisions. Identity and lineage are unchanged.

Observed failure: Core Actions run `35382333866` job `105721187947` fails `cargo fmt --all -- --check`, including sibling SDK sources, then skips check/test/clippy. This checkpoint pins the toolchain and SDK dependency, restricts formatting to all owned workspace members, adds separate non-fail-fast CI lanes and deterministic formatting normalization with conflict detection. No tests are removed or marked passing. Python syntax and workflow YAML were checked locally; actual Rust gates still require Actions results.

The current executor cannot clone or install dependencies over its local network, so the existing authorized GitHub Actions environment executes these checks. No credential values are collected or committed. Formatting maintenance can only stage changed `crates/**/*.rs` plus its explicit Agnir evidence, rejects unexpected changes, refuses stale main publication, verifies destination ref and requests new CI on the published source.
