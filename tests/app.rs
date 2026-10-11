//! End-to-end checks of the showcase's Inertia responses, without a browser.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use inertia::Config;
use inertia::testing::AssertablePage;
use omega_showcase::db::Db;
use omega_showcase::{app_with, shared_props};
use serde_json::Value;
use tower::ServiceExt;

/// The app, with a session cookie for the signed-in demo user.
struct TestApp {
    router: Router,
    session: Option<String>,
}

fn guest() -> TestApp {
    TestApp {
        router: app_with(Db::seeded(), shared_props(Config::new().version("test"))),
        session: None,
    }
}

async fn app() -> TestApp {
    let mut app = guest();
    let response = send(
        &app,
        form("POST", "/login")
            .body(Body::from(
                r#"{"email":"ferris@example.com","password":"rustacean"}"#,
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(response.header("location"), Some("/"));
    app.session = Some(response.session_cookie().expect("a session cookie"));
    app
}

struct Sent {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    body: String,
}

impl Sent {
    fn page(&self) -> AssertablePage {
        AssertablePage::from_body(&self.body)
    }

    fn json(&self) -> Value {
        serde_json::from_str(&self.body).expect("an Inertia JSON page")
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }

    /// The `id=..` pair of the session cookie, if the response set one.
    fn session_cookie(&self) -> Option<String> {
        self.headers
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .find(|cookie| cookie.starts_with("id="))
            .and_then(|cookie| cookie.split(';').next())
            .map(str::to_owned)
    }
}

async fn send(app: &TestApp, mut request: Request<Body>) -> Sent {
    if let Some(session) = &app.session {
        request
            .headers_mut()
            .append(header::COOKIE, session.parse().unwrap());
    }
    let response = app.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    Sent {
        status,
        headers,
        body: String::from_utf8(body.to_vec()).unwrap(),
    }
}

fn inertia(method: &str, uri: &str) -> axum::http::request::Builder {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Inertia", "true")
        .header("X-Inertia-Version", "test")
}

const XSRF: &str = "test-token";

/// An Inertia visit that changes data, with the CSRF cookie and the header
/// Inertia's client copies it into.
fn form(method: &str, uri: &str) -> axum::http::request::Builder {
    inertia(method, uri)
        .header(header::COOKIE, format!("XSRF-TOKEN={XSRF}"))
        .header("X-XSRF-TOKEN", XSRF)
        .header(header::CONTENT_TYPE, "application/json")
}

#[tokio::test]
async fn first_visits_render_html_with_shared_and_deferred_props() {
    let response = send(&app().await, Request::get("/").body(Body::empty()).unwrap()).await;

    assert_eq!(response.status, StatusCode::OK);
    response
        .page()
        .component("Dashboard")
        .equals("app.name", "Omega Showcase")
        .equals("auth.user.name", "Ferris Crab")
        .has("tips")
        .missing("stats")
        .deferred("stats", &["stats", "forecast"])
        .deferred("activity", &["activity"]);
}

#[tokio::test]
async fn deferred_groups_load_with_partial_reloads_and_failures_are_rescued() {
    let response = send(
        &app().await,
        inertia("GET", "/")
            .header("X-Inertia-Partial-Component", "Dashboard")
            .header("X-Inertia-Partial-Data", "stats,forecast")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    let page = response.json();
    assert_eq!(page["props"]["stats"]["contacts"], 24);
    assert!(page["props"].get("forecast").is_none());
    assert_eq!(page["rescuedProps"], serde_json::json!(["forecast"]));
}

#[tokio::test]
async fn outdated_assets_force_a_full_reload() {
    let response = send(
        &app().await,
        Request::get("/")
            .header("X-Inertia", "true")
            .header("X-Inertia-Version", "old")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert!(response.header("x-inertia-location").is_some());
}

#[tokio::test]
async fn invalid_forms_redirect_to_the_form_with_errors_in_the_session() {
    let app = app().await;
    let response = send(
        &app,
        form("POST", "/contacts")
            .header(header::REFERER, "https://evil.example/")
            .body(Body::from(r#"{"name":"Al","email":"nope"}"#))
            .unwrap(),
    )
    .await;

    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(response.header("location"), Some("/contacts/create"));
    let cookie = response
        .header("set-cookie")
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();

    let response = send(
        &app,
        inertia("GET", "/contacts/create")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    response
        .page()
        .component("Contacts/Create")
        .equals("errors.name", "The name must be at least 3 characters.")
        .equals("errors.email", "The email must be a valid email address.")
        .equals("errors.company", "The company field is required.");
}

#[tokio::test]
async fn valid_forms_redirect_with_a_flash_message() {
    let app = app().await;
    let response = send(
        &app,
        form("POST", "/contacts")
            .body(Body::from(
                r#"{"name":"Jess Rust","email":"jess@example.com","company":"Crabs","role":"Founder"}"#,
            ))
            .unwrap(),
    )
    .await;

    assert_eq!(response.header("location"), Some("/contacts"));
    let cookie = response
        .header("set-cookie")
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();

    let response = send(
        &app,
        inertia("GET", "/contacts")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    response
        .page()
        .equals("contacts.0.name", "Jess Rust")
        .flash("toast", "Jess Rust was added.");
}

#[tokio::test]
async fn redirects_after_put_and_delete_become_303s() {
    let app = app().await;
    let response = send(
        &app,
        form("PUT", "/contacts/1")
            .body(Body::from(
                r#"{"name":"Ada Lovelace","email":"ada@example.com","company":"Analytical","role":"Engineer"}"#,
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(response.status, StatusCode::SEE_OTHER);
    assert_eq!(response.header("location"), Some("/contacts/1"));

    let response = send(
        &app,
        form("DELETE", "/contacts/1").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(response.status, StatusCode::SEE_OTHER);
    assert_eq!(response.header("location"), Some("/contacts"));
}

#[tokio::test]
async fn external_and_fragment_redirects_use_409s() {
    let app = app().await;

    let response = send(
        &app,
        form("POST", "/redirects/external")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(
        response.header("x-inertia-location"),
        Some("https://inertiajs.com")
    );

    let response = send(
        &app,
        form("POST", "/redirects/fragment")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(
        response.header("x-inertia-redirect"),
        Some("/redirects#fragment")
    );
}

#[tokio::test]
async fn the_feed_is_an_infinite_scroll_prop() {
    let response = send(
        &app().await,
        inertia("GET", "/feed?page=2").body(Body::empty()).unwrap(),
    )
    .await;
    let page = response.json();

    assert_eq!(page["props"]["posts"]["data"].as_array().unwrap().len(), 15);
    assert_eq!(page["props"]["posts"]["current_page"], 2);
    assert_eq!(page["scrollProps"]["posts"]["nextPage"], 3);
    assert_eq!(page["scrollProps"]["posts"]["previousPage"], 1);
}

#[tokio::test]
async fn the_leaderboard_merges_and_matches_on_id() {
    let response = send(
        &app().await,
        inertia("GET", "/leaderboard?upto=2")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let page = response.json();

    assert_eq!(page["props"]["players"].as_array().unwrap().len(), 20);
    assert_eq!(page["mergeProps"], serde_json::json!(["players"]));
    assert_eq!(page["matchPropsOn"], serde_json::json!(["players.id"]));
    assert_eq!(page["deepMergeProps"], serde_json::json!(["summary"]));
}

#[tokio::test]
async fn polls_get_only_what_they_ask_for_plus_always_props() {
    let response = send(
        &app().await,
        inertia("GET", "/live")
            .header("X-Inertia-Partial-Component", "Live")
            .header("X-Inertia-Partial-Data", "metrics")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let props = &response.json()["props"];

    assert!(props.get("metrics").is_some());
    assert!(props.get("serverTime").is_some());
    assert!(props.get("loadedAt").is_none());
    assert!(props.get("report").is_none());
}

#[tokio::test]
async fn the_vault_is_encrypted_in_history() {
    let response = send(
        &app().await,
        inertia("GET", "/vault").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(response.json()["encryptHistory"], true);
}

#[tokio::test]
async fn unknown_urls_render_the_error_page() {
    let response = send(
        &app().await,
        Request::get("/missing").body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    response.page().component("Error").equals("status", 404);
}

#[tokio::test]
async fn forms_without_the_csrf_header_are_refused() {
    let app = app().await;
    let response = send(
        &app,
        inertia("POST", "/contacts")
            .header(header::COOKIE, format!("XSRF-TOKEN={XSRF}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"name":"Mallory","email":"mallory@example.com","company":"Evil","role":"Founder"}"#,
            ))
            .unwrap(),
    )
    .await;

    assert_eq!(response.status.as_u16(), 419);
    response.page().component("Error").equals("status", 419);

    let response = send(
        &app,
        inertia("POST", "/contacts")
            .header(header::COOKIE, format!("XSRF-TOKEN={XSRF}"))
            .header("X-XSRF-TOKEN", "a-guess")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .unwrap(),
    )
    .await;
    assert_eq!(response.status.as_u16(), 419);

    let response = send(
        &app,
        inertia("GET", "/contacts?search=mallory")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(response.json()["props"]["contacts"], serde_json::json!([]));
}

#[tokio::test]
async fn visits_without_a_csrf_cookie_are_given_one() {
    let response = send(&app().await, Request::get("/").body(Body::empty()).unwrap()).await;

    let cookie = response
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|cookie| cookie.starts_with("XSRF-TOKEN="))
        .expect("an XSRF-TOKEN cookie");
    assert!(cookie.contains("SameSite=Strict"));
    assert!(
        !cookie.contains("HttpOnly"),
        "Inertia's client must be able to read it"
    );
}

#[tokio::test]
async fn redirects_back_ignore_the_referer() {
    let app = app().await;

    let response = send(
        &app,
        form("POST", "/redirects/back")
            .header(header::REFERER, "https://evil.example/")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(response.header("location"), Some("/redirects"));

    let response = send(
        &app,
        form("POST", "/contacts/quick")
            .header(header::REFERER, "https://evil.example/")
            .body(Body::from(
                r#"{"name":"Quinn Quick","email":"quinn@example.com"}"#,
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(response.header("location"), Some("/contacts"));

    let response = send(
        &app,
        form("PUT", "/contacts/2")
            .header(header::REFERER, "https://evil.example/")
            .body(Body::from(r#"{"name":"","email":""}"#))
            .unwrap(),
    )
    .await;
    assert_eq!(response.header("location"), Some("/contacts/2/edit"));
}

#[tokio::test]
async fn contact_fields_have_length_limits() {
    let app = app().await;
    let body = serde_json::json!({
        "name": "N".repeat(101),
        "email": format!("{}@example.com", "e".repeat(250)),
        "company": "C".repeat(101),
        "role": "Founder",
    });
    let response = send(
        &app,
        form("POST", "/contacts")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await;
    let cookie = response
        .header("set-cookie")
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();

    let response = send(
        &app,
        inertia("GET", "/contacts/create")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    response
        .page()
        .equals(
            "errors.name",
            "The name may not be longer than 100 characters.",
        )
        .equals(
            "errors.email",
            "The email may not be longer than 254 characters.",
        )
        .equals(
            "errors.company",
            "The company may not be longer than 100 characters.",
        );
}

#[tokio::test]
async fn huge_leaderboard_pages_are_clamped_to_the_last_one() {
    let app = app().await;

    let response = send(
        &app,
        inertia("GET", "/leaderboard?upto=18446744073709551615")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let page = response.json();
    assert_eq!(page["props"]["players"].as_array().unwrap().len(), 40);
    assert_eq!(page["props"]["page"], 4);
    assert_eq!(page["props"]["hasMore"], false);
    assert_eq!(
        page["props"]["summary"]["loadedPages"]
            .as_object()
            .unwrap()
            .len(),
        4
    );

    let response = send(
        &app,
        inertia("GET", "/leaderboard?page=18446744073709551615")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let page = response.json();
    assert_eq!(page["props"]["players"].as_array().unwrap().len(), 10);
    assert_eq!(page["props"]["page"], 4);
    assert_eq!(page["props"]["hasMore"], false);
}

#[tokio::test]
async fn the_vault_is_never_cached() {
    let response = send(
        &app().await,
        Request::get("/vault").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(response.header("cache-control"), Some("no-store"));
}

#[tokio::test]
async fn every_response_has_security_headers() {
    for uri in ["/", "/missing", "/build/missing.js"] {
        let response = send(&app().await, Request::get(uri).body(Body::empty()).unwrap()).await;
        assert_eq!(
            response.header("x-content-type-options"),
            Some("nosniff"),
            "{uri}"
        );
        assert_eq!(response.header("x-frame-options"), Some("DENY"), "{uri}");
        assert!(
            response
                .header("content-security-policy")
                .is_some_and(|csp| csp.contains("frame-ancestors 'none'")),
            "{uri}"
        );
    }
}

#[tokio::test]
async fn guests_are_sent_to_the_login_page_and_back_after_signing_in() {
    let mut app = guest();
    let response = send(
        &app,
        inertia("GET", "/contacts?search=ada")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(response.header("location"), Some("/login"));
    app.session = response.session_cookie();

    let response = send(&app, inertia("GET", "/login").body(Body::empty()).unwrap()).await;
    response
        .page()
        .component("Auth/Login")
        .equals("auth.user", Value::Null)
        .equals("demo.email", "ferris@example.com");

    let response = send(
        &app,
        form("POST", "/login")
            .body(Body::from(
                r#"{"email":"Ferris@Example.com ","password":"rustacean"}"#,
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(response.header("location"), Some("/contacts?search=ada"));
    let session = response.session_cookie().expect("a session cookie");
    assert_ne!(
        Some(&session),
        app.session.as_ref(),
        "signing in gives the session a new id"
    );
    app.session = Some(session);

    let response = send(
        &app,
        inertia("GET", "/contacts").body(Body::empty()).unwrap(),
    )
    .await;
    response
        .page()
        .component("Contacts/Index")
        .equals("auth.user.email", "ferris@example.com")
        .missing("auth.user.password_hash")
        .flash("toast", "Welcome back, Ferris Crab.");
}

#[tokio::test]
async fn unknown_urls_are_still_404s_for_guests() {
    let response = send(
        &guest(),
        Request::get("/missing").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn wrong_passwords_get_one_vague_error() {
    for body in [
        r#"{"email":"ferris@example.com","password":"wrong"}"#,
        r#"{"email":"nobody@example.com","password":"rustacean"}"#,
    ] {
        let mut app = guest();
        let response = send(&app, form("POST", "/login").body(Body::from(body)).unwrap()).await;
        assert_eq!(response.header("location"), Some("/login"));
        app.session = response.session_cookie();

        let response = send(&app, inertia("GET", "/login").body(Body::empty()).unwrap()).await;
        response
            .page()
            .equals("auth.user", Value::Null)
            .equals("errors.email", "These credentials don't match our records.");
    }
}

#[tokio::test]
async fn repeated_failures_lock_the_email_for_a_while() {
    let mut app = guest();
    for _ in 0..5 {
        send(
            &app,
            form("POST", "/login")
                .body(Body::from(
                    r#"{"email":"ferris@example.com","password":"wrong"}"#,
                ))
                .unwrap(),
        )
        .await;
    }

    // Even the right password is refused until the lockout ends.
    let response = send(
        &app,
        form("POST", "/login")
            .body(Body::from(
                r#"{"email":"ferris@example.com","password":"rustacean"}"#,
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(response.header("location"), Some("/login"));
    app.session = response.session_cookie();

    let response = send(&app, inertia("GET", "/login").body(Body::empty()).unwrap()).await;
    let page = response.json();
    let error = page["props"]["errors"]["email"].as_str().unwrap();
    assert!(error.starts_with("Too many sign-in attempts."), "{error}");
}

#[tokio::test]
async fn signed_in_users_skip_the_guest_pages() {
    let app = app().await;
    for uri in ["/login", "/register"] {
        let response = send(&app, inertia("GET", uri).body(Body::empty()).unwrap()).await;
        assert_eq!(response.header("location"), Some("/"), "{uri}");
    }
}

#[tokio::test]
async fn registering_validates_and_then_signs_in() {
    let mut app = guest();
    let response = send(
        &app,
        form("POST", "/register")
            .body(Body::from(
                r#"{"name":"","email":"FERRIS@example.com","password":"short","password_confirmation":""}"#,
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(response.header("location"), Some("/register"));
    app.session = response.session_cookie();

    let response = send(
        &app,
        inertia("GET", "/register").body(Body::empty()).unwrap(),
    )
    .await;
    response
        .page()
        .component("Auth/Register")
        .equals("errors.name", "The name field is required.")
        .equals("errors.email", "That email address is already registered.")
        .equals(
            "errors.password",
            "The password must be at least 8 characters.",
        );

    let response = send(
        &app,
        form("POST", "/register")
            .body(Body::from(
                r#"{"name":"Corro Crab","email":"corro@example.com","password":"unsafe-rust","password_confirmation":"unsafe-rust"}"#,
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(response.header("location"), Some("/"));
    app.session = response.session_cookie();

    let response = send(&app, inertia("GET", "/").body(Body::empty()).unwrap()).await;
    response
        .page()
        .equals("auth.user.name", "Corro Crab")
        .equals("auth.user.email", "corro@example.com");
}

#[tokio::test]
async fn signing_out_ends_the_session_and_clears_history() {
    let mut app = app().await;
    let response = send(&app, form("POST", "/logout").body(Body::empty()).unwrap()).await;
    assert_eq!(response.header("location"), Some("/login"));
    let previous = app.session.replace(response.session_cookie().unwrap());
    assert_ne!(
        previous, app.session,
        "signing out gives the session a new id"
    );

    let response = send(&app, inertia("GET", "/login").body(Body::empty()).unwrap()).await;
    let page = response.json();
    assert_eq!(page["props"]["auth"]["user"], Value::Null);
    assert_eq!(page["clearHistory"], true);
    assert_eq!(page["flash"]["toast"], "You've been signed out.");

    // The old session id no longer signs anyone in.
    app.session = previous;
    let response = send(&app, inertia("GET", "/").body(Body::empty()).unwrap()).await;
    assert_eq!(response.header("location"), Some("/login"));
}

#[tokio::test]
async fn signing_out_needs_the_csrf_header() {
    let app = app().await;
    let response = send(
        &app,
        inertia("POST", "/logout")
            .header(header::COOKIE, format!("XSRF-TOKEN={XSRF}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(response.status.as_u16(), 419);
}
