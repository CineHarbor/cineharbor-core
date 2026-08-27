# cineharbor-core

CineHarbor 的 Rust 核心：本地数据面与守护服务。对应 Stremio 的 `stremio-core`。

## crate

| crate | 职责 |
| --- | --- |
| `cineharbor-core` | 核心门面：聚合 storage/sync/profile/download 的公共数据模型与 API（P1→P3 长成 Stremio-core 等价物） |
| `cineharbor-storage` | 本地持久化（sqlite） |
| `cineharbor-sync` | 云端/跨端同步 |
| `cineharbor-profile` | 用户配置与鉴权 |
| `cineharbor-download` | 下载执行器 |
| `cineharbor-local-service` | 本地守护服务 + addon host（二进制） |

## 构建

```bash
cargo check --workspace
cargo test --workspace
```

## 依赖

P3 起 `cineharbor-local-service` 将依赖 [cineharbor-addon-sdk](../cineharbor-addon-sdk)（addon 协议 + host 接线），通过 git/crates.io 引入，详见门面仓 `docs/adr/0003`。

## 许可证

CC BY-NC-SA 4.0
