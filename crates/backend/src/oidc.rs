use oauth2::{
    ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    IssuerUrl, Nonce,
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

    pub async fn auth_url(&self) -> Result<AuthUrl> {
        // TODO: Cache provider metadata?
        let provider_metadata = match CoreProviderMetadata::discover_async(
            self.issuer_url.clone(),
            oauth2::reqwest::async_http_client,
        )
        .await
        {
            Ok(provider_metadata) => provider_metadata,
            Err(e) => {
                tracing::error!(
                    "Error doing metadata discovery for {}: {e:?}",
                    self.issuer_url.as_str()
                );
                return Err(api::ApiError::InternalServerError.into());
            }
        };

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            self.client_id.clone(),
            Some(self.client_secret.clone()),
        )
        .set_redirect_uri(self.redirect_url.clone());

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
}
