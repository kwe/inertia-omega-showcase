//! Error pages are Inertia pages too, with the right status code.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use inertia::{Inertia, props};

/// The router's fallback: any unknown URL.
pub async fn not_found(inertia: Inertia) -> Response {
    not_found_page(&inertia)
}

pub fn not_found_page(inertia: &Inertia) -> Response {
    (
        StatusCode::NOT_FOUND,
        inertia.render(
            "Error",
            props! {
                "status" => 404,
                "message" => "Sorry, there's nothing at this address.",
            },
        ),
    )
        .into_response()
}
