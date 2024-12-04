use api::ApiError;
use leptos::prelude::{expect_context, provide_context};

use super::global_redirect::{use_global_redirect, GlobalRedirect};

#[derive(Clone)]
#[non_exhaustive]
pub struct Backend {
    global_redirect: GlobalRedirect,
}

const BASE_URL: &str = "/api";

type BackendResult<T> = Result<Result<T, ApiError>, gloo_net::Error>;

#[allow(dead_code)]
impl Backend {
    pub fn new() -> Self {
        Self {
            global_redirect: use_global_redirect(),
        }
    }

    async fn json_response<Res>(&self, response: gloo_net::http::Response) -> BackendResult<Res>
    where
        Res: serde::de::DeserializeOwned,
    {
        if response.ok() {
            let response: Res = response.json().await?;
            Ok(Ok(response))
        } else {
            let response: ApiError = response.json().await?;
            match response {
                ApiError::NotAuthorized => self.global_redirect.navigate("/auth/login".to_string()),
                ApiError::AuthorizationPending => {
                    self.global_redirect.navigate("/auth/pending".to_string())
                }
                ApiError::CsrfFailure
                | ApiError::NotFound
                | ApiError::InternalServerError
                | ApiError::Unknown(_) => (),
            }
            Ok(Err(response))
        }
    }

    async fn get<Res>(&self, path: &str) -> BackendResult<Res>
    where
        Res: serde::de::DeserializeOwned,
    {
        let response = gloo_net::http::Request::get(&format!("{BASE_URL}{path}"))
            .header("X-LISTEN-CSRF-PROTECTION", "1")
            .send()
            .await?;

        self.json_response(response).await
    }

    async fn post<Res>(&self, path: &str) -> BackendResult<Res>
    where
        Res: serde::de::DeserializeOwned,
    {
        let response = gloo_net::http::Request::post(&format!("{BASE_URL}{path}"))
            .header("X-LISTEN-CSRF-PROTECTION", "1")
            .send()
            .await?;

        self.json_response(response).await
    }

    async fn post_json<Body, Res>(&self, path: &str, body: &Body) -> BackendResult<Res>
    where
        Body: ?Sized + serde::Serialize,
        Res: serde::de::DeserializeOwned,
    {
        let response = gloo_net::http::Request::post(&format!("{BASE_URL}{path}"))
            .header("X-LISTEN-CSRF-PROTECTION", "1")
            .json(body)?
            .send()
            .await?;

        self.json_response(response).await
    }

    pub async fn list_videos(&self) -> BackendResult<Vec<api::Video>> {
        self.get("/videos").await
    }

    pub async fn get_video(&self, video: api::VideoId) -> BackendResult<api::Video> {
        self.get(&format!("/videos/{video}")).await
    }

    pub async fn list_downloads(&self) -> BackendResult<Vec<(api::Video, Vec<api::Download>)>> {
        self.get("/downloads").await
    }

    pub async fn add_download(&self, request: &api::DownloadRequest) -> BackendResult<()> {
        self.post_json("/downloads/add", request).await
    }

    pub async fn get_auth(&self) -> BackendResult<Option<serde_json::Value>> {
        self.get("/get-auth").await
    }

    pub async fn auth_url(&self) -> BackendResult<api::AuthUrlResponse> {
        self.post("/auth/auth-url").await
    }

    pub async fn auth_verify(&self, request: &api::AuthVerificationRequest) -> BackendResult<bool> {
        self.post_json("/auth/auth-verify", request).await
    }

    pub async fn logout(&self) -> BackendResult<()> {
        self.post("/auth/logout").await
    }

    pub async fn get_profile(&self) -> BackendResult<api::User> {
        self.get("/users/profile").await
    }

    pub async fn get_unauthorized(&self) -> BackendResult<()> {
        self.get("/auth/test-unauthorized").await
    }

    pub async fn get_authorization_pending(&self) -> BackendResult<()> {
        self.get("/auth/test-authorization-pending").await
    }
}

pub fn provide_backend() {
    provide_context(Backend::new());
}

pub fn use_backend() -> Backend {
    expect_context()
}
