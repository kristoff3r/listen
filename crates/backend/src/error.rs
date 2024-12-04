use std::fmt::{self, Debug, Display};

use api::ApiError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing_error::SpanTrace;

pub type Result<T> = std::result::Result<T, ListenError>;

pub struct ListenError {
    pub api_error: ApiError,
    pub inner: anyhow::Error,
    pub context: SpanTrace,
}

impl Debug for ListenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListenError")
            .field("message", &self.api_error)
            .field("inner", &self.inner)
            .field("context", &self.context)
            .finish()
    }
}

impl Display for ListenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", &self.api_error)?;
        writeln!(f, "{:?}", self.inner)?;
        fmt::Display::fmt(&self.context, f)
    }
}

impl IntoResponse for ListenError {
    fn into_response(self) -> Response {
        let status = if let Some(diesel::result::Error::NotFound) =
            self.inner.downcast_ref::<diesel::result::Error>()
        {
            StatusCode::NOT_FOUND
        } else {
            match self.api_error {
                ApiError::NotFound => StatusCode::NOT_FOUND,
                ApiError::CsrfFailure => StatusCode::BAD_REQUEST,
                ApiError::NotAuthorized => StatusCode::UNAUTHORIZED,
                ApiError::AuthorizationPending => StatusCode::FORBIDDEN,
                ApiError::InternalServerError => {
                    tracing::error!("Internal server error: {:?}\n{}", self.inner, self.context);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                ApiError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        };

        let body = serde_json::to_vec(&self.api_error).unwrap();
        (status, body).into_response()
    }
}

impl From<ApiError> for ListenError {
    fn from(api_error: ApiError) -> Self {
        let inner = anyhow::anyhow!("{}", api_error);
        ListenError {
            api_error,
            inner,
            context: SpanTrace::capture(),
        }
    }
}

pub trait ListenErrorExt<T, E: Into<anyhow::Error>>: Sized {
    fn with_api_error(self, error_type: ApiError) -> Result<T>;
    fn with_internal_server_error(self) -> Result<T> {
        self.with_api_error(api::ApiError::InternalServerError)
    }
}

impl<T, E: Into<anyhow::Error>> ListenErrorExt<T, E> for std::result::Result<T, E> {
    fn with_api_error(self, error_type: ApiError) -> Result<T> {
        self.map_err(|error| ListenError {
            api_error: error_type,
            inner: error.into(),
            context: SpanTrace::capture(),
        })
    }
}
pub trait ListenErrorExt2<T>: Sized {
    fn with_api_error(self, error_type: ApiError) -> Result<T>;
    fn with_internal_server_error(self) -> Result<T> {
        self.with_api_error(api::ApiError::InternalServerError)
    }
}

impl<T> ListenErrorExt2<T> for Result<T> {
    fn with_api_error(self, error_type: ApiError) -> Result<T> {
        self.map_err(|mut e| {
            e.api_error = error_type;
            e
        })
    }
}
