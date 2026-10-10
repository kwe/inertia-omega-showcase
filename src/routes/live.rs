//! Polling, partial reloads, `always` props and optional props.

use axum::extract::State;
use inertia::{Inertia, props};
use serde_json::json;

use crate::db::{Db, now_millis};

pub async fn index(inertia: Inertia, State(db): State<Db>) -> inertia::Response {
    let metrics_db = db.clone();

    inertia.render(
        "Live",
        props! {
            // The page polls with `only: ['metrics']`, so this is the only
            // prop computed on each poll...
            "metrics" => inertia::lazy(move || async move {
                let roll = metrics_db.random();
                json!({
                    "requestsPerSecond": 800 + roll % 400,
                    "p99": 12 + (roll >> 12) % 30,
                    "activeUsers": 40 + (roll >> 24) % 60,
                })
            }),
            // ...plus this one, which is sent with every response.
            "serverTime" => inertia::always(now_millis()),
            // Rendered once with the page; partial reloads leave it alone.
            "loadedAt" => now_millis(),
            // Never part of a normal visit: only sent when a partial reload
            // asks for it, here when `<WhenVisible>` scrolls into view.
            "report" => inertia::optional(|| async {
                super::slow(700).await;
                json!({
                    "generatedAt": now_millis(),
                    "rows": [
                        { "region": "eu-west", "uptime": "99.99%", "errors": 3 },
                        { "region": "us-east", "uptime": "99.95%", "errors": 11 },
                        { "region": "ap-south", "uptime": "99.97%", "errors": 6 },
                    ],
                })
            }),
        },
    )
}
