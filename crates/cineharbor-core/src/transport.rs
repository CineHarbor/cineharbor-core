//! 平台无关的异步 HTTP 传输抽象。
//!
//! 供 core 的 addon 派发 / 同步等纯逻辑使用：native 实现走 reqwest（feature `native-http`，
//! 默认关闭，保证 core 本体 wasm 干净）；wasm 实现将在阶段 3 走 `fetch`。

use http::{Method, StatusCode};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: Method::GET,
            url: url.into(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HttpError {
    #[error("连接失败: {0}")]
    Connect(String),
    #[error("请求失败: {0}")]
    Request(String),
}

// `?Send`：wasm 侧 `fetch`（JsFuture）的 future 非 `Send`，native reqwest 仍可返回 `Send`
// future（比 `?Send` 更严格，协变通过）。故 trait 层放宽为 `?Send` 以支持双目标。
#[async_trait::async_trait(?Send)]
pub trait HttpClient: Send + Sync {
    async fn request(&self, request: HttpRequest) -> Result<HttpResponse, HttpError>;
}

#[cfg(feature = "native-http")]
pub use reqwest_http::ReqwestHttpClient;

#[cfg(feature = "native-http")]
mod reqwest_http {
    use super::{HttpClient, HttpError, HttpRequest, HttpResponse};

    #[derive(Clone, Default)]
    pub struct ReqwestHttpClient {
        client: reqwest::Client,
    }

    impl ReqwestHttpClient {
        pub fn new() -> Self {
            Self {
                client: reqwest::Client::new(),
            }
        }
    }

    #[async_trait::async_trait(?Send)]
    impl HttpClient for ReqwestHttpClient {
        async fn request(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
            let mut builder = self.client.request(request.method, &request.url);
            for (name, value) in request.headers {
                builder = builder.header(name, value);
            }
            let response = builder
                .body(request.body)
                .send()
                .await
                .map_err(|error| HttpError::Connect(error.to_string()))?;

            let status = response.status();
            let headers = response
                .headers()
                .iter()
                .map(|(name, value)| {
                    (
                        name.as_str().to_string(),
                        value.to_str().unwrap_or_default().to_string(),
                    )
                })
                .collect();
            let body = response
                .bytes()
                .await
                .map_err(|error| HttpError::Request(error.to_string()))?
                .to_vec();

            Ok(HttpResponse {
                status,
                headers,
                body,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::ReqwestHttpClient;
        use crate::transport::{HttpClient, HttpRequest};
        use axum::{Router, routing::get};

        #[tokio::test]
        async fn round_trips_against_axum_mock() {
            let app = Router::new().route("/ping", get(|| async { "hello" }));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });

            let client = ReqwestHttpClient::new();
            let response = client
                .request(HttpRequest::get(format!("http://{address}/ping")))
                .await
                .unwrap();

            assert_eq!(response.status, http::StatusCode::OK);
            assert_eq!(String::from_utf8(response.body).unwrap(), "hello");

            server.abort();
        }
    }
}