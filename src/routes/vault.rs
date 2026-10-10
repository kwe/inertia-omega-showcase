//! History encryption: the page is encrypted in the browser's history, and
//! clearing the history makes it unrecoverable with the back button.

use axum::http::header;
use axum::response::{IntoResponse, Response};
use inertia::{Inertia, props};

use crate::db::now_millis;

/// `no-store` too: history encryption only covers `history.state`, and a
/// cached copy of the page would still hold the secret after `lock`.
pub async fn index(inertia: Inertia) -> Response {
    let page = inertia
        .render(
            "Vault",
            props! {
                "secret" => "The crab is the true owner of the borrow checker.",
                "openedAt" => now_millis(),
            },
        )
        .encrypt_history(true);

    ([(header::CACHE_CONTROL, "no-store")], page).into_response()
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
