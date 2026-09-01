# 2026-08-31 core-web FetchHttpClient（P3 传输第一步）

- `cineharbor-core-web` 增 `src/fetch.rs`：**`FetchHttpClient` = `transport::HttpClient` 的 fetch 实现**，
  只用 js-sys + wasm-bindgen-futures（不引 web-sys）处理 method/headers/body → 全局 `fetch` → status/arrayBuffer/content-type。
- 版本对齐（crates.io 查证，避免拆两个 wasm-bindgen）：pinned `wasm-bindgen =0.2.126` 配套
  `wasm-bindgen-futures =0.4.76`、`js-sys =0.3.103`、`wasm-bindgen-test =0.3.76`（its deps 均 `=0.2.126`）。
- 关键调整：core `HttpClient` trait 由 `#[async_trait]` 改 **`#[async_trait(?Send)]`**（wasm JsFuture 的
  future 非 Send；native reqwest 仍 Send，协变满足）。三个 impl（`ReqwestHttpClient` / 测试 `NoopClient` /
  `FetchHttpClient`）同步 `?Send`；native 语义未变，5 测试 + workspace check 不回归。
- 薄客户端入口（wasm-bindgen async fn → JS Promise）：`addon_manifest_json` / `addon_catalog_json`
  经 `RemoteAddon<FetchHttpClient>`（这俩是 web 前端「addon HTTP 直连」的桥）。
- 验证（exit 0）：
  1. `cargo check -p cineharbor-core-web --target wasm32-unknown-unknown` / `cargo build` 绿。
  2. `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner cargo test -p cineharbor-core-web --target wasm32`：
     `fetch::tests::fetches_data_url_via_global_fetch` 通过（**wasm 真实跑在 node**，经全局 `fetch` 抓
     `data:application/json;base64`，断言 status 200 + body 字节 + content-type）。
  3. `wasm-bindgen --target web` glue 生成，导出 `addon_manifest_json` / `addon_catalog_json` / `core_version` / `default_sync_domains`。
  4. native `cargo test -p cineharbor-core --features "native-storage,native-http"` 5 绿；`cargo check --workspace` 绿。
- P3 剩余：Storage 的 IndexedDB 实现、web Worker + 薄客户端（getState/dispatch 式）、Service Worker（CORS/缓存/流）。