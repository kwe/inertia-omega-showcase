import { Head, router } from '@inertiajs/react'
import { useState } from 'react'
import { Button, Card, PageHeader } from '@/components/ui'

interface Player {
  id: number
  name: string
  score: number
  updated: number
}

interface Props {
  players: Player[]
  page: number
  hasMore: boolean
  summary: { total: number; loadedPages: Record<string, boolean> }
}

export default function Leaderboard({ players, page, hasMore, summary }: Props) {
  const [busy, setBusy] = useState(false)
  const latest = Math.max(0, ...players.map((player) => player.updated))
  const ranked = [...players].sort((a, b) => b.score - a.score || a.id - b.id)

  // A partial reload of the next page. `players` is a merge prop, so the new
  // rows are appended to the ones already on screen instead of replacing them.
  const loadMore = () =>
    router.reload({
      data: { page: page + 1 },
      only: ['players', 'page', 'hasMore', 'summary'],
      onStart: () => setBusy(true),
      onFinish: () => setBusy(false),
    })

  // After the POST the server redirects to `?upto=<page>`, and the partial
  // reload returns every row on screen again. They're matched on `id`, so
  // players already shown are updated in place rather than duplicated.
  const play = () =>
    router.post('/leaderboard/play', { upto: page }, { only: ['players', 'summary'], preserveScroll: true })

  return (
    <>
      <Head title="Leaderboard" />
      <PageHeader title="Leaderboard">
        <code>inertia::merge(rows).match_on("id")</code> appends each page to the client's list, and updates rows it
        already has. <code>inertia::deep_merge</code> keeps a summary object growing across reloads.
      </PageHeader>

      <Card title={`Top players (${players.length} of ${summary.total} loaded)`} feature="merge · match_on · deep_merge">
        <div className="mb-4 flex flex-wrap items-center gap-2">
          <Button onClick={play}>Play a round</Button>
          <span className="text-xs text-stone-500">
            Pages loaded (deep merged): {Object.keys(summary.loadedPages).join(', ')}
          </span>
        </div>

        <ol className="divide-y divide-stone-100">
          {ranked.map((player, index) => (
            <li
              key={player.id}
              className={`flex items-center gap-3 px-2 py-2 transition-colors ${
                player.updated === latest && latest > 0 ? 'bg-amber-50' : ''
              }`}
            >
              <span className="w-8 text-right text-sm text-stone-400 tabular-nums">{index + 1}</span>
              <span className="flex-1 font-medium">{player.name}</span>
              <span className="text-sm tabular-nums">{player.score}</span>
              <span className="w-12 text-xs text-stone-400">#{player.id}</span>
            </li>
          ))}
        </ol>

        {hasMore && (
          <Button variant="secondary" onClick={loadMore} disabled={busy} className="mt-4 w-full">
            {busy ? 'Loading…' : 'Load more'}
          </Button>
        )}
      </Card>
    </>
  )
}
