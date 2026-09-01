# 2026-08-31 core-web 桥全通（catalog→meta→streams 经 WASM + fetch）

- 补全薄客户端四端点桥：`addon_meta_json` / `addon_streams_json`（与已有 `addon_manifest_json` /
  `addon_catalog_json` 组成 manifest/catalog/meta/streams 完整数据面），均经 `RemoteAddon<FetchHttpClient>`。
- 端到端集成测试 `crates/cineharbor-core-web/tests/node_addon_flow.js`（durable）：
  1. node `http` 起一个 **mock Stremio addon**，serve `/manifest.json`、`/catalog/movie/top.json`、
     `/meta/movie/tt1.json`、`/stream/movie/tt1.json`（JSON 字段与 `cineharbor-addon-protocol` 严格对齐）。
  2. `wasm-bindgen --target nodejs` glue 生成后 `require` 进来，调四个桥（wasm 内经全局 `fetch` 直连 addon HTTP）。
  3. 断言 catalog.metas[0].id==tt1 → meta.meta.id==tt1 → streams.streams[0].url==cdn.test（全链）。
- 验证（exit 0）：`cargo build -p cineharbor-core-web --target wasm32` + `wasm-bindgen --target nodejs` +
  `CH_GLUE_DIR=/tmp/chweb-node node tests/node_addon_flow.js` → **"INTEGRATION OK: catalog -> meta ->
  streams 经 WASM core + fetch 全通"**。
- 即满足阶段 3 退出判据的 **WASM 侧**进一步：web 前端可由 WASM core 拿到一条 catalog→meta→streams 数据。
- P3 剩余：web Worker + 薄客户端（在真实 `cineharbor-web` 接四桥）、Service Worker（CORS/缓存/流）、Storage IndexedDB。