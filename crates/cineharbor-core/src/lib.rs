//! CineHarbor 生态核心门面（纯状态机库，目标对标 `stremio-core`）。
//!
//! 本 crate 只承载**平台无关的纯逻辑**：内容模型、库/同步、profile、addon 聚合与
//! 派发的数据类型与算法。不含网络 I/O、不含 sqlite、不含媒体代理——这些属于平台
//! 实现（native：sqlite/reqwest）或 remote addon（Stremio 协议 HTTP），见
//! `docs/adr/0006-stremio-faithful-core-wasm.md` 与
//! `docs/plans/stremio-faithful-cutover-plan.md`。
//!
//! 双编译目标：native + `wasm32-unknown-unknown`。

pub mod addons;
pub mod model;
pub mod storage;
pub mod sync;
pub mod transport;