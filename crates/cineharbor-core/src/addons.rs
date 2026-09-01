//! Remote addon 派发与聚合（Stremio-core 的 addon transport / dispatch 等价物）。
//!
//! 通过 [`crate::transport::HttpClient`] 走 HTTP，不绑定 reqwest/fetch：native 用
//! `ReqwestHttpClient`（feature `native-http`），wasm 用 `fetch`（阶段 3）。端点契约与
//! URL 规则（`.json` 后缀、`skip=N` / `name=value`）对齐 `cineharbor-addon-sdk::client::AddonClient`。

use http::StatusCode;
use serde::de::DeserializeOwned;
use thiserror::Error;

use cineharbor_addon_protocol::{
    CatalogResponse, ContentType, Manifest, MetaResponse, StreamsResponse, SubtitlesResponse,
};

use crate::transport::{HttpClient, HttpError, HttpRequest};

#[derive(Debug, Error)]
pub enum RemoteAddonError {
    #[error("base url 非法: {0}")]
    InvalidBaseUrl(String),
    #[error("HTTP 请求失败: {0}")]
    Http(#[from] HttpError),
    #[error("响应解析失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("addon 返回非成功状态: {0}")]
    Status(StatusCode),
}

/// 一个通过 HTTP 访问的 Stremio 兼容 addon（transport-agnostic）。
pub struct RemoteAddon<C> {
    base_url: String,
    client: C,
}

impl<C: HttpClient> RemoteAddon<C> {
    /// 规范化 base URL：去尾部斜杠、校验为 http/https。
    pub fn new(base_url: impl Into<String>, client: C) -> Result<Self, RemoteAddonError> {
        let trimmed = base_url.into().trim_end_matches('/').to_string();
        let parsed = url::Url::parse(&trimmed)
            .map_err(|error| RemoteAddonError::InvalidBaseUrl(error.to_string()))?;
        match parsed.scheme() {
            "http" | "https" => {}
            scheme => {
                return Err(RemoteAddonError::InvalidBaseUrl(format!(
                    "不支持的 scheme: {scheme}"
                )));
            }
        }
        Ok(Self {
            base_url: trimmed,
            client,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// 目录 URL；`extra` 为 (扩展名, 值)，`skip` 为分页偏移。
    pub fn catalog_url(
        &self,
        ty: ContentType,
        id: &str,
        extra: Option<(&str, &str)>,
        skip: Option<u32>,
    ) -> String {
        match (extra, skip) {
            (Some((name, value)), _) => {
                self.endpoint(&format!("/catalog/{ty}/{id}/{name}={value}.json"))
            }
            (None, Some(n)) => self.endpoint(&format!("/catalog/{ty}/{id}/skip={n}.json")),
            (None, None) => self.endpoint(&format!("/catalog/{ty}/{id}.json")),
        }
    }

    async fn get_json<T>(&self, url: &str) -> Result<T, RemoteAddonError>
    where
        T: DeserializeOwned,
    {
        let response = self.client.request(HttpRequest::get(url)).await?;
        if !response.status.is_success() {
            return Err(RemoteAddonError::Status(response.status));
        }
        serde_json::from_slice(&response.body).map_err(Into::into)
    }

    pub async fn manifest(&self) -> Result<Manifest, RemoteAddonError> {
        self.get_json(&self.endpoint("/manifest.json")).await
    }

    pub async fn catalog(
        &self,
        ty: ContentType,
        id: &str,
        extra: Option<(&str, &str)>,
        skip: Option<u32>,
    ) -> Result<CatalogResponse, RemoteAddonError> {
        let url = self.catalog_url(ty, id, extra, skip);
        self.get_json(&url).await
    }

    /// 返回 `Ok(None)` 表示 addon 侧 404（未收录）。
    pub async fn meta(&self, ty: ContentType, id: &str) -> Result<Option<MetaResponse>, RemoteAddonError> {
        let url = self.endpoint(&format!("/meta/{ty}/{id}.json"));
        let response = self.client.request(HttpRequest::get(url.as_str())).await?;
        if response.status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status.is_success() {
            return Err(RemoteAddonError::Status(response.status));
        }
        Ok(Some(serde_json::from_slice(&response.body)?))
    }

    pub async fn streams(
        &self,
        ty: ContentType,
        id: &str,
    ) -> Result<StreamsResponse, RemoteAddonError> {
        self.get_json(&self.endpoint(&format!("/stream/{ty}/{id}.json")))
            .await
    }

    pub async fn subtitles(
        &self,
        ty: ContentType,
        id: &str,
        extra: &str,
    ) -> Result<SubtitlesResponse, RemoteAddonError> {
        self.get_json(&self.endpoint(&format!("/subtitles/{ty}/{id}/{extra}.json")))
            .await
    }
}

/// 把多个 addon 的目录结果按声明顺序拼接 metadata。
pub fn merge_catalogs(responses: impl IntoIterator<Item = CatalogResponse>) -> CatalogResponse {
    let mut metas = Vec::new();
    for response in responses {
        metas.extend(response.metas);
    }
    CatalogResponse { metas }
}

/// 把多个 addon 的播放流结果按声明顺序拼接。
pub fn merge_streams(responses: impl IntoIterator<Item = StreamsResponse>) -> StreamsResponse {
    let mut streams = Vec::new();
    for response in responses {
        streams.extend(response.streams);
    }
    StreamsResponse { streams }
}

/// 取第一个命中的 meta（对齐首 addon 命中即返回的聚合语义）。
pub fn first_meta(
    responses: impl IntoIterator<Item = Option<MetaResponse>>,
) -> Option<MetaResponse> {
    responses.into_iter().flatten().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{HttpResponse, HttpRequest};
    use cineharbor_addon_protocol::{MetaDetail, MetaPreview, Stream};

    struct NoopClient;

    #[async_trait::async_trait(?Send)]
    impl HttpClient for NoopClient {
        async fn request(&self, _request: HttpRequest) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
    }

    #[test]
    fn builds_catalog_urls_and_validates_base() {
        let addon = RemoteAddon::new("https://addon.example.test/", NoopClient).expect("valid");
        assert_eq!(addon.base_url(), "https://addon.example.test");
        assert_eq!(
            addon.catalog_url(ContentType::Movie, "top", None, None),
            "https://addon.example.test/catalog/movie/top.json"
        );
        assert_eq!(
            addon.catalog_url(ContentType::Movie, "top", Some(("search", "matrix")), None),
            "https://addon.example.test/catalog/movie/top/search=matrix.json"
        );
        assert_eq!(
            addon.catalog_url(ContentType::Series, "top", None, Some(30)),
            "https://addon.example.test/catalog/series/top/skip=30.json"
        );
        assert!(RemoteAddon::new("ftp://x.test", NoopClient).is_err());
        assert!(RemoteAddon::new("not a url", NoopClient).is_err());
    }

    #[test]
    fn merges_and_picks_first_meta() {
        let merged = merge_catalogs(vec![
            CatalogResponse {
                metas: vec![MetaPreview::new("a", ContentType::Movie, "A")],
            },
            CatalogResponse {
                metas: vec![MetaPreview::new("b", ContentType::Movie, "B")],
            },
        ]);
        assert_eq!(merged.metas.len(), 2);

        let streams = merge_streams(vec![
            StreamsResponse {
                streams: vec![Stream {
                    name: Some("s1".into()),
                    ..Stream::default()
                }],
            },
            StreamsResponse {
                streams: vec![Stream {
                    name: Some("s2".into()),
                    ..Stream::default()
                }],
            },
        ]);
        assert_eq!(streams.streams.len(), 2);

        let meta = MetaResponse {
            meta: MetaDetail::new("x", ContentType::Movie, "X"),
        };
        assert_eq!(first_meta([None, Some(meta.clone())]), Some(meta));
    }
}

#[cfg(all(test, feature = "native-http"))]
mod native_tests {
    use super::*;
    use crate::transport::ReqwestHttpClient;
    use axum::Json;
    use axum::routing::get;
    use axum::Router;
    use cineharbor_addon_protocol::{MetaDetail, MetaPreview, Resource, Stream};

    fn manifest_value() -> Manifest {
        Manifest {
            id: "mock.catalog".into(),
            version: "1.0.0".into(),
            name: "Mock".into(),
            description: None,
            resources: vec![Resource::Catalog, Resource::Stream],
            types: vec![ContentType::Movie],
            catalogs: vec![],
            id_prefixes: None,
            icon: None,
            logo: None,
            background: None,
            behavior_hints: None,
        }
    }

    fn catalog_value() -> CatalogResponse {
        CatalogResponse {
            metas: vec![MetaPreview::new("tt1", ContentType::Movie, "T1")],
        }
    }

    fn meta_value() -> MetaResponse {
        MetaResponse {
            meta: MetaDetail::new("tt1", ContentType::Movie, "T1"),
        }
    }

    fn streams_value() -> StreamsResponse {
        StreamsResponse {
            streams: vec![Stream {
                name: Some("1080p".into()),
                ..Stream::default()
            }],
        }
    }

    #[tokio::test]
    async fn fetches_through_reqwest_client() {
        let app = Router::new()
            .route("/manifest.json", get(|| async { Json(manifest_value()) }))
            .route(
                "/catalog/movie/top.json",
                get(|| async { Json(catalog_value()) }),
            )
            .route("/meta/movie/tt1.json", get(|| async { Json(meta_value()) }))
            .route(
                "/meta/movie/nope.json",
                get(|| async { StatusCode::NOT_FOUND }),
            )
            .route(
                "/stream/movie/tt1.json",
                get(|| async { Json(streams_value()) }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let addon = RemoteAddon::new(format!("http://{address}"), ReqwestHttpClient::new()).unwrap();

        assert_eq!(addon.manifest().await.unwrap().id, "mock.catalog");
        assert_eq!(
            addon.catalog(ContentType::Movie, "top", None, None)
                .await
                .unwrap()
                .metas
                .len(),
            1
        );
        assert!(addon.meta(ContentType::Movie, "tt1").await.unwrap().is_some());
        assert!(addon.meta(ContentType::Movie, "nope").await.unwrap().is_none());
        assert_eq!(
            addon.streams(ContentType::Movie, "tt1")
                .await
                .unwrap()
                .streams
                .len(),
            1
        );

        server.abort();
    }
}