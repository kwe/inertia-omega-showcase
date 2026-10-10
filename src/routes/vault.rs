//! History encryption: the page is encrypted in the browser's history, and
//! clearing the history makes it unrecoverable with the back button.

use axum::response::{IntoResponse, Response};
use inertia::{Inertia, props};

use crate::db::now_millis;

pub async fn index(inertia: Inertia) -> inertia::Response {
    inertia
        .render(
            "Vault",
            props! {
                "secret" => "The crab is the true owner of the borrow checker.",
                "openedAt" => now_millis(),
            },
        )
        .encrypt_history(true)
}

/// Rotate the history encryption key, like a logout would. Pages encrypted
/// with the old key can no longer be restored from the history.
pub async fn lock(inertia: Inertia) -> Response {
    inertia.clear_history();
    inertia.flash(
        "toast",
        "History cleared: the vault can't be restored with the back button.",
    );
    inertia::redirect("/").into_response()
}
