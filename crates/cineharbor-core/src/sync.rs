//! 账号同步（profile-sync）的纯领域模型与无 I/O 逻辑。
//!
//! 自 `cineharbor-sync` 抽出：网络 I/O（reqwest client 与 cookie 转发）留在
//! `cineharbor-sync`，此处只保留可双编译（native + wasm）的类型与纯函数。

use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub const PROFILE_SYNC_DEFAULT_USER_DATA_DOMAINS: [&str; 5] = [
    "playrecords",
    "favorites",
    "follows",
    "searchhistory",
    "skipconfigs",
];

pub const PROFILE_SYNC_ADMIN_SETTINGS_DOMAIN: &str = "adminsettings";

pub const PROFILE_SYNC_USER_DATA_DOMAINS: [&str; 6] = [
    "playrecords",
    "favorites",
    "follows",
    "searchhistory",
    "skipconfigs",
    PROFILE_SYNC_ADMIN_SETTINGS_DOMAIN,
];

pub fn default_profile_sync_selected_domains() -> Vec<String> {
    PROFILE_SYNC_DEFAULT_USER_DATA_DOMAINS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileSyncErrorKind {
    NotConfigured,
    InvalidBaseUrl,
    Unreachable,
    Unauthorized,
    ProtocolIncompatible,
    UpstreamFailure,
}

impl ProfileSyncErrorKind {
    pub fn http_status(self) -> StatusCode {
        match self {
            Self::NotConfigured => StatusCode::NOT_IMPLEMENTED,
            Self::InvalidBaseUrl => StatusCode::BAD_REQUEST,
            Self::Unreachable
            | Self::Unauthorized
            | Self::ProtocolIncompatible
            | Self::UpstreamFailure => StatusCode::BAD_GATEWAY,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ProfileSyncError {
    pub kind: ProfileSyncErrorKind,
    pub message: String,
}

impl ProfileSyncError {
    pub fn new(kind: ProfileSyncErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn http_status(&self) -> StatusCode {
        self.kind.http_status()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSyncSession {
    pub username: String,
    pub role: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSyncStatusResponse {
    pub enabled: bool,
    pub reachable: bool,
    pub authenticated: bool,
    pub username: Option<String>,
    pub role: Option<String>,
    pub storage_type: Option<String>,
    pub profile_mode: Option<String>,
    pub error: Option<String>,
    pub error_kind: Option<ProfileSyncErrorKind>,
    pub sync_domains: Vec<String>,
    pub pending_outbox_count: u64,
    pub reauth_required: bool,
    pub last_outbox_error: Option<String>,
    pub next_outbox_attempt_at: Option<i64>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct RemoteServerConfigResponse {
    pub storage_type: Option<String>,
    pub profile_mode: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct RemoteLoginResponse {
    pub ok: Option<bool>,
    pub username: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileSyncSessionMutation {
    Keep,
    Clear,
    Set(ProfileSyncSession),
}

pub fn build_profile_sync_target_url(
    remote_base_url: &str,
    request_path: &str,
) -> Result<Url, ProfileSyncError> {
    let base_url = format!("{}/", remote_base_url.trim_end_matches('/'));
    let base = Url::parse(&base_url).map_err(|error| {
        ProfileSyncError::new(
            ProfileSyncErrorKind::InvalidBaseUrl,
            format!("无效的账号同步地址: {error}"),
        )
    })?;
    base.join(request_path.trim_start_matches('/'))
        .map_err(|error| {
            ProfileSyncError::new(
                ProfileSyncErrorKind::InvalidBaseUrl,
                format!("无法解析账号同步目标地址: {error}"),
            )
        })
}

pub fn session_from_login_response(status: StatusCode, body: &[u8]) -> Option<ProfileSyncSession> {
    if !status.is_success() {
        return None;
    }

    let login_response = serde_json::from_slice::<RemoteLoginResponse>(body).ok()?;
    if !login_response.ok.unwrap_or(true) {
        return None;
    }

    let username = normalize_optional_string(login_response.username)?;
    let role =
        normalize_optional_string(login_response.role).unwrap_or_else(|| "user".to_string());

    Some(ProfileSyncSession { username, role })
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}