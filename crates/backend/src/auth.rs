use api::{ApiError, UserSessionId};
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use serde::{Deserialize, Serialize};

use crate::error::ListenError;

#[derive(Debug, PartialEq, Eq, Clone)]
struct AuthBearer(pub String);

#[axum::async_trait]
impl<B> FromRequestParts<B> for AuthBearer
where
    B: Send + Sync,
{
    type Rejection = ListenError;

    async fn from_request_parts(req: &mut Parts, _: &B) -> Result<Self, Self::Rejection> {
        let authorization = req
            .headers
            .get(AUTHORIZATION)
            .ok_or(ApiError::NotAuthorized)?
            .to_str()
            .map_err(|_| ApiError::NotAuthorized)?;

        let (left, right) = authorization
            .split_once(' ')
            .ok_or(ApiError::NotAuthorized)?;

        if left != "Bearer" {
            return Err(ApiError::NotAuthorized.into());
        }

        Ok(Self(right.to_string()))
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Claims {
    exp: usize,
    sub: UserSessionId,
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
