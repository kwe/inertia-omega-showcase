//! Infinite scroll: a paginated prop the `<InfiniteScroll>` component pages through.

use axum::extract::{Query, State};
use inertia::{Inertia, Paginator, props};
use serde::Deserialize;

use crate::db::Db;

#[derive(Deserialize)]
pub struct Page {
    page: Option<u64>,
}

pub async fn index(
    inertia: Inertia,
    State(db): State<Db>,
    Query(query): Query<Page>,
) -> inertia::Response {
    let page = query.page.unwrap_or(1);

    inertia.render(
        "Feed",
        props! {
            // A scroll prop merges each page into the list the client already
            // has, and tells it which pages come before and after. The client
            // asks for `?page=N` with a partial reload as you scroll.
            "posts" => inertia::scroll_with(move || async move {
                super::slow(300).await;
                Paginator::from_items(db.posts(), 15, page)
            }),
        },
    )
}
