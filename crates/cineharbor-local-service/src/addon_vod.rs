//! 内置 point-of-entry（vod）addon：把 local-service 的聚合搜索/详情包装成
//! Stremio 兼容的 movie/series 目录 + 播放流。
//!
//! 条目 id 形如 `vod:{source}:{id}`，播放流走 local-service 的 vod 代理
//! `/media/vod/m3u8`（`build_vod_proxy_m3u8_url`）。

use async_trait::async_trait;

use cineharbor_addon_host::{
    Addon, Catalog, CatalogRequest, CatalogResponse, ContentType, Manifest, MetaDetail,
    MetaPreview, MetaResponse, Resource, Stream, StreamsResponse,
};

use super::{AppState, SearchResult, build_vod_proxy_m3u8_url};

const ID_PREFIX: &str = "vod:";

pub(crate) struct BuiltinVodAddon {
    state: AppState,
}

impl BuiltinVodAddon {
    pub(crate) fn new(state: AppState) -> Self {
        Self { state }
    }

    fn split_id<'a>(&self, id: &'a str) -> Option<(&'a str, &'a str)> {
        let rest = id.strip_prefix(ID_PREFIX)?;
        rest.split_once(':')
    }
}

fn content_type_for(result: &SearchResult) -> ContentType {
    let label = result.type_name.as_deref().unwrap_or_default();
    let class = result.class.as_deref().unwrap_or_default();
    let kind = format!("{label} {class}");

    if kind.contains("电影") {
        return ContentType::Movie;
    }
    if kind.contains("电视剧")
        || kind.contains("综艺")
        || kind.contains("动漫")
        || kind.contains('剧')
    {
        return ContentType::Series;
    }
    if result.episodes.len() > 1 {
        ContentType::Series
    } else {
        ContentType::Movie
    }
}

#[async_trait]
impl Addon for BuiltinVodAddon {
    async fn manifest(&self) -> Manifest {
        Manifest {
            id: "cineharbor.vod".into(),
            version: "0.1.0".into(),
            name: "CineHarbor Movies/Series".into(),
            description: Some("聚合点播源（vod）的 movie/series 目录与播放流 addon".into()),
            resources: vec![Resource::Catalog, Resource::Meta, Resource::Stream],
            types: vec![ContentType::Movie, ContentType::Series],
            catalogs: vec![
                Catalog {
                    r#type: ContentType::Movie,
                    id: "search".into(),
                    name: "搜索".into(),
                    extra: vec![],
                    extra_supported: vec!["search".into()],
                },
                Catalog {
                    r#type: ContentType::Series,
                    id: "search".into(),
                    name: "搜索".into(),
                    extra: vec![],
                    extra_supported: vec!["search".into()],
                },
            ],
            id_prefixes: Some(vec!["vod".into()]),
            icon: None,
            logo: None,
            background: None,
            behavior_hints: None,
        }
    }

    async fn catalog(&self, req: CatalogRequest) -> CatalogResponse {
        let Some((name, query)) = req.extra else {
            return CatalogResponse::default();
        };
        if name != "search" || query.trim().is_empty() {
            return CatalogResponse::default();
        }
        let Some(config) = self.state.load_config().ok() else {
            return CatalogResponse::default();
        };

        let results =
            super::content_search::search_all_sites(&self.state.client, &config, query.trim())
                .await;
        let metas = results
            .into_iter()
            .map(|result| {
                let ty = content_type_for(&result);
                MetaPreview {
                    poster: (!result.poster.is_empty()).then_some(result.poster.clone()),
                    year: (!result.year.is_empty()).then_some(result.year.clone()),
                    ..MetaPreview::new(
                        format!("{ID_PREFIX}{}:{}", result.source, result.id),
                        ty,
                        &result.title,
                    )
                }
            })
            .collect();
        CatalogResponse { metas }
    }

    async fn meta(&self, ty: ContentType, id: &str) -> Option<MetaResponse> {
        let (source, vid) = self.split_id(id)?;
        let config = self.state.load_config().ok()?;
        let api_site = config
            .api_sites
            .iter()
            .find(|site| site.key == source && !site.disabled)?
            .clone();
        let result =
            super::content_detail::fetch_content_detail(&self.state.client, &api_site, vid)
                .await
                .ok()?;

        Some(MetaResponse {
            meta: MetaDetail {
                poster: (!result.poster.is_empty()).then_some(result.poster),
                year: (!result.year.is_empty()).then_some(result.year),
                description: result.desc,
                ..MetaDetail::new(id, ty, &result.title)
            },
        })
    }

    async fn streams(&self, _ty: ContentType, id: &str) -> StreamsResponse {
        let Some((source, vid)) = self.split_id(id) else {
            return StreamsResponse::default();
        };
        let Some(config) = self.state.load_config().ok() else {
            return StreamsResponse::default();
        };
        let Some(api_site) = config
            .api_sites
            .iter()
            .find(|site| site.key == source && !site.disabled)
            .cloned()
        else {
            return StreamsResponse::default();
        };
        let Ok(result) =
            super::content_detail::fetch_content_detail(&self.state.client, &api_site, vid).await
        else {
            return StreamsResponse::default();
        };

        let base = self.state.public_base_url();
        let streams = result
            .episodes
            .iter()
            .enumerate()
            .map(|(index, url)| Stream {
                name: Some(
                    result
                        .episodes_titles
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| format!("第 {} 集", index + 1)),
                ),
                title: Some(result.source_name.clone()),
                url: Some(build_vod_proxy_m3u8_url(base, &result.source, url)),
                ..Stream::default()
            })
            .collect();
        StreamsResponse { streams }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(type_name: Option<&str>, class: Option<&str>, episodes: usize) -> SearchResult {
        SearchResult {
            id: "1".into(),
            title: "t".into(),
            poster: String::new(),
            episodes: vec![String::new(); episodes],
            episodes_titles: Vec::new(),
            source: "s".into(),
            source_name: "S".into(),
            class: class.map(String::from),
            year: "2024".into(),
            desc: None,
            type_name: type_name.map(String::from),
            douban_id: None,
        }
    }

    #[test]
    fn infers_content_type() {
        assert_eq!(
            content_type_for(&result(Some("电影"), None, 1)),
            ContentType::Movie
        );
        assert_eq!(
            content_type_for(&result(Some("电视剧"), None, 12)),
            ContentType::Series
        );
        assert_eq!(content_type_for(&result(None, None, 1)), ContentType::Movie);
        assert_eq!(
            content_type_for(&result(None, None, 3)),
            ContentType::Series
        );
    }
}
