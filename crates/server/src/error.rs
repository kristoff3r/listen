use std::fmt::{self, Debug, Display};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use strum::Display;
use tracing_error::SpanTrace;

pub type Result<T> = std::result::Result<T, ListenError>;

pub struct ListenError {
    pub error_type: ListenErrorType,
    pub inner: anyhow::Error,
    pub context: SpanTrace,
}

impl<T> From<T> for ListenError
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        let cause = t.into();
        ListenError {
            error_type: ListenErrorType::Unknown(format!("{}", &cause)),
            inner: cause,
            context: SpanTrace::capture(),
        }
    }
}

impl Debug for ListenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListenError")
            .field("message", &self.error_type)
            .field("inner", &self.inner)
            .field("context", &self.context)
            .finish()
    }
}

impl Display for ListenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", &self.error_type)?;
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
            match self.error_type {
                ListenErrorType::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        };

        let body = serde_json::to_vec(&self.error_type).unwrap();
        (status, body).into_response()
    }
}

#[derive(Display, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
pub enum ListenErrorType {
    NotFound,
    Unknown(String),
}

impl From<ListenErrorType> for ListenError {
    fn from(error_type: ListenErrorType) -> Self {
        let inner = anyhow::anyhow!("{}", error_type);
        ListenError {
            error_type,
            inner,
            context: SpanTrace::capture(),
        }
    }
}

pub trait ListenErrorExt<T, E: Into<anyhow::Error>> {
    fn with_error_type(self, error_type: ListenErrorType) -> Result<T>;
}

impl<T, E: Into<anyhow::Error>> ListenErrorExt<T, E> for std::result::Result<T, E> {
    fn with_error_type(self, error_type: ListenErrorType) -> Result<T> {
        self.map_err(|error| ListenError {
            error_type,
            inner: error.into(),
            context: SpanTrace::capture(),
        })
    }
}
pub trait ListenErrorExt2<T> {
    fn with_error_type(self, error_type: ListenErrorType) -> Result<T>;
}

impl<T> ListenErrorExt2<T> for Result<T> {
    fn with_error_type(self, error_type: ListenErrorType) -> Result<T> {
        self.map_err(|mut e| {
            e.error_type = error_type;
            e
        })
    }
}
