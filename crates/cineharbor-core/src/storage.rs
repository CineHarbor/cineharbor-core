//! 平台无关的键值存储抽象（core 状态机的 Storage 平台缝，对标 Stremio-core `Storage`）。
//!
//! 供 core 纯逻辑（profile / favorites / 播放记录等持久化）使用。本模块 wasm 干净：
//! native 实现走 sqlite（feature `native-storage`，默认关闭）；wasm 实现走 IndexedDB
//! （阶段 3，与 `HttpClient` 的 fetch 同步接入）。键的命名空间由调用方用前缀约定
//! （如 `profile:`、`favorite:`）。

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StorageError {
    #[error("存储未初始化: {0}")]
    NotInitialized(String),
    #[error("存储操作失败: {0}")]
    Operation(String),
}

#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
    async fn set(&self, key: &str, value: &[u8]) -> Result<(), StorageError>;
    async fn remove(&self, key: &str) -> Result<(), StorageError>;
}

#[cfg(feature = "native-storage")]
pub use sqlite_storage::SqliteStorage;

#[cfg(feature = "native-storage")]
mod sqlite_storage {
    use std::sync::{Arc, Mutex};

    use super::{Storage, StorageError};

    /// SQLite 键值存储（`kv(key TEXT PRIMARY KEY, value BLOB)`），异步经 `spawn_blocking`。
    #[derive(Clone)]
    pub struct SqliteStorage {
        conn: Arc<Mutex<rusqlite::Connection>>,
    }

    impl SqliteStorage {
        pub fn open(path: &str) -> Result<Self, StorageError> {
            let conn = rusqlite::Connection::open(path)
                .map_err(|error| StorageError::NotInitialized(error.to_string()))?;
            Self::init(conn)
        }

        pub fn in_memory() -> Result<Self, StorageError> {
            let conn = rusqlite::Connection::open_in_memory()
                .map_err(|error| StorageError::NotInitialized(error.to_string()))?;
            Self::init(conn)
        }

        fn init(conn: rusqlite::Connection) -> Result<Self, StorageError> {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value BLOB NOT NULL);",
            )
            .map_err(|error| StorageError::NotInitialized(error.to_string()))?;
            Ok(Self {
                conn: Arc::new(Mutex::new(conn)),
            })
        }
    }

    #[async_trait::async_trait]
    impl Storage for SqliteStorage {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
            let conn = Arc::clone(&self.conn);
            let key = key.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = conn
                    .lock()
                    .map_err(|_| StorageError::Operation("连接锁被污染".to_string()))?;
                let mut statement = conn
                    .prepare("SELECT value FROM kv WHERE key = ?1")
                    .map_err(|error| StorageError::Operation(error.to_string()))?;
                let mut rows = statement
                    .query([&key])
                    .map_err(|error| StorageError::Operation(error.to_string()))?;
                let value = rows
                    .next()
                    .map_err(|error| StorageError::Operation(error.to_string()))?
                    .map(|row| row.get::<_, Vec<u8>>(0))
                    .transpose()
                    .map_err(|error| StorageError::Operation(error.to_string()))?;
                Ok(value)
            })
            .await
            .map_err(|error| StorageError::Operation(error.to_string()))?
        }

        async fn set(&self, key: &str, value: &[u8]) -> Result<(), StorageError> {
            let conn = Arc::clone(&self.conn);
            let key = key.to_string();
            let value = value.to_vec();
            tokio::task::spawn_blocking(move || {
                let conn = conn
                    .lock()
                    .map_err(|_| StorageError::Operation("连接锁被污染".to_string()))?;
                conn.execute(
                    "INSERT INTO kv (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    rusqlite::params![&key, &value],
                )
                .map_err(|error| StorageError::Operation(error.to_string()))?;
                Ok(())
            })
            .await
            .map_err(|error| StorageError::Operation(error.to_string()))?
        }

        async fn remove(&self, key: &str) -> Result<(), StorageError> {
            let conn = Arc::clone(&self.conn);
            let key = key.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = conn
                    .lock()
                    .map_err(|_| StorageError::Operation("连接锁被污染".to_string()))?;
                conn.execute("DELETE FROM kv WHERE key = ?1", rusqlite::params![&key])
                    .map_err(|error| StorageError::Operation(error.to_string()))?;
                Ok(())
            })
            .await
            .map_err(|error| StorageError::Operation(error.to_string()))?
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{SqliteStorage, Storage};

        #[tokio::test]
        async fn sqlite_round_trips() {
            let storage = SqliteStorage::in_memory().unwrap();
            let key = "profile:username";

            assert_eq!(storage.get(key).await.unwrap(), None);

            storage.set(key, b"alice").await.unwrap();
            assert_eq!(storage.get(key).await.unwrap(), Some(b"alice".to_vec()));

            storage.set(key, b"bob").await.unwrap();
            assert_eq!(storage.get(key).await.unwrap(), Some(b"bob".to_vec()));

            storage.remove(key).await.unwrap();
            assert_eq!(storage.get(key).await.unwrap(), None);
        }
    }
}