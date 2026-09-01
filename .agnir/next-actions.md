# cineharbor-core Next Actions

0. **提交并推送本次 Agnir 初始化 + P1 纯化改动**（`AGNIR.yaml` / `AGENTS.md` / `.agnir/` / README 段 / crates），均为未提交改动；push 前需用户授权。

1. P1→P3 长成 Stremio-core 等价物。**进度**：core 纯化 ✅（`model`/`sync`/`transport`/`addons` 纯模块 + wasm32 绿）；`cineharbor-core-web` 骨架 ✅；sync 纯模型入 core + 重导出 ✅；`core::addons::RemoteAddon` 派发 + merge 聚合 ✅；addon-host catalog/streams 收口走 core merge ✅；`core::storage` Storage trait + `native-storage`(sqlite) ✅；core-web `FetchHttpClient` + `addon_{manifest,catalog,meta,streams}_json` 四桥 ✅（wasm-bindgen-test node data-URL 往返 + `node_addon_flow.js` mock addon 端到端 catalog→meta→streams 全通）。**续**：Storage 的 IndexedDB 实现、web Worker + 薄客户端（在 `cineharbor-web` 接四桥）、Service Worker（CORS/缓存/流）。
2. P3 起 `cineharbor-local-service` 依赖 `cineharbor-addon-sdk`（git/crates.io 引入），见门面仓 `docs/adr/0003`。
3. `cargo check --workspace` / `cargo test --workspace`。已验证：`cargo test --workspace`、`cargo test -p cineharbor-core --features native-http`、wasm check/build 均绿。