//! A small in-process broadcaster for live props, standing in for Pusher,
//! Reverb or Echo.
//!
//! Handlers `send` events on channels. Each browser holds one Server-Sent
//! Events connection (`GET /broadcasts/stream`), and `resources/js/live.ts`
//! hands the events to Inertia, which reloads (or patches) the live props that
//! listen for them.
//!
//! Private channels are authorized on the server: a user only receives
//! `users.{their id}`. A stream ends when its user signs out, and after
//! `MAX_LIFETIME` in any case, so the browser reconnects through the sign-in
//! check and an expired session stops receiving. Every connection gets a
//! socket id, which Inertia sends back as `X-Socket-Id`, so an event can skip
//! the tab that caused it.

use std::convert::Infallible;
use std::time::Duration;

use axum::Extension;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use inertia::{ChannelType, LiveChannel};
use rand::Rng;
use serde::Serialize;
use serde_json::{Value, json};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

use crate::db::{Db, User};

/// Events a slow connection can fall behind by. One that falls further
/// behind is closed, and the browser reconnects and reloads every live prop.
const BUFFER: usize = 64;

/// How long a stream lasts before the browser has to reconnect, which signs
/// it in again.
const MAX_LIFETIME: Duration = Duration::from_secs(5 * 60);

/// One broadcast event, as the browser receives it.
#[derive(Clone, Debug, Serialize)]
pub struct Message {
    pub channel: LiveChannel,
    pub event: String,
    pub payload: Value,
}

/// What goes through the broadcaster: events for browsers, and sign-outs,
/// which end that user's streams.
#[derive(Clone, Debug)]
pub enum Signal {
    Message(Message),
    SignedOut(u64),
}

#[derive(Clone)]
pub struct Broadcaster(broadcast::Sender<Signal>);

impl Default for Broadcaster {
    fn default() -> Self {
        Self(broadcast::channel(BUFFER).0)
    }
}

impl Broadcaster {
    /// Send an event to everyone listening on the channel. Nobody listening
    /// is fine: the event is dropped.
    pub fn send(&self, channel: LiveChannel, event: &str, payload: Value) {
        let _ = self.0.send(Signal::Message(Message {
            channel,
            event: event.to_owned(),
            payload,
        }));
    }

    /// End every stream of this user. Ones from browsers still signed in
    /// reconnect straight away.
    pub fn signed_out(&self, user: u64) {
        let _ = self.0.send(Signal::SignedOut(user));
    }

    /// Whether any browser is connected, so background work can idle.
    pub fn has_listeners(&self) -> bool {
        self.0.receiver_count() > 0
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Signal> {
        self.0.subscribe()
    }
}

/// The private channel only `user` may listen on.
pub fn user_channel(user: &User) -> LiveChannel {
    LiveChannel::private(format!("users.{}", user.id))
}

/// Mark `payload` as caused by the tab with this socket id, so that tab skips
/// it (the `toOthers` of Laravel's broadcasting). Ids that aren't ours are
/// ignored.
pub fn to_others(mut payload: Value, headers: &HeaderMap) -> Value {
    let socket_id = headers
        .get("x-socket-id")
        .and_then(|value| value.to_str().ok())
        .filter(|id| id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit()));

    if let (Some(id), Some(payload)) = (socket_id, payload.as_object_mut()) {
        let envelope = payload.entry("__inertia").or_insert_with(|| json!({}));
        if let Some(envelope) = envelope.as_object_mut() {
            envelope.insert("socketId".to_owned(), json!(id));
        }
    }
    payload
}

/// `GET /broadcasts/stream`: this user's events, as Server-Sent Events. The
/// first event, `connected`, carries the connection's socket id.
pub async fn stream(
    State(db): State<Db>,
    Extension(user): Extension<User>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    let socket_id: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();

    let hello = Event::default()
        .event("connected")
        .data(json!({ "socketId": socket_id }).to_string());

    let mine = user_channel(&user);
    let events = BroadcastStream::new(db.broadcaster().subscribe())
        // `None` ends the stream: on a sign-out, and when the connection
        // lagged, since a skipped event could leave a prop stale. Inertia
        // reloads every live prop once the browser has reconnected.
        .map_while(move |signal| match signal {
            Ok(Signal::Message(message)) => Some(Some(message)),
            Ok(Signal::SignedOut(id)) if id == user.id => None,
            Ok(Signal::SignedOut(_)) => Some(None),
            Err(_lagged) => None,
        })
        .filter_map(move |message| {
            let message = message?;
            let allowed = match message.channel.kind {
                ChannelType::Public => true,
                _ => message.channel == mine,
            };
            allowed.then(|| {
                Event::default()
                    .json_data(&message)
                    .expect("a broadcast message should serialize")
            })
        });

    let stream = tokio_stream::once(hello).chain(events);
    let stream = futures_util::StreamExt::take_until(stream, tokio::time::sleep(MAX_LIFETIME));
    Sse::new(stream.map(Ok)).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_others_only_takes_well_formed_socket_ids() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-socket-id",
            "0123456789abcdef0123456789abcdef".parse().unwrap(),
        );
        let payload = to_others(json!({ "__inertia": { "props": {} } }), &headers);
        assert_eq!(
            payload["__inertia"]["socketId"],
            "0123456789abcdef0123456789abcdef"
        );
        assert!(payload["__inertia"]["props"].is_object());

        headers.insert("x-socket-id", "<script>".parse().unwrap());
        let payload = to_others(json!({}), &headers);
        assert!(payload.get("__inertia").is_none());
    }
}
