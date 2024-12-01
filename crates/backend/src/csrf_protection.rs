use std::str::FromStr;

use api::ApiError;
use axum::extract::{Host, Request};
use hyper::{HeaderMap, Method, Uri};
use leptos::prelude::*;

use crate::error::ListenError;

pub async fn csrf_protection(
    headers: HeaderMap,
    host: Host,
    request: Request,
) -> Result<Request, ListenError> {
    validate_csrf_header(&headers)?;
    validate_origin_header(request.method(), &headers, &host)?;

    Ok(request)
}

fn validate_csrf_header(headers: &HeaderMap) -> Result<(), ListenError> {
    if headers
        .get("X-LISTEN-CSRF-PROTECTION")
        .is_some_and(|h| h == "1")
    {
        tracing::debug!("XSRF header present");
        Ok(())
    } else {
        tracing::warn!("API request without the `X-LISTEN-CSRF-PROTECTION` header");
        Err(ApiError::CsrfFailure.into())
    }
}

fn validate_origin_header(
    method: &Method,
    headers: &HeaderMap,
    host: &Host,
) -> Result<(), ListenError> {
    let Some(origin) = headers.get(hyper::http::header::ORIGIN) else {
        if method == Method::GET || method == Method::HEAD {
            // Get GET/HEAD requests don't send the origin header for same-origin requests
            tracing::debug!("Origin header missing for a GET/HEAD request");
            return Ok(());
        } else {
            tracing::warn!("API request without the `Origin` header");
            return Err(ApiError::CsrfFailure.into());
        }
    };

    let Ok(origin) = std::str::from_utf8(origin.as_bytes()) else {
        tracing::warn!("API request with a non-utf8 `Origin` header");
        return Err(ApiError::CsrfFailure.into());
    };

    let origin_uri = match Uri::from_str(origin) {
        Ok(origin_uri) => origin_uri,
        Err(e) => {
            tracing::warn!("API request with an invalid `Origin` header: {e:?}");
            return Err(ApiError::CsrfFailure.into());
        }
    };

    if !origin_uri.host().is_some_and(|h| h == host.0) {
        tracing::warn!("API request where the `Origin` did not match the host header");
        return Err(ApiError::CsrfFailure.into());
    }

    // TODO: Do we also want to validate the source port and protocol matched?
    /*
    if !origin_uri.scheme().is_some_and(|s| s == &Scheme::HTTPS) {
        tracing::warn!("API request where the `Origin` header was not a https address");
        return Err(ApiError::CsrfFailure.into());
    }

    if !origin_uri.port().is_some_and(|p| p == PORT) {
        tracing::warn!("API request where the `Origin` port did not match the expected port");
        return Err(ApiError::CsrfFailure.into());
    }
     */

    tracing::debug!("Origin header valid");

    Ok(())
}
