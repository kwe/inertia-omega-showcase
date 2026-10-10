//! The protocol's redirect rules, one button each.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use inertia::{Inertia, props};

pub async fn index(inertia: Inertia) -> inertia::Response {
    inertia.render("Redirects", props! {})
}

/// A `302` back to this page, with a flash message. The target is a known
/// route: `inertia.back()` redirects to whatever the `Referer` header says,
/// which a cross-site form can set to any URL.
pub async fn back(inertia: Inertia) -> Response {
    inertia.flash("toast", "Redirected back with a 302.");
    inertia::redirect("/redirects").into_response()
}

/// A redirect after a `PUT` becomes a `303 See Other`, so it's followed with a `GET`.
pub async fn see_other(inertia: Inertia) -> Response {
    inertia.flash("toast", "PUT, then a 303 to a GET.");
    inertia::redirect("/redirects").into_response()
}

/// `fetch` drops URL fragments, so a redirect to one becomes a `409` with an
/// `X-Inertia-Redirect` header, and the client visits it itself.
pub async fn fragment(inertia: Inertia) -> Response {
    inertia.flash("toast", "Redirected to #fragment.");
    inertia::redirect("/redirects#fragment").into_response()
}

/// An external URL: a `409` with `X-Inertia-Location`, and a full page visit.
pub async fn external(inertia: Inertia) -> Response {
    inertia.location("https://inertiajs.com").into_response()
}

/// An empty `200` to an Inertia visit becomes a redirect back to the `Referer`.
/// Only Inertia visits get this, and their `X-Inertia` header can't be sent
/// cross-site without a CORS preflight, so the `Referer` is our own page.
pub async fn empty(inertia: Inertia) -> StatusCode {
    inertia.flash("toast", "An empty 200 became a redirect back.");
    StatusCode::OK
}
