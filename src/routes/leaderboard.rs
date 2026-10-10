//! Merge props: pages of players are appended to the list on the client, and
//! matched on `id`, so reloading a player already shown updates it in place.

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use inertia::{Inertia, props};
use serde::Deserialize;

use crate::db::Db;

const PER_PAGE: usize = 10;

#[derive(Deserialize)]
pub struct Page {
    page: Option<usize>,
}

pub async fn index(inertia: Inertia, State(db): State<Db>, Query(query): Query<Page>) -> inertia::Response {
    let page = query.page.unwrap_or(1).max(1);
    let players = db.players();
    let total = players.len();
    let start = (page - 1) * PER_PAGE;
    let rows: Vec<_> = players.into_iter().skip(start).take(PER_PAGE).collect();

    inertia.render(
        "Leaderboard",
        props! {
            "players" => inertia::merge(rows).match_on("id"),
            "page" => page,
            "hasMore" => start + PER_PAGE < total,
            // Deep merged: the client keeps keys it already has and updates the rest.
            "summary" => inertia::deep_merge(props! {
                format!("loadedPages.{page}") => true,
                "total" => total,
            }),
        },
    )
}

/// Play a round: a few players score points. The page then reloads the
/// players it shows, and the merge updates their rows instead of duplicating them.
pub async fn play(inertia: Inertia, State(db): State<Db>) -> Response {
    let changed = db.play_round();
    inertia.flash("toast", format!("{} players scored this round.", changed.len()));

    inertia.back().into_response()
}
