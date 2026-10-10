//! Omega Showcase: an Inertia.js v3 tour, served by Axum through inertia-omega.

use std::net::SocketAddr;

use omega_showcase::app;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "omega_showcase=info,inertia=info".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(3000);
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("the port should be free");

    tracing::info!("Omega Showcase is running at http://{address}");
    axum::serve(listener, app())
        .await
        .expect("the server should run");
}
