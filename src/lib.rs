//! Omega Showcase: every route is an Inertia page, rendered by React.
//!
//! - `db`: the seeded in-memory data.
//! - `vite`: asset tags and the asset version.
//! - `routes`: one module per showcase page.
//! - `csrf`: the `XSRF-TOKEN` cookie and its check on every form submission.
//! - `session`: an in-memory session store that forgets expired sessions.
//! - `auth`: sign-in sessions, password hashing and the auth middleware.

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderName, HeaderValue, header};
use axum::middleware;
use axum::routing::{get, post, put};
use inertia::axum::InertiaLayer;
use inertia::ssr::HttpGateway;
use inertia::{Config, View, props};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};

pub mod auth;
pub mod csrf;
pub mod db;
pub mod routes;
pub mod session;
pub mod vite;

use crate::csrf::SecureCookies;
use crate::db::Db;
use crate::session::PruningMemoryStore;
use crate::vite::Vite;

/// Form bodies are a few hundred bytes; axum's default limit is 2 MB.
const BODY_LIMIT: usize = 16 * 1024;

/// Sent with every response, the built assets included.
const SECURITY_HEADERS: [(HeaderName, &str); 4] = [
    (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    (header::X_FRAME_OPTIONS, "DENY"),
    (header::REFERRER_POLICY, "same-origin"),
    // No `script-src`: in development the page loads scripts from the Vite
    // dev server and runs an inline React Refresh preamble. A production app
    // should add one, with a nonce for any inline script.
    (
        header::CONTENT_SECURITY_POLICY,
        "frame-ancestors 'none'; base-uri 'self'; form-action 'self'; object-src 'none'",
    ),
];

/// The application, with its routes and middleware.
pub fn app() -> Router {
    app_with(Db::seeded(), inertia_config())
}

pub fn app_with(db: Db, config: Config) -> Router {
    use routes::*;

    // Cookies only get `Secure` when the app is served over HTTPS, since
    // browsers drop them on plain `http://127.0.0.1`.
    let secure = std::env::var("APP_URL").is_ok_and(|url| url.starts_with("https://"));

    // Signed-in users are sent away from these.
    let guest = Router::new()
        .route("/login", get(auth::create_login).post(auth::login))
        .route("/register", get(auth::create_register).post(auth::register))
        .route_layer(middleware::from_fn(crate::auth::require_guest));

    // Guests are sent to /login from these. Unknown URLs, handled by the
    // fallback, still get a 404 rather than a redirect.
    let members = Router::new()
        .route("/", get(dashboard::show))
        .route("/contacts", get(contacts::index).post(contacts::store))
        .route("/contacts/create", get(contacts::create))
        .route("/contacts/quick", post(contacts::quick_store))
        .route(
            "/contacts/{id}",
            get(contacts::show)
                .put(contacts::update)
                .delete(contacts::destroy),
        )
        .route("/contacts/{id}/edit", get(contacts::edit))
        .route("/feed", get(feed::index))
        .route("/leaderboard", get(leaderboard::index))
        .route("/leaderboard/play", post(leaderboard::play))
        .route("/live", get(live::index))
        .route("/redirects", get(redirects::index))
        .route("/redirects/back", post(redirects::back))
        .route("/redirects/see-other", put(redirects::see_other))
        .route("/redirects/fragment", post(redirects::fragment))
        .route("/redirects/external", post(redirects::external))
        .route("/redirects/empty", put(redirects::empty))
        .route("/vault", get(vault::index))
        .route("/vault/lock", post(vault::lock))
        .route("/layouts", get(layouts::index))
        .route("/layouts/other", get(layouts::other))
        .route("/logout", post(auth::logout))
        .route_layer(middleware::from_fn(crate::auth::require_user));

    let router = Router::new()
        .merge(guest)
        .merge(members)
        .fallback(errors::not_found)
        .with_state(db.clone())
        // Inside the Inertia layer, so a rejected form gets an error page.
        .layer(middleware::from_fn_with_state(
            SecureCookies(secure),
            csrf::verify,
        ))
        // Inside the Inertia layer too, so it can share the user with every
        // page, and inside the session layer, which holds the user's id. It
        // wraps the CSRF check, so a 419 page still knows who's signed in.
        .layer(middleware::from_fn_with_state(
            db,
            crate::auth::authenticate,
        ))
        .layer(InertiaLayer::new(config))
        // Outside the Inertia layer, so flash data and errors have a session.
        .layer(
            SessionManagerLayer::new(PruningMemoryStore::default())
                .with_secure(secure)
                .with_expiry(Expiry::OnInactivity(Duration::hours(2))),
        )
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .nest_service("/build", ServeDir::new("public/build"));

    SECURITY_HEADERS
        .into_iter()
        .fold(router, |router, (name, value)| {
            router.layer(SetResponseHeaderLayer::if_not_present(
                name,
                HeaderValue::from_static(value),
            ))
        })
}

/// The Inertia configuration: asset version, root view, shared props and SSR.
pub fn inertia_config() -> Config {
    let vite = Vite::new();

    shared_props(Config::new())
        // A new build changes the manifest, and so the version: clients still
        // running the old assets do a full page load on their next visit.
        .version_with({
            let vite = vite.clone();
            move || vite.version()
        })
        .root_view(move |view: &View<'_>| {
            format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    {assets}
    {head}
</head>
<body class="bg-stone-50 text-stone-900 antialiased">
    {body}
</body>
</html>"#,
                assets = vite.tags(),
                head = view.head,
                body = view.body,
            )
        })
        // First visits are rendered by Node: through the Vite dev server while
        // `npm run dev` runs, or by `npm run ssr` once `npm run build` has
        // built the SSR bundle. If neither is up, the client renders instead.
        .ssr(
            HttpGateway::new()
                .hot_file("public/hot")
                .bundle("bootstrap/ssr/app.js"),
        )
}

/// Props shared with every page. The lazy prop is only computed when it's
/// part of the response, so partial reloads that skip it don't pay for it.
pub fn shared_props(config: Config) -> Config {
    config.share(|_request| {
        props! {
            "app.name" => "Omega Showcase",
            "app.stack" => inertia::lazy(|| async { "Axum 0.8 · inertia-omega · React 19 · Vite" }),
        }
    })
}
