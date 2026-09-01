# 2026-08-31 core Storage 平台缝（P1.c2 收尾）

- `cineharbor-core/src/storage.rs` 新增 **wasm 干净的 `Storage` 平台缝**（对标 Stremio-core `Storage`）：
  - `trait Storage: Send + Sync { get / set / remove }`（`Vec<u8>` 值，键命名空间由调用方前缀约定）。
  - `StorageError { NotInitialized, Operation }`（thiserror）。
- native 实现 `SqliteStorage`（feature `native-storage`，默认关闭）：
  - `kv(key TEXT PRIMARY KEY, value BLOB)`；`open(path)` / `in_memory()`；三个操作经 `tokio::task::spawn_blocking`
    包 rusqlite 同步调用（`Arc<Mutex<Connection>>`），失败路径完整（open/init/prepare/query/execute 均 map_err）。
- feature：`native-storage = ["dep:rusqlite", "dep:tokio"]`（core 本体对 wasm 依旧干净，见 [2/3]）。
- 验证（exit 0）：`cargo test -p cineharbor-core --features "native-storage,native-http"` 5 测试绿（含新
  `sqlite_round_trips`：get(None)→set(alice)→set(bob 覆盖)→remove→get(None)）；`cargo check --target
  wasm32-unknown-unknown -p cineharbor-core` 干净；`cargo check --workspace` 干净。
- 至此 HttpClient ✅ + Storage ✅（objective 的「平台缝 via traits」两条都落地）。wasm 侧 Storage（IndexedDB）
  与 fetch HttpClient 同批在 P3 接入（wasm-bindgen-futures/js-sys 版本需与 wasm-bindgen 0.2.126 对齐，先查再写）。