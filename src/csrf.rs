//! CSRF protection with a double-submit cookie, the way Laravel does it for
//! Inertia apps.
//!
//! Every response sets a random `XSRF-TOKEN` cookie if the browser doesn't
//! have one yet. Inertia's client reads that cookie and sends it back in an
//! `X-XSRF-TOKEN` header on every same-origin request. Another site can make
//! the browser send the cookie, but can't read it, so it can't send the header:
//! any `POST`, `PUT`, `PATCH` or `DELETE` without a matching header is refused
//! with a `419`.

use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use inertia::{Inertia, props};
use rand::RngCore;

pub const COOKIE: &str = "XSRF-TOKEN";
pub const HEADER: &str = "x-xsrf-token";

/// Whether the cookie gets the `Secure` attribute, for apps served over HTTPS.
#[derive(Clone, Copy, Debug)]
pub struct SecureCookies(pub bool);

pub async fn verify(
    State(SecureCookies(secure)): State<SecureCookies>,
    inertia: Inertia,
    request: Request,
    next: Next,
) -> Response {
    let token = cookie_token(request.headers());

    let safe = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let mut response = if safe || header_matches(request.headers(), token.as_deref()) {
        next.run(request).await
    } else {
        expired_page(&inertia)
    };

    if token.is_none() {
        let cookie = format!(
            "{COOKIE}={}; Path=/; SameSite=Strict{}",
            new_token(),
            if secure { "; Secure" } else { "" }
        );
        if let Ok(cookie) = HeaderValue::try_from(cookie) {
            response.headers_mut().append(header::SET_COOKIE, cookie);
        }
    }

    response
}

/// Laravel's "Page Expired" status, shown when the token is missing or stale.
fn expired_page(inertia: &Inertia) -> Response {
    (
        StatusCode::from_u16(419).expect("419 is a valid status code"),
        inertia.render(
            "Error",
            props! {
                "status" => 419,
                "message" => "This page expired. Reload it and try again.",
            },
        ),
    )
        .into_response()
}

fn cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|cookies| cookies.split(';'))
        .filter_map(|cookie| cookie.trim().split_once('='))
        .find(|(name, value)| *name == COOKIE && !value.is_empty())
        .map(|(_, value)| value.to_owned())
}

fn header_matches(headers: &HeaderMap, token: Option<&str>) -> bool {
    let (Some(token), Some(sent)) = (token, headers.get(HEADER)) else {
        return false;
    };
    constant_time_eq(token.as_bytes(), sent.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0, |diff, (x, y)| diff | (x ^ y)) == 0
}

fn new_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_random_hex() {
        let token = new_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(token, new_token());
    }

    #[test]
    fn the_token_is_read_from_among_other_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("id=abc; XSRF-TOKEN=t0k3n"),
        );
        assert_eq!(cookie_token(&headers).as_deref(), Some("t0k3n"));
    }

    #[test]
    fn the_header_must_match_the_cookie() {
        let mut headers = HeaderMap::new();
        assert!(!header_matches(&headers, Some("t0k3n")));
        headers.insert(HEADER, HeaderValue::from_static("other"));
        assert!(!header_matches(&headers, Some("t0k3n")));
        assert!(!header_matches(&headers, None));
        headers.insert(HEADER, HeaderValue::from_static("t0k3n"));
        assert!(header_matches(&headers, Some("t0k3n")));
    }
}
