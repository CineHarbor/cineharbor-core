//! 内置 douban（豆瓣）参考 addon：把 local-service 的豆瓣搜索包装成
//! Stremio 兼容的 movie/series 目录（仅 catalog/search；无独立详情与流源）。

use async_trait::async_trait;

use cineharbor_addon_host::{
    Addon, Catalog, CatalogRequest, CatalogResponse, ContentType, Manifest, MetaPreview,
    MetaResponse, Resource, StreamsResponse,
};

use super::{
    AppState, fetch_douban_search_page, is_douban_search_subject_item, map_douban_search_item,
};

const ID_PREFIX: &str = "douban:";

pub(crate) struct BuiltinDoubanAddon {
    state: AppState,
}

impl BuiltinDoubanAddon {
    pub(crate) fn new(state: AppState) -> Self {
        Self { state }
    }
}

fn content_type_for(play_type: Option<&'static str>) -> ContentType {
    match play_type {
        Some("tv") => ContentType::Series,
        _ => ContentType::Movie,
    }
}

#[async_trait]
impl Addon for BuiltinDoubanAddon {
    async fn manifest(&self) -> Manifest {
        Manifest {
            id: "cineharbor.douban".into(),
            version: "0.1.0".into(),
            name: "CineHarbor Douban".into(),
            description: Some("豆瓣检索参考 addon（catalog/search）".into()),
            resources: vec![Resource::Catalog],
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
            id_prefixes: Some(vec!["douban".into()]),
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
        let start = req.skip.map(|s| s as usize).unwrap_or(0);
        let Ok(page) = fetch_douban_search_page(
            &self.state.client,
            &config,
            &self.state.douban_search_api_base_url,
            query.trim(),
            start,
        )
        .await
        else {
            return CatalogResponse::default();
        };

        let metas = page
            .items
            .into_iter()
            .filter(is_douban_search_subject_item)
            .map(|item| {
                let mapped = map_douban_search_item(&item);
                let ty = content_type_for(mapped.play_type);
                MetaPreview {
                    poster: (!mapped.poster.is_empty()).then_some(mapped.poster),
                    year: (!mapped.year.is_empty()).then_some(mapped.year.clone()),
                    ..MetaPreview::new(format!("{ID_PREFIX}{}", mapped.id), ty, &mapped.title)
                }
            })
            .collect();
        CatalogResponse { metas }
    }

    async fn meta(&self, _ty: ContentType, _id: &str) -> Option<MetaResponse> {
        None
    }

    async fn streams(&self, _ty: ContentType, _id: &str) -> StreamsResponse {
        StreamsResponse::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_play_type_to_content_type() {
        assert_eq!(content_type_for(Some("tv")), ContentType::Series);
        assert_eq!(content_type_for(Some("movie")), ContentType::Movie);
        assert_eq!(content_type_for(None), ContentType::Movie);
    }
}
