//! 内置直播 addon：把 local-service 已配置的直播源暴露为 Stremio 兼容的
//! `type=tv` 目录/详情/播放流，供 `/addons` 主机聚合。
//!
//! 当前取第一个启用的直播源（`config.live_sources` 中首个 `!disabled`）；频道 id 形如
//! `live:{index}`，按该源的频道顺序稳定映射。

use async_trait::async_trait;

use cineharbor_addon_host::{
    Addon, Catalog, CatalogRequest, CatalogResponse, ContentType, Manifest, MetaDetail,
    MetaPreview, MetaResponse, Resource, Stream, StreamsResponse,
};

use super::{
    AppState, LiveChannel, LiveSourceConfig, ServiceConfig, build_live_proxy_m3u8_url,
    get_or_refresh_live_channels_cache,
};

const ID_PREFIX: &str = "live:";

pub(crate) struct BuiltinLiveAddon {
    state: AppState,
}

impl BuiltinLiveAddon {
    pub(crate) fn new(state: AppState) -> Self {
        Self { state }
    }

    fn enabled_source(&self, config: &ServiceConfig) -> Option<LiveSourceConfig> {
        config.live_sources.iter().find(|s| !s.disabled).cloned()
    }

    async fn resolve(&self, id: &str) -> Option<(LiveSourceConfig, LiveChannel)> {
        let idx: usize = id.strip_prefix(ID_PREFIX)?.parse().ok()?;
        let config = self.state.load_config().ok()?;
        let source = self.enabled_source(&config)?;
        let cache = get_or_refresh_live_channels_cache(&self.state, &source)
            .await
            .ok()?;
        let channel = cache.channels.get(idx)?.clone();
        Some((source, channel))
    }
}

#[async_trait]
impl Addon for BuiltinLiveAddon {
    async fn manifest(&self) -> Manifest {
        Manifest {
            id: "cineharbor.live".into(),
            version: "0.1.0".into(),
            name: "CineHarbor Live".into(),
            description: Some("本地直播源（IPTV）".into()),
            resources: vec![Resource::Catalog, Resource::Meta, Resource::Stream],
            types: vec![ContentType::Tv],
            catalogs: vec![Catalog {
                r#type: ContentType::Tv,
                id: "channels".into(),
                name: "频道".into(),
                extra: vec![],
                extra_supported: vec![],
            }],
            id_prefixes: Some(vec!["live".into()]),
            icon: None,
            logo: None,
            background: None,
            behavior_hints: None,
        }
    }

    async fn catalog(&self, _req: CatalogRequest) -> CatalogResponse {
        let Some(config) = self.state.load_config().ok() else {
            return CatalogResponse::default();
        };
        let Some(source) = self.enabled_source(&config) else {
            return CatalogResponse::default();
        };
        let Ok(cache) = get_or_refresh_live_channels_cache(&self.state, &source).await else {
            return CatalogResponse::default();
        };
        let metas = cache
            .channels
            .iter()
            .enumerate()
            .map(|(idx, ch)| MetaPreview {
                poster: (!ch.logo.is_empty()).then(|| ch.logo.clone()),
                description: (!ch.group.is_empty()).then(|| ch.group.clone()),
                ..MetaPreview::new(format!("{ID_PREFIX}{idx}"), ContentType::Tv, &ch.name)
            })
            .collect();
        CatalogResponse { metas }
    }

    async fn meta(&self, _ty: ContentType, id: &str) -> Option<MetaResponse> {
        let (_, channel) = self.resolve(id).await?;
        Some(MetaResponse {
            meta: MetaDetail {
                poster: (!channel.logo.is_empty()).then(|| channel.logo.clone()),
                description: (!channel.group.is_empty()).then(|| channel.group.clone()),
                ..MetaDetail::new(id, ContentType::Tv, &channel.name)
            },
        })
    }

    async fn streams(&self, _ty: ContentType, id: &str) -> StreamsResponse {
        let Some((source, channel)) = self.resolve(id).await else {
            return StreamsResponse::default();
        };
        let url = build_live_proxy_m3u8_url(
            self.state.public_base_url(),
            &source.key,
            &channel.url,
            true,
        );
        StreamsResponse {
            streams: vec![Stream {
                name: Some(channel.name.clone()),
                title: Some(source.name.clone()),
                url: Some(url),
                ..Stream::default()
            }],
        }
    }
}
