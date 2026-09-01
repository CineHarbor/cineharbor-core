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

## Agnir Project Instructions

本项目使用 **Agnir**（project-owned durable continuity protocol）持久保存可恢复的 Project 连续性，本仓库根目录是已授权的 Project Entry Point。开始任何 Project 工作前：

1. 读取顶层 `AGNIR.yaml`；
2. 加载 Current State（`.agnir/state.md`）与 Next Actions（`.agnir/next-actions.md`）；
3. 需要时再加载 Decisions（`.agnir/decisions.md`）与 Evidence（`.agnir/evidence/`）；
4. durable Agnir Project truth 优先于聊天记录与 Agent 私有记忆，除非被更新的 Principal 指令或直接观测到的当前 Project 事实覆盖；
5. 在保存进度、checkpoint 或结束工作时，把重要的 state / next-action / decision / evidence 变更写回 `AGNIR.yaml` 声明的 durable memory 位置。
6. 在 repository / VCS 上下文中，把已授权的 `commit`、`提交`、`提交代码` 或同义请求视为 checkpoint boundary：先 reconcile Agnir 再 commit，优先把 Project 改动与 Agnir 改动放进同一 revision；`commit and push`、`提交推送` 或同义请求表示 checkpoint + commit + push，并在声明了 authoritative ref 时验证推送结果。
