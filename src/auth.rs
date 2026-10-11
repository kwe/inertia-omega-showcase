//! Session-based authentication.
//!
//! Signing in stores the user's id in the session, after giving the session a
//! new id so one planted before sign-in can't be reused. Every request then:
//!
//! - `authenticate`: loads the user, shares it with the page as `auth.user`
//!   (`null` for guests), and puts it in the request extensions.
//! - `require_user`: sends guests to `/login`, remembering where they were
//!   going, on every route except the guest ones.
//! - `require_guest`: sends signed-in users away from `/login` and `/register`.
//!
//! Passwords are hashed with Argon2id, using the crate's default parameters
//! (19 MiB, 2 iterations), which are OWASP's recommendation.

use std::sync::LazyLock;

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use inertia::Inertia;
use tower_sessions::Session;

use crate::db::{Db, User};

/// The seeded account, whose credentials the login page shows.
pub const DEMO_NAME: &str = "Ferris Crab";
pub const DEMO_EMAIL: &str = "ferris@example.com";
pub const DEMO_PASSWORD: &str = "rustacean";

const USER_KEY: &str = "auth.user_id";
const INTENDED_KEY: &str = "auth.intended";

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Argon2 with default parameters should hash any password")
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    })
}

/// Hashing takes tens of milliseconds of CPU, so it runs off the async workers.
pub async fn hash_password_blocking(password: String) -> String {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .expect("hashing should not panic")
}

/// Checks a password against the account's hash, or against a dummy hash
/// when there's no account, so an unknown email takes as long to reject as a
/// wrong password and the timing doesn't reveal who has an account.
pub async fn verify_password_blocking(password: String, hash: Option<String>) -> bool {
    static DUMMY: LazyLock<String> = LazyLock::new(|| hash_password("not anyone's password"));

    tokio::task::spawn_blocking(move || {
        let known = hash.is_some();
        let hash = hash.unwrap_or_else(|| DUMMY.clone());
        verify_password(&password, &hash) && known
    })
    .await
    .expect("verifying should not panic")
}

/// Emails are compared case-insensitively.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Signs the user in on this session, under a new session id.
pub async fn login(session: &Session, user: &User) -> Result<(), Response> {
    session.cycle_id().await.map_err(session_error)?;
    session
        .insert(USER_KEY, user.id)
        .await
        .map_err(session_error)
}

/// Signs the user out, under a new session id. The rest of the session stays,
/// so the page after it can still get a flash message.
pub async fn logout(session: &Session) -> Result<(), Response> {
    session
        .remove::<u64>(USER_KEY)
        .await
        .map_err(session_error)?;
    session.cycle_id().await.map_err(session_error)
}

/// Where to go after signing in: the page the guest was sent away from, or
/// the dashboard.
pub async fn intended(session: &Session) -> String {
    session
        .remove::<String>(INTENDED_KEY)
        .await
        .ok()
        .flatten()
        .filter(|path| is_local_path(path))
        .unwrap_or_else(|| "/".to_owned())
}

/// A path on this site. `//evil.example` and `/\evil.example` are not:
/// browsers treat both as links to another host.
fn is_local_path(path: &str) -> bool {
    path.starts_with('/') && !path.starts_with("//") && !path.starts_with("/\\")
}

fn session_error(error: tower_sessions::session::Error) -> Response {
    tracing::error!(%error, "session store failed");
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

pub async fn authenticate(
    State(db): State<Db>,
    session: Session,
    inertia: Inertia,
    mut request: Request,
    next: Next,
) -> Response {
    let user = match session.get::<u64>(USER_KEY).await {
        Ok(Some(id)) => db.user(id),
        _ => None,
    };

    inertia.share("auth.user", user.clone());
    if let Some(user) = user {
        request.extensions_mut().insert(user);
    }

    next.run(request).await
}

pub async fn require_user(session: Session, request: Request, next: Next) -> Response {
    if request.extensions().get::<User>().is_some() {
        return next.run(request).await;
    }

    // Remember full page visits only: not form submissions, which can't be
    // replayed with a redirect, nor link prefetches the user never clicked.
    let prefetch = request
        .headers()
        .get("purpose")
        .is_some_and(|purpose| purpose == "prefetch");
    if request.method() == Method::GET && !prefetch {
        let path = request
            .uri()
            .path_and_query()
            .map_or("/", |path| path.as_str());
        if let Err(response) = session
            .insert(INTENDED_KEY, path)
            .await
            .map_err(session_error)
        {
            return response;
        }
    }

    inertia::redirect("/login").into_response()
}

pub async fn require_guest(request: Request, next: Next) -> Response {
    if request.extensions().get::<User>().is_some() {
        return inertia::redirect("/").into_response();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwords_are_salted_and_verified() {
        let hash = hash_password("hunter22");
        assert!(hash.starts_with("$argon2id$"));
        assert_ne!(hash, hash_password("hunter22"));
        assert!(verify_password("hunter22", &hash));
        assert!(!verify_password("hunter23", &hash));
        assert!(!verify_password("hunter22", "not a hash"));
    }

    #[tokio::test]
    async fn unknown_accounts_never_verify() {
        assert!(!verify_password_blocking("anything".into(), None).await);
        let hash = hash_password("anything");
        assert!(verify_password_blocking("anything".into(), Some(hash)).await);
    }

    #[test]
    fn only_local_paths_are_followed_after_sign_in() {
        assert!(is_local_path("/contacts?search=ada"));
        assert!(!is_local_path("//evil.example"));
        assert!(!is_local_path("/\\evil.example"));
        assert!(!is_local_path("https://evil.example"));
    }
}
