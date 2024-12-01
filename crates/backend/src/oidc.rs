use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope,
};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    EndUserEmail, EndUserPictureUrl, EndUserUsername, IssuerUrl, Nonce, SubjectIdentifier,
    TokenResponse,
};
use url::Url;

use crate::error::Result;

#[derive(Clone)]
pub struct OidcClient {
    issuer_url: IssuerUrl,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_url: RedirectUrl,
}

#[derive(Debug)]
pub struct AuthUrl {
    pub issuer_url: IssuerUrl,
    pub auth_url: Url,
    pub csrf_token: CsrfToken,
    pub nonce: Nonce,
    pub pkce_code_verifier: PkceCodeVerifier,
}

#[derive(Clone, Debug)]
pub struct OidcClaims {
    pub oidc_id: SubjectIdentifier,
    pub oidc_issuer_url: IssuerUrl,
    pub preferred_username: Option<EndUserUsername>,
    pub email: EndUserEmail,
    pub picture_url: Option<EndUserPictureUrl>,
}

impl OidcClient {
    pub fn new(
        issuer_url: IssuerUrl,
        client_id: ClientId,
        client_secret: ClientSecret,
        redirect_url: RedirectUrl,
    ) -> Self {
        Self {
            issuer_url,
            client_id,
            client_secret,
            redirect_url,
        }
    }

    // TODO: Do caching?
    async fn get_provider_metadata(&self) -> Result<CoreProviderMetadata> {
        CoreProviderMetadata::discover_async(
            self.issuer_url.clone(),
            oauth2::reqwest::async_http_client,
        )
        .await
        .map_err(|e| {
            tracing::error!(
                "Error doing metadata discovery for {}: {e:?}",
                self.issuer_url.as_str()
            );
            api::ApiError::InternalServerError.into()
        })
    }

    async fn get_client(&self) -> Result<CoreClient> {
        let client = CoreClient::from_provider_metadata(
            self.get_provider_metadata().await?,
            self.client_id.clone(),
            Some(self.client_secret.clone()),
        )
        .set_redirect_uri(self.redirect_url.clone());
        Ok(client)
    }
    pub async fn auth_url(&self) -> Result<AuthUrl> {
        let client = self.get_client().await?;

        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_code_challenge)
            .url();

        Ok(AuthUrl {
            issuer_url: self.issuer_url.clone(),
            auth_url,
            csrf_token,
            nonce,
            pkce_code_verifier,
        })
    }

    pub async fn auth_verify(
        &self,
        code: AuthorizationCode,
        pkce_code_verifier: PkceCodeVerifier,
        nonce: Nonce,
        issuer_url: IssuerUrl,
    ) -> Result<OidcClaims> {
        let client = self.get_client().await?;

        if self.issuer_url != issuer_url {
            tracing::error!(
                "BUG: Error provided issuer url does not match the one for this client: {} != {}",
                issuer_url.as_str(),
                self.issuer_url.as_str()
            );
            return Err(api::ApiError::InternalServerError.into());
        }

        let token_response = client
            .exchange_code(code)
            .set_pkce_verifier(pkce_code_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Error exchanging code for {}: {e:?}",
                    self.issuer_url.as_str()
                );
                api::ApiError::InternalServerError
            })?;

        // The two most interesting parts of the token response is the id_token, which is a jwt and
        // the access token. The access token is to make further requests to the oauth provider.
        // We don't need to do that.
        let id_token = token_response.id_token().ok_or_else(|| {
            tracing::error!("OIDC response without id token");
            api::ApiError::InternalServerError
        })?;

        // The claims are actual contents inside the jwt
        let claims = id_token
            .claims(&client.id_token_verifier(), &nonce)
            .map_err(|e| {
                tracing::error!("Invalid OIDC claims: {e:?}");
                api::ApiError::InternalServerError
            })?;

        if claims.issuer() != &self.issuer_url {
            tracing::error!(
                "OIDC response with unexpected issuer: {} != {}",
                claims.issuer().as_str(),
                self.issuer_url.as_str()
            );
            return Err(api::ApiError::InternalServerError.into());
        }

        let email = claims
            .email()
            .ok_or_else(|| {
                tracing::error!("OIDC response without an email");
                api::ApiError::InternalServerError
            })?
            .clone();
        let picture_url = claims.picture().and_then(|p| p.get(None)).cloned();
        let preferred_username = claims.preferred_username().cloned();

        Ok(OidcClaims {
            oidc_id: claims.subject().clone(),
            oidc_issuer_url: self.issuer_url.clone(),
            preferred_username,
            email,
            picture_url,
        })
    }
}
