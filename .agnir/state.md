# cineharbor-core Current State

CineHarbor 的 Rust 核心：本地数据面与守护服务，对应 Stremio `stremio-core`。

- crates：`cineharbor-core`（公共数据模型/API 门面）、`cineharbor-storage`（sqlite）、`cineharbor-sync`（云端/跨端）、`cineharbor-profile`（配置与鉴权）、`cineharbor-download`（下载）、`cineharbor-local-service`（守护服务 + addon host）。
- 构建：`cargo check --workspace` / `cargo test --workspace`。
- 许可证：CC BY-NC-SA 4.0。
