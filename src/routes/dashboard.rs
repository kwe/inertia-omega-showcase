//! Deferred props (in parallel groups), a rescued failing prop, and a once prop.

use std::time::Duration;

use axum::extract::State;
use inertia::{Inertia, props};
use serde_json::json;

use super::slow;
use crate::db::{Db, now_millis};

pub async fn show(inertia: Inertia, State(db): State<Db>) -> inertia::Response {
    let stats_db = db.clone();

    inertia.render(
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

            // Resolved once, then remembered by the client for a minute:
            // revisit the dashboard and the timestamp stays the same.
            "tips" => inertia::once(|| async {
                json!({
                    "generatedAt": now_millis(),
                    "items": [
                        "Hover a nav link: the page is prefetched before you click.",
                        "Submit an empty contact form to see validation errors.",
                        "Open the Live page and watch it poll.",
                    ],
                })
            }).until(Duration::from_secs(60)),
        },
    )
}
