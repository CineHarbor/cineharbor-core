# 2026-08-31 cineharbor-core 纯化 + wasm 桥进度

门面仓 ADR-0006（终态对齐 Stremio）在本仓库的落地，多轮累进，均已验证（exit 0）：

## 已完成
1. `cineharbor-core` 纯化：拆掉 storage/sync/profile/download 等 IO 依赖，只留 `http`/`serde`/`serde_json`/`thiserror`/`url`（均 wasm 干净）。
2. `core::model`（内容模型：SearchResult/SearchResponse/ContentSuggestion/LiveChannel/LiveProgram/LiveEpgData）+ `core::sync`（profile-sync 纯模型 + 无 IO 函数）。
3. `cineharbor-sync`：纯类型/函数改 `pub use cineharbor_core::sync::{...}` 重导出，reqwest `ProfileSyncClient` 留在原地；local-service 消费方零改动。
4. `core::transport`：异步 `HttpClient` trait + `HttpRequest/HttpResponse/HttpError`（`async-trait` + `http` 类型，wasm 干净）；`native-http` feature 提供 `ReqwestHttpClient` 实现，含 axum mock 往返测试。
5. `cineharbor-core-web`（对标 `stremio-core-web`）：cdylib + wasm-bindgen，暴露 `core_version`/`demo_search_result_json`/`default_sync_domains`。
6. 验证：`cargo test --workspace`、`cargo test -p cineharbor-core --features native-http`、`cargo check -p cineharbor-core --target wasm32-unknown-unknown`、`cargo build -p cineharbor-core-web --target wasm32-unknown-unknown` + `wasm-bindgen --target web` 均绿，导出符号可见。

## 待办
- core 加 addon 聚合/派发（依赖 `cineharbor-addon-protocol` 纯类型）。
- storage 抽象 trait（sqlite native / IndexedDB wasm）。
- `native-http` 供 `cineharbor-sync`、local-service 的抓取 HTTP 采用。
- web worker 桥接 + 抓取/媒体代理外置 remote addon（门面仓计划阶段 2/3）。

## 环境注意
沙箱 `workspace-write` 下 `~/.cargo` 只读；下载新 crate 用 `CARGO_HOME=/Users/jay/Code/CineHarbor/.cargo-home`（workspace 本地）。