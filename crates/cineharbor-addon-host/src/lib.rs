//! Addon host：装载并聚合多个 Stremio 兼容 addon。
//!
//! P3 起由 `cineharbor-local-service` 持有：把内置/外部 addon 统一暴露为一个本地
//! addon 集合，供各客户端消费。
//!
//! 暴露两组路由：
//! - `per_addon_router()`：每个 addon 挂到 `/addon/{i}/…`（各自完整的 Stremio 端点）
//! - `aggregate_router()`：`/manifest.json`、`/catalog/…`、`/meta/…`、`/stream/…`，
//!   目录/播放流取全部 addon 并集，meta 取首个命中。

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use cineharbor_addon_sdk::addon::router as addon_router;
use cineharbor_addon_sdk::{
    Addon, CatalogRequest, CatalogResponse, ContentType, Manifest, MetaResponse, StreamsResponse,
};

type Addons = Vec<Arc<dyn Addon>>;

/// 一组已装载的 addon。
pub struct AddonHost {
    addons: Addons,
}

#[derive(Serialize)]
struct AddonEntry {
    transport_url: String,
    manifest: Manifest,
}

#[derive(Serialize)]
struct AddonCollection {
    addons: Vec<AddonEntry>,
}

impl Default for AddonHost {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl AddonHost {
    pub fn new(addons: Vec<Arc<dyn Addon>>) -> Self {
        Self { addons }
    }

    pub fn with(addon: Arc<dyn Addon>) -> Self {
        Self {
            addons: vec![addon],
        }
    }

    pub fn add(mut self, addon: Arc<dyn Addon>) -> Self {
        self.addons.push(addon);
        self
    }

    pub fn len(&self) -> usize {
        self.addons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.addons.is_empty()
    }

    /// 每个 addon 的独立 Stremio 端点，挂到 `/addon/{index}` 前缀。
    pub fn per_addon_router(&self) -> Router {
        let mut router = Router::new();
        for (i, addon) in self.addons.iter().enumerate() {
            router = router.nest(&format!("/addon/{i}"), addon_router(addon.clone()));
        }
        router
    }

    /// 聚合端点。
    pub fn aggregate_router(&self) -> Router {
        Router::new()
            .route("/manifest.json", get(manifest_handler))
            .route("/catalog/{ty}/{id}", get(catalog_handler))
            .route("/catalog/{ty}/{id}/{seg}", get(catalog_extra_handler))
            .route("/meta/{ty}/{id}", get(meta_handler))
            .route("/stream/{ty}/{id}", get(stream_handler))
            .with_state(self.addons.clone())
    }

