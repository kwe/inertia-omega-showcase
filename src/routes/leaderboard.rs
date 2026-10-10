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
    let players = db.players();
    let total = players.len();
    // Both come from the query string: clamp them to the pages that exist, so
    // `?upto=18446744073709551615` can't build a huge `loadedPages` or overflow.
    let last = total.div_ceil(PER_PAGE).max(1);
    let (first, page) = match (query.page, query.upto) {
        (Some(page), _) => (page.clamp(1, last), page.clamp(1, last)),
        (None, Some(upto)) => (1, upto.clamp(1, last)),
        (None, None) => (1, 1),
    };
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
            "hasMore" => page < last,
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

    // `index` clamps `upto` to the pages that exist.
    inertia::redirect(&format!("/leaderboard?upto={}", play.upto)).into_response()
}
