use api::ApiError;
use leptos::prelude::use_context;

#[derive(Clone)]
#[non_exhaustive]
pub struct Backend {}

const BASE_URL: &str = "/api";

type BackendResult<T> = Result<Result<T, ApiError>, gloo_net::Error>;

#[allow(dead_code)]
impl Backend {
    pub fn new() -> Self {
        Self {}
    }

    async fn get<Res>(path: &str) -> BackendResult<Res>
    where
        Res: serde::de::DeserializeOwned,
    {
        let response = gloo_net::http::Request::get(&format!("{BASE_URL}{path}"))
            .header("X-LISTEN-CSRF-PROTECTION", "1")
            .send()
            .await?;

        if response.ok() {
            let response: Res = response.json().await?;
            Ok(Ok(response))
        } else {
            let response: ApiError = response.json().await?;
            Ok(Err(response))
        }
    }

    async fn post<Res>(path: &str) -> BackendResult<Res>
    where
        Res: serde::de::DeserializeOwned,
    {
        let response = gloo_net::http::Request::post(&format!("{BASE_URL}{path}"))
            .header("X-LISTEN-CSRF-PROTECTION", "1")
            .send()
            .await?;

        if response.ok() {
            let response: Res = response.json().await?;
            Ok(Ok(response))
        } else {
            let response: ApiError = response.json().await?;
            Ok(Err(response))
        }
    }

    async fn post_json<Body, Res>(path: &str, body: &Body) -> BackendResult<Res>
    where
        Body: ?Sized + serde::Serialize,
        Res: serde::de::DeserializeOwned,
    {
        let response = gloo_net::http::Request::post(&format!("{BASE_URL}{path}"))
            .header("X-LISTEN-CSRF-PROTECTION", "1")
            .json(body)?
            .send()
            .await?;

        if response.ok() {
            let response: Res = response.json().await?;
            Ok(Ok(response))
        } else {
            let response: ApiError = response.json().await?;
            Ok(Err(response))
        }
    }

    pub async fn list_videos(&self) -> BackendResult<Vec<api::Video>> {
        Self::get("/videos").await
    }

    pub async fn get_video(&self, video: api::VideoId) -> BackendResult<api::Video> {
        Self::get(&format!("/videos/{video}")).await
    }

    pub async fn list_downloads(&self) -> BackendResult<Vec<(api::Video, Vec<api::Download>)>> {
        Self::get("/downloads").await
    }

    pub async fn add_download(&self, request: &api::DownloadRequest) -> BackendResult<()> {
        Self::post_json("/downloads/add", request).await
    }

    pub async fn get_auth(&self) -> BackendResult<Option<serde_json::Value>> {
        Self::get("/get-auth").await
    }

    pub async fn set_auth(&self) -> BackendResult<()> {
        Self::post("/set-auth").await
    }

    pub async fn clear_auth(&self) -> BackendResult<()> {
        Self::post("/clear-auth").await
    }
}

pub fn use_backend() -> Backend {
    use_context::<Backend>().expect("Expected Backend in context")
}
