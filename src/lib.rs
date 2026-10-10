//! Omega Showcase: every route is an Inertia page, rendered by React.
//!
//! - `db`: the seeded in-memory data.
//! - `vite`: asset tags and the asset version.
//! - `routes`: one module per showcase page.

use axum::Router;
use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post, put};
use inertia::axum::InertiaLayer;
use inertia::ssr::HttpGateway;
use inertia::{Config, Inertia, View, props};
use tower_http::services::ServeDir;
use tower_sessions::{MemoryStore, SessionManagerLayer};

pub mod db;
pub mod routes;
pub mod vite;

use crate::db::Db;
use crate::vite::Vite;

/// The application, with its routes and middleware.
pub fn app() -> Router {
    app_with(Db::seeded(), inertia_config())
}

pub fn app_with(db: Db, config: Config) -> Router {
    use routes::*;

    Router::new()
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
        .fallback(errors::not_found)
        .with_state(db)
        // Inside the Inertia layer, so it can share props with every page.
        .layer(middleware::from_fn(share_auth))
        .layer(InertiaLayer::new(config))
        // Outside the Inertia layer, so flash data and errors have a session.
        .layer(SessionManagerLayer::new(MemoryStore::default()).with_secure(false))
        .nest_service("/build", ServeDir::new("public/build"))
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

/// Share the signed-in user with every page, like an auth middleware would.
async fn share_auth(inertia: Inertia, request: Request, next: Next) -> Response {
    inertia.share(
        "auth.user",
        props! {
            "name" => "Ferris Crab",
            "email" => "ferris@example.com",
        },
    );
    next.run(request).await
}
