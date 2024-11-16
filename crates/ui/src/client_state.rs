use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const AUTH_TOKEN_KEY: &str = "auth_token";
const ROLES_KEY: &str = "roles";

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
    Signal::derive(move || auth_state.0.get().0)
}

pub fn use_roles() -> Signal<Vec<String>> {
    let auth_state = use_client_state().auth_state;
    Signal::derive(move || auth_state.0.get().1)
}

/// Possibly non-existent auth token.
///
/// The auth token may or may not be present in local storage, so this
/// struct captures that detail.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AuthState(RwSignal<(Option<AuthToken>, Vec<String>)>);

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

impl AuthState {
    pub fn load_from_storage() -> Self {
        if let Some(auth_token) = get_local_storage(AUTH_TOKEN_KEY) {
            if let Some(roles) = get_local_storage(ROLES_KEY) {
                return Self(RwSignal::new((Some(auth_token), roles)));
            }
        }

        Self::clear_storage();
        Self(RwSignal::new((None, Vec::new())))
    }

    fn clear_storage() {
        delete_local_storage(AUTH_TOKEN_KEY);
        delete_local_storage(ROLES_KEY);
    }

    /// Clears from local storage and sets to none
    pub fn clear(&self) {
        Self::clear_storage();
        self.0.set((None, Vec::new()));
    }

    pub fn set(&self, auth_token: AuthToken, roles: Vec<String>) {
        if set_local_storage(AUTH_TOKEN_KEY, &auth_token).is_ok()
            && set_local_storage(ROLES_KEY, &roles).is_ok()
        {
            self.0.set((Some(auth_token), roles));
        } else {
            self.clear();
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuthToken(String);

impl std::fmt::Display for AuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
