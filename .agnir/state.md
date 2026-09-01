# cineharbor-core Current State

CineHarbor 的 Rust 核心：本地数据面与守护服务，对应 Stremio `stremio-core`。按 ADR-0006（终态对齐 Stremio）演进：`cineharbor-core` 纯化为「纯状态机」，native + wasm32 双编译。

- crates：`cineharbor-core`（纯状态机：`model`/`sync`/`transport`/`addons`/`storage` 纯模块，经 `cineharbor-addon-protocol` 跨仓依赖，无 IO 重依赖，wasm 干净；`native-http` feature 含 reqwest 传输实现、`native-storage` feature 含 sqlite 存储实现）、`cineharbor-core-web`（wasm-bindgen 桥，对标 `stremio-core-web`，cdylib；含 fetch 版 `HttpClient`（`FetchHttpClient`）+ `addon_{manifest,catalog,meta,streams}_json` 薄客户端四桥 + `tests/node_addon_flow.js` 端到端集成）、`cineharbor-storage`（sqlite）、`cineharbor-sync`（reqwest 客户端 + 重导出 `core::sync` 纯类型）、`cineharbor-profile`（配置与鉴权）、`cineharbor-download`（下载）、`cineharbor-local-service`（守护服务 + addon host）。
- 构建：`cargo check --workspace` / `cargo test --workspace`；core 纯逻辑 `cargo check -p cineharbor-core --target wasm32-unknown-unknown`；core→wasm 产物 `cargo build -p cineharbor-core-web --target wasm32-unknown-unknown` + `wasm-bindgen --target web`。
- 许可证：CC BY-NC-SA 4.0。

> 环境：沙箱 `workspace-write` 下 `~/.cargo` 只读；需要下载新 crate 时用 `CARGO_HOME=/Users/jay/Code/CineHarbor/.cargo-home`。

- Agnir 操作基线：`iorLab/agnir` 稳定发布 `v0.1.0`（revision `2a0cb7bf2068b11f361e315670b2f2dc497b2588`，distribution `agnir-agent-skill`），2026-09-01 兼容操作升级。