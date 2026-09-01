//! CineHarbor core 的 wasm-bindgen 桥（对标 Stremio 的 `stremio-core-web`）。
//!
//! 把 `cineharbor-core` 的纯模型/状态机暴露为 web 可消费的 WASM 产物。本 crate 只做
//! 绑定与 JSON 序列化，不含业务逻辑；业务逻辑在 `cineharbor-core`（native + wasm 双编译）。

use wasm_bindgen::prelude::*;

use cineharbor_core::model::{SearchResponse, SearchResult};
use cineharbor_core::sync::default_profile_sync_selected_domains;

mod fetch;
pub use fetch::FetchHttpClient;

#[wasm_bindgen]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 演示：构造一个 `SearchResult` 并序列化为 JSON（证明 serde 在 wasm 可用）。
#[wasm_bindgen]
pub fn demo_search_result_json() -> String {
    let result = SearchResult {
        id: "tt0133093".into(),
        title: "The Matrix".into(),
        poster: String::new(),
        episodes: Vec::new(),
        episodes_titles: Vec::new(),
        source: "demo".into(),
        source_name: "Demo".into(),
        class: None,
        year: "1999".into(),
        desc: None,
        type_name: Some("movie".into()),
        douban_id: None,
    };
    let response = SearchResponse {
        results: vec![result],
    };
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".into())
}

/// 演示：默认同步域（纯函数在 wasm 可用）；`Vec<String>` 映射为 JS `string[]`。
#[wasm_bindgen]
pub fn default_sync_domains() -> Vec<String> {
    default_profile_sync_selected_domains()
}