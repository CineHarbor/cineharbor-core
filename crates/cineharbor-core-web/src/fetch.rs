//! 用浏览器 / Node 全局 `fetch` 实现 core 的 [`HttpClient`]（P3 的「addon HTTP 直连」传输）。
//!
//! 只依赖 js-sys + wasm-bindgen-futures（与 wasm-bindgen 0.2.126 配套的 0.3.103 / 0.4.76），
//! 不引入 web-sys，因此 browser 与 node（wasm-bindgen-test-runner）都能跑。

use cineharbor_addon_protocol::ContentType;
use cineharbor_core::addons::RemoteAddon;
use cineharbor_core::transport::{HttpClient, HttpError, HttpRequest, HttpResponse};
use http::StatusCode;
use js_sys::{ArrayBuffer, Function, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// 无内部状态的 fetch 客户端（`Clone` 便于各处持有）。
#[derive(Clone, Default)]
pub struct FetchHttpClient;

#[async_trait::async_trait(?Send)]
impl HttpClient for FetchHttpClient {
    async fn request(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        fetch_request(request).await
    }
}

async fn fetch_request(request: HttpRequest) -> Result<HttpResponse, HttpError> {
    let global = js_sys::global();
    let fetch_fn = Reflect::get(&global, &JsValue::from_str("fetch"))
        .map_err(|error| HttpError::Connect(format!("读取全局 fetch 失败: {error:?}")))?
        .dyn_into::<Function>()
        .map_err(|_| HttpError::Connect("全局 fetch 不是函数".to_string()))?;

    let init = Object::new();
    Reflect::set(
        &init,
        &JsValue::from_str("method"),
        &JsValue::from_str(request.method.as_str()),
    )
    .map_err(|error| HttpError::Request(format!("设置 method 失败: {error:?}")))?;

    let headers = Object::new();
    for (name, value) in &request.headers {
        Reflect::set(&headers, &JsValue::from_str(name), &JsValue::from_str(value))
            .map_err(|error| HttpError::Request(format!("设置 header {name} 失败: {error:?}")))?;
    }
    Reflect::set(&init, &JsValue::from_str("headers"), &headers)
        .map_err(|error| HttpError::Request(format!("设置 headers 失败: {error:?}")))?;

    if !request.body.is_empty() {
        let body = Uint8Array::new_with_length(request.body.len() as u32);
        body.copy_from(&request.body);
        Reflect::set(&init, &JsValue::from_str("body"), &body)
            .map_err(|error| HttpError::Request(format!("设置 body 失败: {error:?}")))?;
    }

    let promise = fetch_fn
        .call2(&global, &JsValue::from_str(&request.url), &init)
        .map_err(|error| HttpError::Connect(format!("调用 fetch 失败: {error:?}")))?;
    let response = JsFuture::from(Promise::from(promise))
        .await
        .map_err(|error| HttpError::Connect(format!("fetch 被拒: {error:?}")))?;

    let status = Reflect::get(&response, &JsValue::from_str("status"))
        .ok()
        .and_then(|value| value.as_f64())
        .map(|value| value as u16)
        .ok_or_else(|| HttpError::Request("响应缺少 status".to_string()))?;
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);

    let array_buffer_fn = Reflect::get(&response, &JsValue::from_str("arrayBuffer"))
        .map_err(|error| HttpError::Request(format!("读取 arrayBuffer 失败: {error:?}")))?
        .dyn_into::<Function>()
        .map_err(|_| HttpError::Request("arrayBuffer 不是函数".to_string()))?;
    let body = JsFuture::from(Promise::from(
        array_buffer_fn
            .call0(&response)
            .map_err(|error| HttpError::Request(format!("调用 arrayBuffer 失败: {error:?}")))?,
    ))
    .await
    .map_err(|error| HttpError::Request(format!("arrayBuffer 被拒: {error:?}")))?
    .dyn_into::<ArrayBuffer>()
    .map_err(|_| HttpError::Request("正文不是 ArrayBuffer".to_string()))?;
    let body = Uint8Array::new(&body).to_vec();

    let mut response_headers = Vec::new();
    if let Ok(headers_obj) = Reflect::get(&response, &JsValue::from_str("headers")) {
        if let Ok(get_fn) = Reflect::get(&headers_obj, &JsValue::from_str("get")) {
            if let Ok(get_fn) = get_fn.dyn_into::<Function>() {
                if let Ok(value) = get_fn.call1(&headers_obj, &JsValue::from_str("content-type")) {
                    if let Some(value) = value.as_string() {
                        response_headers.push(("content-type".to_string(), value));
                    }
                }
            }
        }
    }

    Ok(HttpResponse {
        status,
        headers: response_headers,
        body,
    })
}