    /// 合并路由。
    pub fn router(&self) -> Router {
        self.per_addon_router().merge(self.aggregate_router())
    }
}

fn strip_json(s: &str) -> &str {
    s.strip_suffix(".json").unwrap_or(s)
}

fn parse_ty(s: &str) -> Option<ContentType> {
    s.parse().ok()
}

async fn manifest_handler(State(addons): State<Addons>) -> Json<AddonCollection> {
    let mut entries = Vec::with_capacity(addons.len());
    for (i, addon) in addons.iter().enumerate() {
        entries.push(AddonEntry {
            transport_url: format!("/addon/{i}/manifest.json"),
            manifest: addon.manifest().await,
        });
    }
    Json(AddonCollection { addons: entries })
}

async fn catalog_handler(
    State(addons): State<Addons>,
    Path((ty, id)): Path<(String, String)>,
) -> Result<Json<CatalogResponse>, StatusCode> {
    let ty = parse_ty(&ty).ok_or(StatusCode::BAD_REQUEST)?;
    let mut metas = Vec::new();
    for addon in &addons {
        let req = CatalogRequest {
            ty,
            id: strip_json(&id).to_string(),
            extra: None,
            skip: None,
        };
        metas.extend(addon.catalog(req).await.metas);
    }
    Ok(Json(CatalogResponse { metas }))
}

async fn catalog_extra_handler(
    State(addons): State<Addons>,
    Path((ty, id, seg)): Path<(String, String, String)>,
) -> Result<Json<CatalogResponse>, StatusCode> {
    let ty = parse_ty(&ty).ok_or(StatusCode::BAD_REQUEST)?;
    let seg = strip_json(&seg);
    let (extra, skip) = if let Some(n) = seg.strip_prefix("skip=") {
        (None, n.parse::<u32>().ok())
    } else {
        let (name, value) = seg.split_once('=').ok_or(StatusCode::BAD_REQUEST)?;
        (Some((name.to_string(), value.to_string())), None)
    };
    let mut metas = Vec::new();
    for addon in &addons {
        let req = CatalogRequest {
            ty,
            id: id.clone(),
            extra: extra.clone(),
            skip,
        };
        metas.extend(addon.catalog(req).await.metas);
    }
    Ok(Json(CatalogResponse { metas }))
}

async fn meta_handler(
    State(addons): State<Addons>,
    Path((ty, id)): Path<(String, String)>,
) -> Result<Json<MetaResponse>, StatusCode> {
    let ty = parse_ty(&ty).ok_or(StatusCode::BAD_REQUEST)?;
    let id = strip_json(&id);
    for addon in &addons {
        if let Some(meta) = addon.meta(ty, id).await {
            return Ok(Json(meta));
        }
    }
    Err(StatusCode::NOT_FOUND)
}

async fn stream_handler(
    State(addons): State<Addons>,
    Path((ty, id)): Path<(String, String)>,
) -> Result<Json<StreamsResponse>, StatusCode> {
    let ty = parse_ty(&ty).ok_or(StatusCode::BAD_REQUEST)?;
    let id = strip_json(&id);
    let mut streams = Vec::new();
    for addon in &addons {
        streams.extend(addon.streams(ty, id).await.streams);
    }
    Ok(Json(StreamsResponse { streams }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use cineharbor_addon_sdk::{MetaPreview, Stream};

    struct Fake {
        id: String,
        metas: usize,
        stream_name: String,
    }

    fn minimal_manifest(id: &str) -> Manifest {
        Manifest {
            id: id.into(),
            version: "1.0.0".into(),
            name: id.into(),
            description: None,
            resources: vec![
                cineharbor_addon_sdk::Resource::Catalog,
                cineharbor_addon_sdk::Resource::Stream,
            ],
            types: vec![ContentType::Movie],
            catalogs: vec![],
            id_prefixes: None,
            icon: None,
            logo: None,
            background: None,
            behavior_hints: None,
        }
    }

    #[async_trait]
    impl Addon for Fake {
        async fn manifest(&self) -> Manifest {
            minimal_manifest(&self.id)
        }

        async fn catalog(&self, _req: CatalogRequest) -> CatalogResponse {
            let metas = (0..self.metas)
                .map(|i| {
                    MetaPreview::new(
                        format!("{}:tt{}", self.id, i),
                        ContentType::Movie,
                        format!("t{}", i),
                    )
                })
                .collect();
            CatalogResponse { metas }
        }

        async fn meta(&self, _ty: ContentType, _id: &str) -> Option<MetaResponse> {
            None
        }

        async fn streams(&self, _ty: ContentType, _id: &str) -> StreamsResponse {
            StreamsResponse {
                streams: vec![Stream {
                    name: Some(self.stream_name.clone()),
                    ..Stream::default()
                }],
            }
        }
    }

    async fn spawn(app: Router) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });
        format!("http://{addr}")
    }

    fn sample_host() -> AddonHost {
        AddonHost::new(vec![
            Arc::new(Fake {
                id: "a".into(),
                metas: 2,
                stream_name: "A".into(),
            }),
            Arc::new(Fake {
                id: "b".into(),
                metas: 3,
                stream_name: "B".into(),
            }),
        ])
    }

    #[tokio::test]
    async fn aggregates_catalog_and_streams() {
        let base = spawn(sample_host().router()).await;
        let client = cineharbor_addon_sdk::AddonClient::new(base).unwrap();
        let cat = client
            .catalog(ContentType::Movie, "top", None, None)
            .await
            .unwrap();
        assert_eq!(cat.metas.len(), 5);
        let streams = client.streams(ContentType::Movie, "tt1").await.unwrap();
        assert_eq!(streams.streams.len(), 2);
    }

    #[tokio::test]
    async fn mounts_each_addon() {
        let base = spawn(sample_host().per_addon_router()).await;
        let body = reqwest::get(format!("{base}/addon/1/manifest.json"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let manifest: Manifest = serde_json::from_str(&body).unwrap();
        assert_eq!(manifest.id, "b");
    }

    #[tokio::test]
    async fn empty_host_serves_empty_collection() {
        let base = spawn(AddonHost::default().router()).await;
        let body = reqwest::get(format!("{base}/manifest.json"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["addons"].as_array().unwrap().len(), 0);
    }
}
