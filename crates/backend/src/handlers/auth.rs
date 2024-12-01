use api::UserSessionId;
use axum::{
    extract::{Request, State},
    response::IntoResponse,
    Extension,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::{Deserialize, Serialize};

use crate::{
    error::{ListenErrorExt, Result},
    oidc::OidcClient,
    PgPool,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Claims {
    exp: usize,
    sub: UserSessionId,
}

const COOKIE_NAME: &str = "__Host-user_token";

fn decode_jwt(
    jwt_decoding_key: &jsonwebtoken::DecodingKey,
    cookie_jar: &CookieJar,
) -> Option<Claims> {
    let token = cookie_jar.get(COOKIE_NAME)?;
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    match jsonwebtoken::decode::<Claims>(&token.value(), &jwt_decoding_key, &validation) {
        Ok(claims) => Some(claims.claims),
        Err(err) => {
            tracing::error!("Unable to validate jwt: {err:?}");
            None
        }
    }
}

pub async fn auth_optional(
    State(jwt_decoding_key): State<jsonwebtoken::DecodingKey>,
    cookie_jar: CookieJar,
    mut request: Request,
) -> Result<Request> {
    if let Some(claims) = decode_jwt(&jwt_decoding_key, &cookie_jar) {
        tracing::debug!("Inserting extension claims {claims:?}");
        request.extensions_mut().insert(claims);
    }

    Ok(request)
}

pub async fn auth_required(
    State(jwt_decoding_key): State<jsonwebtoken::DecodingKey>,
    cookie_jar: CookieJar,
    mut request: Request,
) -> Result<Request> {
    if let Some(claims) = decode_jwt(&jwt_decoding_key, &cookie_jar) {
        tracing::debug!("Inserting extension claims {claims:?}");
        request.extensions_mut().insert(claims);
        Ok(request)
    } else {
        Err(api::ApiError::NotAuthorized.into())
    }
}

pub async fn set_auth(
    State(jwt_encoding_key): State<jsonwebtoken::EncodingKey>,
    cookie_jar: CookieJar,
) -> Result<impl IntoResponse> {
    let expiration = time::OffsetDateTime::now_utc().saturating_add(time::Duration::days(30));

    let token = match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &Claims {
            exp: usize::try_from(expiration.unix_timestamp()).unwrap(),
            sub: UserSessionId::new_random(),
        },
        &jwt_encoding_key,
    ) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Unable to encode jwt token: {e:?}");
            return Err(api::ApiError::InternalServerError.into());
        }
    };

    let cookie_jar = cookie_jar.add(
        Cookie::build((COOKIE_NAME, token))
            .http_only(true)
            .secure(true)
            .path("/")
            .same_site(axum_extra::extract::cookie::SameSite::Strict)
            .expires(expiration),
    );

    Ok((cookie_jar, axum::Json(())))
}

pub async fn get_auth(
    claims: Option<Extension<Claims>>,
) -> Result<axum::Json<Option<serde_json::Value>>> {
    Ok(axum::Json(claims.map(|Extension(claims)| {
        serde_json::value::to_value(&claims).unwrap()
    })))
}

pub async fn clear_auth(cookie_jar: CookieJar) -> Result<impl IntoResponse> {
    let cookie_jar = cookie_jar.remove(
        Cookie::build(COOKIE_NAME)
            .removal()
            .http_only(true)
            .secure(true)
            .path("/")
            .same_site(axum_extra::extract::cookie::SameSite::Strict),
    );

    Ok((cookie_jar, axum::Json(())))
}

pub async fn auth_url(
    State(jwt_encoding_key): State<jsonwebtoken::EncodingKey>,
    State(pool): State<PgPool>,
    State(oidc_client): State<OidcClient>,
    cookie_jar: CookieJar,
) -> Result<(CookieJar, axum::Json<api::AuthUrlResponse>)> {
    tracing::info!("I am here");
    let mut conn = pool.get().await.with_internal_server_error()?;

    let auth_url = oidc_client.auth_url().await?;

    let session = database::models::UserSession::create(
        &mut conn,
        &auth_url.issuer_url,
        &auth_url.csrf_token,
        &auth_url.nonce,
        &auth_url.pkce_code_verifier,
    )
    .await
    .with_internal_server_error()?;

    let expiration = time::OffsetDateTime::now_utc().saturating_add(time::Duration::days(30));

    let token = match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &Claims {
            exp: usize::try_from(expiration.unix_timestamp()).unwrap(),
            sub: session.user_session_id,
        },
        &jwt_encoding_key,
    ) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Unable to encode jwt token: {e:?}");
            return Err(api::ApiError::InternalServerError.into());
        }
    };

    let cookie_jar = cookie_jar.add(
        Cookie::build((COOKIE_NAME, token))
            .http_only(true)
            .secure(true)
            .path("/")
            .same_site(axum_extra::extract::cookie::SameSite::Strict)
            .expires(expiration),
    );

    Ok((
        cookie_jar,
        axum::Json(api::AuthUrlResponse {
            url: auth_url.auth_url.as_str().to_string(),
        }),
    ))
}

/*
pub async fn validate_auth(
    State(jwt_decoding_key): State<jsonwebtoken::DecodingKey>,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    mut request: Request,
) -> Result<Request, ListenError> {
    let mut conn = pool.get().await.with_internal_server_error()?;

    tracing::info!("Running auth validator");
    let Some(auth_context) = validate_session(&mut conn, &jwt_decoding_key, &headers).await else {
        return Err(ApiError::NotAuthorized.into());
    };

    tracing::info!("Inserting extension with valute {auth_context:?}");
    request.extensions_mut().insert(auth_context);

    Ok(request)
}

async fn validate_session(
    conn: &mut AsyncPgConnection,
    jwt_decoding_key: &jsonwebtoken::DecodingKey,
    headers: &HeaderMap,
) -> Option<AuthContext> {
    return Some(AuthContext {});
    let token = get_bearer_token(&headers)?;

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    let claims = match jsonwebtoken::decode::<Claims>(&token, &jwt_decoding_key, &validation) {
        Ok(claims) => claims,
        Err(err) => {
            tracing::error!("Unable to validate jwt: {err:?}");
            return None;
        }
    };

    match database::models::User::get_by_user_session_id(conn, claims.claims.sub).await {
        Ok(Some((user, user_session))) => Some(AuthContext {
            // user: user.into(),
            // user_session: user_session.into(),
        }),
        Ok(None) => {
            tracing::error!("Could not find user session with id {}", claims.claims.sub);
            None
        }
        Err(e) => {
            tracing::error!("Database error while trying to get user session {e:?}");
            None
        }
    }
}

fn get_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let authorization = headers.get(AUTHORIZATION)?.to_str().ok()?;

    let (left, right) = authorization.split_once(' ')?;

    if left != "Bearer" {
        return None;
    }

    Some(right)
}
 */
