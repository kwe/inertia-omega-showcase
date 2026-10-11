//! Sign in, register and sign out. The session work is in `crate::auth`.

use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use inertia::{Inertia, ValidationErrors, props};
use serde::Deserialize;
use tower_sessions::Session;

use super::contacts::is_email;
use crate::auth::{self, DEMO_EMAIL, DEMO_PASSWORD};
use crate::db::Db;

const MAX_NAME: usize = 100;
const MAX_EMAIL: usize = 254;
const MIN_PASSWORD: usize = 8;
/// Argon2 hashes any length, but there's no reason to accept a novel.
const MAX_PASSWORD: usize = 128;

pub async fn create_login(inertia: Inertia) -> inertia::Response {
    inertia.render(
        "Auth/Login",
        props! {
            "demo" => props! { "email" => DEMO_EMAIL, "password" => DEMO_PASSWORD },
        },
    )
}

pub async fn create_register(inertia: Inertia) -> inertia::Response {
    inertia.render("Auth/Register", props! {})
}

#[derive(Deserialize)]
pub struct LoginForm {
    #[serde(default)]
    email: String,
    #[serde(default)]
    password: String,
}

/// `POST /login`. Failures redirect back with one vague error, so the form
/// doesn't reveal which emails have accounts.
pub async fn login(
    inertia: Inertia,
    State(db): State<Db>,
    session: Session,
    Json(form): Json<LoginForm>,
) -> Response {
    let email = auth::normalize_email(&form.email);
    let mut errors = ValidationErrors::new();
    if email.is_empty() {
        errors.add("email", "The email field is required.");
    }
    if form.password.is_empty() {
        errors.add("password", "The password field is required.");
    }
    if errors.is_empty()
        && let Some(wait) = db.login_lockout(&email)
    {
        errors.add(
            "email",
            format!(
                "Too many sign-in attempts. Try again in {} seconds.",
                wait.as_secs().max(1)
            ),
        );
    }
    if !errors.is_empty() {
        inertia.with_errors(errors);
        return inertia::redirect("/login").into_response();
    }

    let account = db.account_by_email(&email);
    let hash = account
        .as_ref()
        .map(|account| account.password_hash.clone());
    let verified = auth::verify_password_blocking(form.password, hash).await;
    let account = match account {
        Some(account) if verified => account,
        _ => {
            db.record_login_failure(&email);
            let mut errors = ValidationErrors::new();
            errors.add("email", "These credentials don't match our records.");
            inertia.with_errors(errors);
            return inertia::redirect("/login").into_response();
        }
    };

    db.clear_login_failures(&email);
    if let Err(error) = auth::login(&session, &account.user).await {
        return auth::session_error(error);
    }
    inertia.flash("toast", format!("Welcome back, {}.", account.user.name));

    inertia::redirect(&auth::intended(&session).await).into_response()
}

#[derive(Deserialize)]
pub struct RegisterForm {
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    password_confirmation: String,
}

/// `POST /register`: creates the account and signs it in.
pub async fn register(
    inertia: Inertia,
    State(db): State<Db>,
    session: Session,
    Json(form): Json<RegisterForm>,
) -> Response {
    let email = auth::normalize_email(&form.email);
    let mut errors = validate_registration(&db, &form, &email);
    if !errors.is_empty() {
        inertia.with_errors(errors);
        return inertia::redirect("/register").into_response();
    }

    let hash = auth::hash_password_blocking(form.password).await;
    let Some(user) = db.create_user(&form.name, &email, hash) else {
        errors.add("email", "That email address is already registered.");
        inertia.with_errors(errors);
        return inertia::redirect("/register").into_response();
    };

    if let Err(error) = auth::login(&session, &user).await {
        return auth::session_error(error);
    }
    inertia.flash(
        "toast",
        format!("Welcome, {}. Your account is ready.", user.name),
    );

    inertia::redirect("/").into_response()
}

fn validate_registration(db: &Db, form: &RegisterForm, email: &str) -> ValidationErrors {
    let mut errors = ValidationErrors::new();

    let name = form.name.trim();
    if name.is_empty() {
        errors.add("name", "The name field is required.");
    } else if name.chars().count() > MAX_NAME {
        errors.add(
            "name",
            format!("The name may not be longer than {MAX_NAME} characters."),
        );
    }

    if email.is_empty() {
        errors.add("email", "The email field is required.");
    } else if email.chars().count() > MAX_EMAIL {
        errors.add(
            "email",
            format!("The email may not be longer than {MAX_EMAIL} characters."),
        );
    } else if !is_email(email) {
        errors.add("email", "The email must be a valid email address.");
    } else if db.account_by_email(email).is_some() {
        errors.add("email", "That email address is already registered.");
    }

    let length = form.password.chars().count();
    if length == 0 {
        errors.add("password", "The password field is required.");
    } else if length < MIN_PASSWORD {
        errors.add(
            "password",
            format!("The password must be at least {MIN_PASSWORD} characters."),
        );
    } else if length > MAX_PASSWORD {
        errors.add(
            "password",
            format!("The password may not be longer than {MAX_PASSWORD} characters."),
        );
    } else if form.password != form.password_confirmation {
        errors.add("password_confirmation", "The passwords don't match.");
    }

    errors
}

/// `POST /logout`. Also clears the browser history, so encrypted pages such
/// as the vault can't be brought back with the back button.
pub async fn logout(inertia: Inertia, session: Session) -> Response {
    if let Err(error) = auth::logout(&session).await {
        return auth::session_error(error);
    }
    inertia.clear_history();
    inertia.flash("toast", "You've been signed out.");

    inertia::redirect("/login").into_response()
}
