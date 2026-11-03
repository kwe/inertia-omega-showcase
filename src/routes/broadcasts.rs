//! Live props: props that reload when the browser receives a broadcast event.
//!
//! - `market` listens on a public channel that ticks every two seconds, and is
//!   throttled to one reload every five.
//! - `messages` is sent by value with each event (`broadcast_props`), so other
//!   tabs apply it without a request. The tab that posted skips its own event.
//! - `inbox` listens on the user's private channel, which only they receive.

use std::time::Duration;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use inertia::{Inertia, LiveChannel, LiveListener, ValidationErrors, props};
use serde::Deserialize;
use serde_json::json;

use crate::broadcast::{to_others, user_channel};
use crate::db::{Db, User};

pub const MARKET_TICK: Duration = Duration::from_secs(2);
const MARKET_THROTTLE: Duration = Duration::from_secs(5);
const NUDGE_DELAY: Duration = Duration::from_secs(2);
const MAX_MESSAGE: usize = 200;

fn market() -> LiveChannel {
    LiveChannel::public("market")
}

fn lobby() -> LiveChannel {
    LiveChannel::public("lobby")
}

pub async fn index(
    inertia: Inertia,
    State(db): State<Db>,
    Extension(user): Extension<User>,
) -> inertia::Response {
    let market_db = db.clone();
    let inbox_db = db.clone();
    let user_id = user.id;

    inertia.render(
        "Broadcasts",
        props! {
            // Reloaded on every `PriceTicked`, but at most once per five
            // seconds however often the market ticks.
            market => inertia::lazy(move || async move {
                let (tick, quotes) = market_db.market();
                json!({ "tick": tick, "quotes": quotes })
            })
            .live([LiveListener::new(market(), ["PriceTicked"])])
            .throttle(MARKET_THROTTLE),

            // `post_message` sends the new list along with the event, so
            // listening tabs write it straight into the page.
            messages => inertia::live(db.messages(), [LiveListener::new(lobby(), ["MessagePosted"])]),

            // Only this user receives events on `users.{id}`.
            inbox => inertia::lazy(move || async move { inbox_db.nudges(user_id) })
                .live([LiveListener::new(user_channel(&user), ["Nudged"])]),
        },
    )
}

/// Tick the market while anyone is connected, for as long as the app runs.
pub fn spawn_market(db: Db) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(MARKET_TICK);
        loop {
            interval.tick().await;
            if db.broadcaster().has_listeners() {
                let tick = db.tick_market();
                db.broadcaster()
                    .send(market(), "PriceTicked", json!({ "tick": tick }));
            }
        }
    });
}

#[derive(Deserialize)]
pub struct NewMessage {
    #[serde(default)]
    body: String,
}

pub async fn post_message(
    inertia: Inertia,
    State(db): State<Db>,
    Extension(user): Extension<User>,
    headers: HeaderMap,
    Json(form): Json<NewMessage>,
) -> Response {
    let body = form.body.trim();
    let mut errors = ValidationErrors::new();
    if body.is_empty() {
        errors.add("body", "Write something first.");
    } else if body.chars().count() > MAX_MESSAGE {
        errors.add(
            "body",
            format!("Messages may not be longer than {MAX_MESSAGE} characters."),
        );
    }
    if !errors.is_empty() {
        inertia.with_errors(errors);
        return inertia::redirect("/broadcasts").into_response();
    }

    let messages = db.post_message(&user.name, body);
    match inertia::broadcast_props(
        json!({ "author": user.name }),
        json!({ "messages": messages }),
    ) {
        Ok(payload) => {
            db.broadcaster()
                .send(lobby(), "MessagePosted", to_others(payload, &headers));
        }
        Err(error) => tracing::error!(%error, "the chat broadcast didn't serialize"),
    }

    // This tab gets the new list from the redirect instead.
    inertia::redirect("/broadcasts").into_response()
}

/// Nudge yourself in two seconds, on your private channel. Every tab you have
/// open on this page reloads its `inbox`.
pub async fn nudge(
    inertia: Inertia,
    State(db): State<Db>,
    Extension(user): Extension<User>,
) -> Response {
    if db.begin_nudge(user.id) {
        let db = db.clone();
        tokio::spawn(async move {
            tokio::time::sleep(NUDGE_DELAY).await;
            db.finish_nudge(user.id);
            db.broadcaster()
                .send(user_channel(&user), "Nudged", json!({}));
        });
        inertia.flash("toast", "Nudge on its way: it lands in two seconds.");
    } else {
        inertia.flash("toast", "A nudge is already on its way.");
    }

    inertia::redirect("/broadcasts").into_response()
}
