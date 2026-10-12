//! One module per showcase page. Each handler returns an Inertia render or a
//! redirect; the `InertiaLayer` turns renders into HTML or JSON.

pub mod auth;
pub mod contacts;
pub mod dashboard;
pub mod errors;
pub mod feed;
pub mod layouts;
pub mod leaderboard;
pub mod live;
pub mod redirects;
pub mod vault;

use std::time::Duration;

/// Pretend a prop comes from a slow service.
pub(crate) async fn slow(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await;
}
