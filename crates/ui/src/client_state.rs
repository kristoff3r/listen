use leptos::prelude::*;

#[derive(Copy, Clone)]
pub struct ClientState {}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientState {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn use_client_state() -> ClientState {
    use_context::<ClientState>().expect("Expected ClientState in context")
}
