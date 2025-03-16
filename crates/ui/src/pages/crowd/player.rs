use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use::{
    core::ConnectionReadyState, use_websocket_with_options, UseWebSocketOptions, UseWebSocketReturn,
};

#[derive(Default)]
struct Heartbeat;

// Simple example for usage with `FromToStringCodec`
impl std::fmt::Display for Heartbeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = serde_json::to_string(&(
            time::UtcDateTime::UNIX_EPOCH,
            api::CrowdParticipantCommand::Ping,
        ))
        .unwrap();
        write!(f, "{msg}")
    }
}

#[component]
pub fn CrowdPlayerPage() -> impl IntoView {
    let (message, set_message) = signal("".to_string());
    let (name, set_name) = signal("".to_string());

    let (initial_message_sent, set_initial_message_sent) = signal(false);

    let UseWebSocketReturn {
        ready_state,
        send,
        open,
        close,
        ..
    } = use_websocket_with_options::<String, String, FromToStringCodec, Heartbeat, FromToStringCodec>(
        "/api/crowd/player",
        UseWebSocketOptions::default()
            .on_message(|message| {
                log::info!("Got message: {message:?}");
            })
            .heartbeat(2000)
            .immediate(false),
    );
    let status = move || ready_state.get().to_string();
    let connected = move || ready_state.get() == ConnectionReadyState::Open;

    let connect = move |_| {
        set_initial_message_sent.set(false);
        open();
    };
    let disconnect = move |_| close();

    {
        let send = send.clone();
        Effect::new(move || {
            if connected() && !initial_message_sent.get() {
                send(&name.get_untracked());
                set_initial_message_sent.set(true);
            }
        });
    }

    let send_message = move |_| {
        send(&message.get_untracked().to_string());
    };

    view! {
        <div>
            <p>"Status:" {move || status()}</p>
            <p>
                <button on:click=send_message disabled=move || !connected()>
                    "Send"
                </button>
            </p>
            <p>
                <button on:click=connect disabled=move || connected()>
                    "Connect"
                </button>
            </p>
            <p>
                <button on:click=disconnect disabled=move || !connected()>
                    "Disconnect"
                </button>
            </p>
            <p>
                "Name:"
                <input
                    type="text"
                    on:input:target=move |ev| {
                        set_name.set(ev.target().value());
                    }
                    prop:value=name
                />
            </p>
            <p>
                "Message:"
                <input
                    type="text"
                    on:input:target=move |ev| {
                        set_message.set(ev.target().value());
                    }
                    prop:value=message
                />
            </p>
        </div>
    }
}
