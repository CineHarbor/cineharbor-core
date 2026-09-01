//! 内容模型（search / live 发现 DTO）。
//!
//! 这些类型与 `cineharbor-local-service` 中同名类型保持一致；local-service 将在
//! 切面阶段 1 逐步改为从此处复用并删除本地拷贝（见
//! `docs/plans/stremio-faithful-cutover-plan.md`）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub poster: String,
    pub episodes: Vec<String>,
    pub episodes_titles: Vec<String>,
    pub source: String,
    pub source_name: String,
    pub class: Option<String>,
    pub year: String,
    pub desc: Option<String>,
    pub type_name: Option<String>,
    pub douban_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ContentSuggestion {
    pub text: String,
    pub r#type: &'static str,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct SuggestionsResponse {
    pub suggestions: Vec<ContentSuggestion>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LiveChannel {
    pub id: String,
    #[serde(rename = "tvgId")]
    pub tvg_id: String,
    pub name: String,
    pub logo: String,
    pub group: String,
    pub url: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct LiveProgram {
    pub start: String,
    pub end: String,
    pub title: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiveEpgData {
    #[serde(rename = "tvgId")]
    pub tvg_id: String,
    pub source: String,
    pub epg_url: String,
    pub programs: Vec<LiveProgram>,
}