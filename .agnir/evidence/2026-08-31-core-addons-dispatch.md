# 2026-08-31 core::addons 派发（transport-agnostic remote addon）落地

- 新增 `cineharbor-core/src/addons.rs`（对标 Stremio-core 的 addon transport/dispatch）：
  - `RemoteAddon<C: HttpClient>`：基于 `core::transport::HttpClient` 的 remote addon 客户端，manifest/catalog/meta/streams/subtitles；URL 规则（`.json` 后缀、`skip=` / `name=value`）对齐 addon-sdk `AddonClient`；meta 404 → `Ok(None)`。
  - `merge_catalogs` / `merge_streams` / `first_meta`：纯聚合函数（addon-host 与未来 web worker 共用）。
  - `RemoteAddonError`：InvalidBaseUrl / Http / Json / Status。
- `cineharbor-core` 新增跨仓依赖 `cineharbor-addon-protocol`（`../../../cineharbor-addon-sdk/crates/cineharbor-addon-protocol`，纯 serde 类型，wasm 干净，ADR-0003）。
- 验证（exit 0）：`cargo test -p cineharbor-core`（2 纯逻辑测试）；`cargo test -p cineharbor-core --features native-http`（4 测试，含 `RemoteAddon` 走 `ReqwestHttpClient` 打 axum mock addon 的端到端往返）；`cargo check -p cineharbor-core --target wasm32-unknown-unknown`；`cargo check --workspace`。

## 与 addon-sdk `AddonClient` 的关系
addon-sdk 的 `AddonClient`（reqwest 绑定）保留给服务端/CLI 直接使用；`core::addons::RemoteAddon` 是 core 内的 transport-agnostic 等价物，WASM 端将用 `fetch` 实现 `HttpClient` 复用同一套派发逻辑（阶段 3）。