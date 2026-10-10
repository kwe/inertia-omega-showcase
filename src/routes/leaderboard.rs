//! Merge props: pages of players are appended to the list on the client, and
//! matched on `id`, so reloading a player already shown updates it in place.

use axum::Json;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use inertia::{Inertia, props};
use serde::Deserialize;

use crate::db::Db;

const PER_PAGE: usize = 10;

#[derive(Deserialize)]
pub struct Page {
    /// One page of players, for "Load more".
    page: Option<usize>,
    /// Every page up to this one, to refresh all the rows on screen.
    upto: Option<usize>,
}

pub async fn index(
    inertia: Inertia,
    State(db): State<Db>,
    Query(query): Query<Page>,
) -> inertia::Response {
    let (first, page) = match (query.page, query.upto) {
        (Some(page), _) => (page.max(1), page.max(1)),
        (None, Some(upto)) => (1, upto.max(1)),
        (None, None) => (1, 1),
    };
    let players = db.players();
    let total = players.len();
    let start = (first - 1) * PER_PAGE;
    let rows: Vec<_> = players
        .into_iter()
        .skip(start)
        .take((page - first + 1) * PER_PAGE)
        .collect();

    inertia.render(
        "Leaderboard",
        props! {
            "players" => inertia::merge(rows).match_on("id"),
            "page" => page,
            "hasMore" => start + PER_PAGE < total,
            // Deep merged: the client keeps keys it already has and updates the rest.
            "summary" => inertia::deep_merge(props! {
                "loadedPages" => (first..=page).map(|page| (page.to_string(), true)).collect::<inertia::Props>(),
                "total" => total,
            }),
        },
    )
}

#[derive(Deserialize)]
pub struct Play {
    upto: usize,
}

/// Play a round: a few players score points. The redirect reloads every page
/// on screen, and matching on `id` updates those rows instead of duplicating them.
pub async fn play(inertia: Inertia, State(db): State<Db>, Json(play): Json<Play>) -> Response {
    let changed = db.play_round();
    inertia.flash(
        "toast",
        format!("{} players scored this round.", changed.len()),
    );

    inertia::redirect(&format!("/leaderboard?upto={}", play.upto.clamp(1, 10))).into_response()
}
