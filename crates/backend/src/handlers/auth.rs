use api::UserSessionId;
use axum::{
    extract::{Request, State},
    response::IntoResponse,
    Extension,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use oauth2::{AuthorizationCode, PkceCodeVerifier};
use openidconnect::{IssuerUrl, Nonce};
use serde::{Deserialize, Serialize};

use crate::{
    error::{ListenErrorExt, Result},
    oidc::OidcClient,
    PgPool,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Claims {
    exp: usize,
    sub: UserSessionId,
}

const USER_COOKIE_NAME: &str = "__Host-user_token";

#[derive(Clone, Debug)]
pub enum SessionState {
    None,
    Unauthenticated {
        user_session: database::models::UserSession,
    },
    Authenticated {
        user_session: database::models::UserSession,
        user: database::models::User,
    },
}

impl SessionState {
    async fn lookup(
        pool: &PgPool,
        jwt_decoding_key: &jsonwebtoken::DecodingKey,
        cookie_jar: &CookieJar,
    ) -> Result<Self> {
        let mut conn = pool.get().await.with_internal_server_error()?;
        let Some(token) = cookie_jar.get(USER_COOKIE_NAME) else {
            return Ok(Self::None);
        };
        let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        let claims =
            match jsonwebtoken::decode::<Claims>(&token.value(), &jwt_decoding_key, &validation) {
                Ok(claims) => claims.claims,
                Err(err) => {
                    tracing::error!("Unable to validate jwt: {err:?}");
                    return Ok(Self::None);
                }
            };

        let Some(user_session) = database::models::UserSession::get_by_id(&mut conn, claims.sub)
            .await
            .with_internal_server_error()?
        else {
            return Ok(Self::None);
        };

        let Some(user_id) = user_session.user_id else {
            return Ok(Self::Unauthenticated { user_session });
        };

        if let Some(user) = database::models::User::get_by_id(&mut conn, user_id)
            .await
            .with_internal_server_error()?
        {
            Ok(Self::Authenticated { user_session, user })
        } else {
            Ok(Self::Unauthenticated { user_session })
        }
    }
}

pub async fn user_session_layer(
    State(pool): State<PgPool>,
    State(jwt_decoding_key): State<jsonwebtoken::DecodingKey>,
    cookie_jar: CookieJar,
    mut request: Request,
) -> Result<Request> {
    let session_state = SessionState::lookup(&pool, &jwt_decoding_key, &cookie_jar).await?;

    request.extensions_mut().insert(session_state);

    Ok(request)
}

pub async fn auth_required_layer(
    Extension(session_state): Extension<SessionState>,
    mut request: Request,
) -> Result<Request> {
    if let SessionState::Authenticated { user, .. } = session_state {
        if user.is_approved {
            request.extensions_mut().insert(user);
            Ok(request)
        } else {
            tracing::debug!("Request with pending user");
            Err(api::ApiError::AuthorizationPending.into())
        }
    } else {
        tracing::debug!("Request required authentication");
        Err(api::ApiError::NotAuthorized.into())
    }
}

pub async fn auth_logout(cookie_jar: CookieJar) -> Result<impl IntoResponse> {
    let cookie_jar = cookie_jar.remove(
        Cookie::build(USER_COOKIE_NAME)
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
        Cookie::build((USER_COOKIE_NAME, token))
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

pub async fn auth_verify(
    Extension(session_state): Extension<SessionState>,
    State(pool): State<PgPool>,
    State(oidc_client): State<OidcClient>,
    axum::Json(request): axum::Json<api::AuthVerificationRequest>,
) -> Result<axum::Json<bool>> {
    let mut conn = pool.get().await.with_internal_server_error()?;
    let user_session = match session_state {
        SessionState::None => return Err(api::ApiError::NotAuthorized.into()),
        SessionState::Authenticated { .. } => return Ok(axum::Json(true)),
        SessionState::Unauthenticated { user_session } => user_session,
    };

    macro_rules! get {
        ($v:ident) => {
            let Some($v) = user_session.$v else {
                return Err(api::ApiError::NotAuthorized.into());
            };
        };
    }
    get!(oidc_issuer_url);
    get!(csrf_token);
    get!(nonce);
    get!(pkce_code_verifier);

    if request.state != csrf_token {
        tracing::error!(
            "Got bad CSRF token from oidc callback! {} != {}",
            request.state,
            csrf_token
        );
        return Err(api::ApiError::NotAuthorized.into());
    }

    let oidc_issuer_url = IssuerUrl::new(oidc_issuer_url.clone()).map_err(|e| {
        tracing::error!("Bad issuer url {oidc_issuer_url}: {e:?}",);
        api::ApiError::InternalServerError
    })?;
    let nonce = Nonce::new(nonce);
    let pkce_code_verifier = PkceCodeVerifier::new(pkce_code_verifier);

    let claims = oidc_client
        .auth_verify(
            AuthorizationCode::new(request.code),
            pkce_code_verifier,
            nonce,
            oidc_issuer_url,
        )
        .await?;

    let (user, _oidc_mapping) = match database::models::User::get_by_oidc(
        &mut conn,
        &claims.oidc_issuer_url,
        &claims.oidc_id,
    )
    .await
    .with_internal_server_error()?
    {
        Some(data) => data,
        None => database::models::User::create(
            &mut conn,
            &format!("Anonymous{:04}", rand::random::<u64>() % 1000),
            &claims.email,
            claims.picture_url.as_ref().map(|s| s.as_str()),
            &claims.oidc_issuer_url,
            &claims.oidc_id,
        )
        .await
        .with_internal_server_error()?,
    };

    database::models::UserSession::update_after_completed_login(
        &mut conn,
        user_session.user_session_id,
        user.user_id,
    )
    .await
    .with_internal_server_error()?;

    Ok(axum::Json(true))
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
