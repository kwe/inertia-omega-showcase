# Omega Showcase

A small, complete [Inertia.js](https://inertiajs.com) v3 app with a Rust backend. Axum handles every route and returns a page component with its props; React renders it. There's no client-side router and no JSON API to maintain.

The server side is [inertia-omega](https://github.com/inertiajs/inertia-omega), the Rust port of the official Laravel adapter. This app tries to touch as much of it as it reasonably can, one page per group of features.

| | |
| --- | --- |
| Backend | Axum 0.8, [inertia-omega](https://github.com/inertiajs/inertia-omega) (pinned to `6667d8d`), tower-sessions |
| Frontend | React 19, `@inertiajs/react` 3.9, TypeScript, Tailwind CSS 4 |
| Build | Vite 8 with `@inertiajs/vite` (page resolution and SSR) |
| Data | Seeded in memory on startup, so there's nothing to set up. Restarting resets it. |

## Running it

You need Rust 1.88 or newer and Node 20.19 or newer.

```bash
npm install
```

### Development

Two terminals:

```bash
npm run dev     # Vite on :5173, with hot reload and SSR
cargo run       # the app on http://127.0.0.1:3000
```

Open http://127.0.0.1:3000 (the Rust server, not Vite). While Vite runs, it writes its URL to `public/hot`. The Rust server sees that file, points the asset tags at the dev server, and renders pages server-side through Vite's `/__inertia_ssr` endpoint, so there's no separate SSR process in development.

### Production build

```bash
npm run build   # typecheck, client bundle in public/build, SSR bundle in bootstrap/ssr
npm run ssr     # the Node SSR server on :13714 (optional)
cargo run --release
```

If the SSR server isn't running, pages render in the browser instead; nothing else changes.

### Tests

```bash
cargo test
```

`tests/app.rs` drives the router directly with `tower::ServiceExt::oneshot` and checks each feature at the protocol level with inertia-omega's `AssertablePage`: deferred groups, rescued props, error bags in the session, flash data, 303s, 409s, scroll and merge metadata, and so on.

## What each page shows

Each card in the app has a small orange label naming the Rust API behind it.

| Page | Inertia features | Where to look |
| --- | --- | --- |
| **Dashboard** `/` | Shared props from `Config::share` and from middleware (`inertia.share`); deferred props in two parallel groups (`defer().group(..)`) with `<Deferred>`; a failing deferred prop that's rescued (`try_lazy().rescue()`) and shown in `<Deferred rescue>`; a once prop the client remembers for 60s (`once().until(..)`); SSR | `src/routes/dashboard.rs`, `src/lib.rs`, `pages/Dashboard.tsx` |
| **Contacts** `/contacts` | Partial reloads (`only: ['contacts', 'filters']`) for live search with `preserveState`; link prefetching on hover and on mount (`prefetch="mount"`); the `<Form>` component; error bags (`errorBag="quickAdd"`); `inertia.back()` | `src/routes/contacts.rs`, `pages/Contacts/Index.tsx` |
| **New / Edit contact** | Server-side validation with `ValidationErrors` and `back_with_errors`; flash messages (`inertia.flash`) shown as a toast; `redirect` after `POST`; `useForm` with `isDirty`/`recentlySuccessful`; `PUT` and `DELETE` redirects turned into `303 See Other` | `pages/Contacts/Create.tsx`, `pages/Contacts/Edit.tsx`, `pages/Contacts/Show.tsx` |
| **Feed** `/feed` | Infinite scroll: `inertia::scroll_with` + `Paginator`, rendered by `<InfiniteScroll>`. The URL tracks the page as you scroll. | `src/routes/feed.rs`, `pages/Feed.tsx` |
| **Leaderboard** `/leaderboard` | Merge props (`merge().match_on("id")`): "Load more" appends pages; "Play a round" reloads every visible row and updates them in place instead of duplicating them. Deep merge (`deep_merge`) for a summary object. `only` on a `POST` visit. | `src/routes/leaderboard.rs`, `pages/Leaderboard.tsx` |
| **Live** `/live` | Polling (`usePoll` with `only: ['metrics']`, pausable); lazy props computed only when requested (`inertia::lazy`); `always` props sent on every partial reload; an optional prop (`inertia::optional`) fetched by `<WhenVisible>` when you scroll to it | `src/routes/live.rs`, `pages/Live.tsx` |
| **Redirects** `/redirects` | Every redirect rule of the protocol: back with `302`, `PUT` → `303`, redirect to a `#fragment` (`409` + `X-Inertia-Redirect`), an empty `200` turned into a redirect back, and an external `inertia.location(..)` (`409` + `X-Inertia-Location`) | `src/routes/redirects.rs`, `pages/Redirects.tsx` |
| **Vault** `/vault` | History encryption (`.encrypt_history(true)`) and `inertia.clear_history()` on "lock", as a logout would | `src/routes/vault.rs`, `pages/Vault.tsx` |
| **Layouts** `/layouts` | Persistent layouts: `AppLayout` is the default for every page (the `layout` option in `app.tsx`), and its "layout up Ns" clock keeps counting across visits. These two pages nest a second persistent layout (`Page.layout = [AppLayout, SectionLayout]`) whose text input survives navigation. | `layouts/`, `pages/Layouts/` |
| **Any other URL** | Error pages are Inertia pages with a status code: `(StatusCode::NOT_FOUND, inertia.render("Error", ..))` from the router's fallback | `src/routes/errors.rs`, `pages/Error.tsx` |

And across the whole app:

- **Asset versioning.** The version is a hash of Vite's build manifest (`Config::version_with`, in `src/vite.rs`). To see it work: open the app from a production build, change a page, run `npm run build`, then click a link in the open tab. The server answers `409` with `X-Inertia-Location`, and the browser does a full reload to pick up the new assets. The current version is in the footer. In development it's `dev`, since Vite serves the latest code anyway.
- **Server-side rendering.** `HttpGateway` with a hot file for development and a bundle check for production. View the page source to see the rendered HTML.
- **Progress indicator.** The bar at the top of the page during slow visits (`progress` in `app.tsx`).
- **Page titles.** `<Head title>` plus the `title` callback in `app.tsx`, rendered on the server too.
- **TypeScript types for shared props and flash data.** `resources/js/types.ts` extends `InertiaConfig`, so `usePage().props.auth` and `usePage().flash.toast` are typed.

## How it fits together

```text
src/
├── main.rs          Starts the server on 127.0.0.1:3000 (PORT to change it)
├── lib.rs           Router, Inertia config (version, root view, shared props, SSR) and the auth middleware
├── vite.rs          Asset tags from public/hot or the build manifest, and the asset version
├── db.rs            The in-memory data
└── routes/          One module per page
resources/
├── css/app.css      Tailwind
└── js/
    ├── app.tsx      createInertiaApp: page resolution, default layout, title, progress
    ├── layouts/     AppLayout (nav, toast, clock) and SectionLayout
    ├── pages/       One component per Inertia page
    └── components/  Small UI pieces
tests/app.rs         Protocol-level tests
vite.config.ts       Vite, the Inertia plugin, and the plugin that writes public/hot
```

The middleware order in `src/lib.rs` matters: the session layer wraps the Inertia layer (so flash data and errors have somewhere to live), and the auth middleware sits inside it (so it can call `inertia.share`).

inertia-omega's README links a companion `laravel-omega-vite` crate for Vite asset tags. It isn't published, so `src/vite.rs` is a ~100-line stand-in covering what this app needs.
