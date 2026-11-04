//! Persistent layouts: the layout component stays mounted between these pages.

use std::time::Duration;

use inertia::{Inertia, props};

use crate::db::now_millis;

pub async fn index(inertia: Inertia) -> inertia::Response {
    inertia.render(
        "Layouts/Index",
        props! {
            "renderedAt" => now_millis(),
            // The dashboard's `tips`, under another name. Both are remembered
            // as "tips", so after visiting the dashboard the server skips this
            // and the client fills it in from what it remembers.
            "hints" => inertia::once(super::dashboard::tips)
                .once_as("tips")
                .until(Duration::from_secs(60)),
        },
    )
}

pub async fn other(inertia: Inertia) -> inertia::Response {
    inertia.render("Layouts/Other", props! { "renderedAt" => now_millis() })
}
