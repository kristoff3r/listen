use api::ApiError;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const AUTH_TOKEN_KEY: &str = "auth_token";

#[derive(Copy, Clone)]
pub struct ClientState {
    pub auth_state: AuthState,
}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            auth_state: AuthState::load_from_storage(),
        }
    }
}

pub fn use_client_state() -> ClientState {
    use_context::<ClientState>().expect("Expected ClientState in context")
}

pub fn use_auth_token() -> Signal<Option<AuthToken>> {
    let auth_state = use_client_state().auth_state;
    Signal::derive(move || auth_state.0.get())
}

/// Possibly non-existent auth token.
///
/// The auth token may or may not be present in local storage, so this
/// struct captures that detail.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AuthState(RwSignal<Option<AuthToken>>);

#[cfg(feature = "ssr")]
mod impls {
    use gloo_storage::errors::StorageError;
    use serde::{Deserialize, Serialize};

    pub fn get_local_storage<T>(_key: &str) -> Option<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        None
    }

    pub fn set_local_storage<T: Serialize>(_key: &str, _value: T) -> Result<(), StorageError> {
        Ok(())
    }

    pub fn delete_local_storage(_key: &str) {}
}

#[cfg(not(feature = "ssr"))]
mod impls {
    use gloo_storage::{errors::StorageError, LocalStorage, Storage};
    use serde::{Deserialize, Serialize};

    pub fn get_local_storage<T>(key: &str) -> Option<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        LocalStorage::get(key).ok()
    }

    pub fn set_local_storage<T: Serialize>(key: &str, value: T) -> Result<(), StorageError> {
        LocalStorage::set(key, value)
    }

    pub fn delete_local_storage(key: &str) {
        LocalStorage::delete(key);
    }
}
use impls::*;
use server_fn::{
    client::{browser::BrowserClient, Client},
    request::browser::BrowserRequest,
    response::browser::BrowserResponse,
};

impl AuthState {
    pub fn load_from_storage() -> Self {
        if let Some(auth_token) = get_local_storage(AUTH_TOKEN_KEY) {
            return Self(RwSignal::new(Some(auth_token)));
        }

        Self::clear_storage();
        Self(RwSignal::new(None))
    }

    fn clear_storage() {
        delete_local_storage(AUTH_TOKEN_KEY);
    }

    /// Clears from local storage and sets to none
    pub fn clear(&self) {
        Self::clear_storage();
        self.0.set(None);
    }

    pub fn set(&self, auth_token: AuthToken) {
        if set_local_storage(AUTH_TOKEN_KEY, &auth_token).is_ok() {
            self.0.set(Some(auth_token));
        } else {
            self.clear();
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuthToken(pub String);

impl std::fmt::Display for AuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct AuthClient;

impl Client<ApiError> for AuthClient {
    type Request = BrowserRequest;
    type Response = BrowserResponse;

    fn send(
        req: Self::Request,
    ) -> impl std::future::Future<Output = Result<Self::Response, ServerFnError<ApiError>>> + Send
    {
        async move {
            if let Some(auth_token) = use_auth_token().get_untracked() {
                req.headers()
                    .append("Authorization", &format!("Bearer {auth_token}"));
                BrowserClient::send(req).await
            } else {
                Err(ServerFnError::WrappedServerError(
                    ApiError::MissingAuthToken,
                ))
            }
        }
    }
}
