use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use typed_uuid::Uuid;

pub type DownloadId = Uuid<Download>;
pub type VideoId = Uuid<Video>;
pub type UserId = Uuid<User>;
pub type UserSessionId = Uuid<UserSession>;
pub type OidcMappingId = Uuid<OidcMapping>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Download {
    pub download_id: DownloadId,
    pub video_id: VideoId,
    pub error: Option<String>,
    pub status: DownloadStatus,
    pub retry_count: i32,
    pub force: bool,
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Processing,
    Finished,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Video {
    pub video_id: VideoId,
    pub title: String,
    pub youtube_id: Option<String>,
    pub url: String,
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub user_id: UserId,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub last_login: OffsetDateTime,
    pub last_activity: OffsetDateTime,
    pub email: String,
    pub handle: String,
    pub profile_picture_url: String,
    pub is_approved: bool,
    pub is_admin: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserSession {
    pub user_session_id: UserSessionId,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    // This info is generated by the oidc client and is needed
    // to validate that the user has successfully logged in.
    // Delete after logging in
    pub oidc_issuer_url: Option<String>,
    pub csrf_token: Option<String>,
    pub nonce: Option<String>,
    pub pkce_code_verifier: Option<String>,
    //
    // // Only set after logged in
    pub user_id: Option<UserId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OidcMapping {
    pub oidc_mapping_id: OidcMappingId,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub oidc_issuer_url: String,
    pub oidc_issuer_id: String,
    pub user_id: UserId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthContext {
    // pub user: User,
    // pub user_session: UserSession,
}

#[derive(strum::Display, strum::EnumString, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
pub enum ApiError {
    NotFound,
    CsrfFailure,
    NotAuthorized,
    InternalServerError,
    Unknown(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DownloadRequest {
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthUrlResponse {
    pub url: String,
}
