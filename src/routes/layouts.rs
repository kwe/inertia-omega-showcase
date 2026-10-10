//! Persistent layouts: the layout component stays mounted between these pages.

use inertia::{Inertia, props};

use crate::db::now_millis;

pub async fn index(inertia: Inertia) -> inertia::Response {
    inertia.render("Layouts/Index", props! { "renderedAt" => now_millis() })
}

pub async fn other(inertia: Inertia) -> inertia::Response {
    inertia.render("Layouts/Other", props! { "renderedAt" => now_millis() })
}
