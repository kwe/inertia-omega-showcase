import { Deferred, Head, Link, usePage } from '@inertiajs/react'
import { Card, Feature, PageHeader, Skeleton, time } from '@/components/ui'

interface Props {
  contactsCount: number
  stats?: { contacts: number; posts: number; players: number }
  activity?: string[]
  forecast?: string[]
  tips: { generatedAt: number; items: string[] }
}

export default function Dashboard({ contactsCount, stats, activity, forecast, tips }: Props) {
  const { app, auth } = usePage().props
  // Always set here: the dashboard is behind the sign-in.
  const user = auth.user!

  return (
    <>
      <Head title="Dashboard" />
      <PageHeader title={`Welcome back, ${user.name.split(' ')[0]}`}>
        A tour of Inertia.js v3, served by Rust. Every page here is a React component rendered from an Axum handler:
        no client-side router, and no API. Pick a page on the left; each one names the features it shows.
      </PageHeader>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card title="Shared props" feature="Config::share · inertia.share">
          <p className="text-sm text-stone-600">
            <strong>{app.name}</strong> comes from <code>Config::share</code>, and the signed-in user (
            <strong>{user.email}</strong>) from an Axum middleware calling <code>inertia.share</code>. Every page
            gets both, without each handler passing them.
          </p>
          <p className="mt-2 text-sm text-stone-600">
            Plain prop on this page: <strong>{contactsCount}</strong> contacts.
          </p>
        </Card>

        <Card title="Stats" feature="defer().group('stats')">
          {/* Deferred props are missing from the first response. <Deferred>
              shows the fallback until the client has fetched them. */}
          <Deferred data="stats" fallback={<Skeleton lines={2} />}>
            {stats && (
              <dl className="grid grid-cols-3 gap-3 text-center">
                {Object.entries(stats).map(([label, value]) => (
                  <div key={label} className="rounded-lg bg-stone-50 p-3">
                    <dt className="text-xs text-stone-500 capitalize">{label}</dt>
                    <dd className="text-xl font-semibold tabular-nums">{value}</dd>
                  </div>
                ))}
              </dl>
            )}
          </Deferred>
        </Card>

        <Card title="Recent activity" feature="defer().group('activity')">
          <p className="mb-3 text-xs text-stone-500">
            This group takes 1.5s on the server, and loads in its own request, in parallel with the stats.
          </p>
          <Deferred data="activity" fallback={<Skeleton lines={4} />}>
            <ul className="space-y-1 text-sm">
              {activity?.map((line) => (
                <li key={line} className="flex gap-2">
                  <span className="text-orange-600">•</span>
                  {line}
                </li>
              ))}
            </ul>
          </Deferred>
        </Card>

        <Card title="Forecast" feature="try_lazy().rescue()">
          <p className="mb-3 text-xs text-stone-500">
            This deferred prop fails on the server. It's rescued, so the page still renders and the client shows the
            rescue slot instead.
          </p>
          <Deferred
            data="forecast"
            fallback={<Skeleton lines={2} />}
            rescue={
              <div className="rounded-lg bg-red-50 p-3 text-sm text-red-700">
                The forecast couldn't be loaded. The rest of the page is fine.
              </div>
            }
          >
            <ul className="text-sm">{forecast?.map((line) => <li key={line}>{line}</li>)}</ul>
          </Deferred>
        </Card>

        <Card title="Tips" feature="once().until(60s)">
          <p className="mb-3 text-xs text-stone-500">
            Resolved once and remembered by the client for a minute. Leave this page and come back: the timestamp stays
            at <strong className="tabular-nums">{time(tips.generatedAt)}</strong> because the server skips the prop.
          </p>
          <ul className="space-y-1 text-sm">
            {tips.items.map((tip) => (
              <li key={tip}>💡 {tip}</li>
            ))}
          </ul>
        </Card>

        <Card title="Server-side rendering" feature="ssr::HttpGateway">
          <p className="text-sm text-stone-600">
            First visits are rendered to HTML by Node through <code>HttpGateway</code>, then hydrated. View the page
            source: with SSR on, this text is in the HTML. Without the SSR server, the page quietly renders in the
            browser instead.
          </p>
          <p className="mt-3 text-sm">
            <Link href="/contacts" className="font-medium text-orange-700 hover:underline">
              Next: forms and validation →
            </Link>
          </p>
        </Card>
      </div>

      <p className="mt-6 text-xs text-stone-400">
        Look for the <Feature>labels</Feature> on each card: they name the Rust API behind it.
      </p>
    </>
  )
}
