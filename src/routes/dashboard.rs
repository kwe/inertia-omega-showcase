//! Deferred props (in parallel groups), a rescued failing prop, a once prop,
//! and an integer too big for a JavaScript number.

use std::time::Duration;

use axum::extract::{Query, State};
use inertia::{Inertia, props};
use serde::Deserialize;
use serde_json::json;

use super::slow;
use crate::db::{Db, now_millis};

/// 2^53 + 1: the first integer a JavaScript number can't hold.
pub const BIG_ID: u64 = 9_007_199_254_740_993;

#[derive(Deserialize)]
pub struct Options {
    /// `?fresh=true` sends the tips even though the client remembers them.
    #[serde(default)]
    fresh: bool,
}

pub async fn show(
    inertia: Inertia,
    State(db): State<Db>,
    Query(options): Query<Options>,
) -> inertia::Response {
    let stats_db = db.clone();

    // Resolved once, then remembered by the client for a minute: revisit the
    // dashboard and the timestamp stays the same. It's remembered as "tips",
    // which the layouts page reads as `hints`.
    let mut tips = inertia::once(tips)
        .once_as("tips")
        .until(Duration::from_secs(60));
    if options.fresh {
        tips = tips.fresh();
    }

    inertia
        .render(
            "Dashboard",
            props! {
                // A plain prop: part of the first response.
                "contactsCount" => db.contact_count(),

                // Deferred props are left out of the first response. The client
                // asks for them right after, one request per group, so the slow
                // "activity" group doesn't hold up "stats".
                "stats" => inertia::defer(move || async move {
                    slow(400).await;
                    json!({
                        "contacts": stats_db.contact_count(),
                        "posts": stats_db.posts().len(),
                        "players": stats_db.players().len(),
                    })
                }).group("stats"),
                "activity" => inertia::defer(|| async {
                    slow(1500).await;
                    vec![
                        "Ada Lovelace was added to Contacts",
                        "Grace Hopper scored 87 points",
                        "Linus Torvalds updated his email",
                        "Ferris Crab opened the vault",
                    ]
                }).group("activity"),

                // A deferred prop that fails. `rescue()` reports it in the page's
                // `rescuedProps` instead of failing the whole response.
                "forecast" => inertia::try_lazy(|| async {
                    slow(800).await;
                    Err::<Vec<String>, _>("the forecast service is down")
                }).group("stats").rescue(),

                tips => tips,

                // Too big for a JavaScript number, which would round it to
                // ...992. `preserve_big_integers` below sends it so the client
                // revives it as a `BigInt`.
                "bigId" => BIG_ID,
            },
        )
        .preserve_big_integers(true)
}

/// The dashboard's tips, shared with the layouts page through `once_as`.
pub async fn tips() -> serde_json::Value {
    json!({
        "generatedAt": now_millis(),
        "items": [
            "Hover a nav link: the page is prefetched before you click.",
            "Submit an empty contact form to see validation errors.",
            "Open the Broadcasts page in two tabs and post a message.",
        ],
    })
}