fn parse_content_type(ty: &str) -> Option<ContentType> {
    match ty {
        "movie" => Some(ContentType::Movie),
        "series" => Some(ContentType::Series),
        "channel" => Some(ContentType::Channel),
        "tv" => Some(ContentType::Tv),
        _ => None,
    }
}

fn string_err(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// 拉取 addon manifest（`base_url` 形如 `http://127.0.0.1:11473`），返回 manifest JSON；
/// 失败返回 JS 异常（字符串）。
#[wasm_bindgen]
pub async fn addon_manifest_json(base_url: String) -> Result<String, JsValue> {
    let addon =
        RemoteAddon::new(base_url, FetchHttpClient).map_err(string_err)?;
    let manifest = addon.manifest().await.map_err(string_err)?;
    serde_json::to_string(&manifest).map_err(string_err)
}

/// 拉取 addon catalog（可带搜索词 / 翻页），返回 `CatalogResponse` JSON。
#[wasm_bindgen]
pub async fn addon_catalog_json(
    base_url: String,
    ty: String,
    id: String,
    extra_name: Option<String>,
    extra_value: Option<String>,
    skip: Option<u32>,
) -> Result<String, JsValue> {
    let addon = RemoteAddon::new(base_url, FetchHttpClient).map_err(string_err)?;
    let ty = parse_content_type(&ty).ok_or_else(|| JsValue::from_str("未知 content type"))?;
    let extra = match (extra_name, extra_value) {
        (Some(name), Some(value)) => Some((name, value)),
        _ => None,
    };
    let response = addon
        .catalog(
            ty,
            &id,
            extra.as_ref().map(|(name, value)| (name.as_str(), value.as_str())),
            skip,
        )
        .await
        .map_err(string_err)?;
    serde_json::to_string(&response).map_err(string_err)
}

/// 拉取 addon meta（未收录返回 `null`），返回 `MetaResponse` JSON（或 `null`）。
#[wasm_bindgen]
pub async fn addon_meta_json(base_url: String, ty: String, id: String) -> Result<String, JsValue> {
    let addon = RemoteAddon::new(base_url, FetchHttpClient).map_err(string_err)?;
    let ty = parse_content_type(&ty).ok_or_else(|| JsValue::from_str("未知 content type"))?;
    let meta = addon.meta(ty, &id).await.map_err(string_err)?;
    serde_json::to_string(&meta).map_err(string_err)
}

/// 拉取 addon streams（播单），返回 `StreamsResponse` JSON。
#[wasm_bindgen]
pub async fn addon_streams_json(
    base_url: String,
    ty: String,
    id: String,
) -> Result<String, JsValue> {
    let addon = RemoteAddon::new(base_url, FetchHttpClient).map_err(string_err)?;
    let ty = parse_content_type(&ty).ok_or_else(|| JsValue::from_str("未知 content type"))?;
    let streams = addon.streams(ty, &id).await.map_err(string_err)?;
    serde_json::to_string(&streams).map_err(string_err)
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn fetches_data_url_via_global_fetch() {
        use base64::Engine as _;
        let payload = r#"{"id":"mock.addon","status":"ok"}"#;
        let encoded = base64::engine::general_purpose::STANDARD.encode(payload.as_bytes());
        let url = format!("data:application/json;base64,{encoded}");

        let client = FetchHttpClient;
        let response = client.request(HttpRequest::get(url)).await.unwrap();

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(String::from_utf8(response.body).unwrap(), payload);
        assert_eq!(
            response
                .headers
                .iter()
                .find(|(name, _)| name == "content-type")
                .map(|(_, value)| value.as_str()),
            Some("application/json")
        );
    }
}