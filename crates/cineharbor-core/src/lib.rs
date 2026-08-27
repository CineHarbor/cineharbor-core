//! CineHarbor 生态核心门面。
//!
//! 聚合 storage / sync / profile / download 四个功能 crate，对外提供统一入口。
//! 对标 Stremio 的 `stremio-core`：后续（P2/P3）会把内容模型、addon 聚合与本地
//! 状态机逐步收敛到这里，作为各客户端的唯一数据面依赖。
//!
//! `cineharbor-local-service` 是独立二进制（本地守护服务 + addon host），不在此
//! 重导出，单独由桌面端作为 sidecar 使用。

pub use cineharbor_download as download;
pub use cineharbor_profile as profile;
pub use cineharbor_storage as storage;
pub use cineharbor_sync as sync;
