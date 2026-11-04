import { Form, Head, router } from '@inertiajs/react'
import { useEffect, useRef, useState } from 'react'
import { Button, Card, Field, PageHeader, inputClass, time } from '@/components/ui'

interface Quote {
  symbol: string
  price: number
  change: number
}

interface Message {
  id: number
  author: string
  body: string
  at: number
}

interface Props {
  market: { tick: number; quotes: Quote[] }
  messages: Message[]
  inbox: number[]
}

interface LoggedEvent {
  at: number
  channel: string
  event: string
  props: string[]
}

export default function Broadcasts({ market, messages, inbox }: Props) {
  const [log, setLog] = useState<LoggedEvent[]>([])
  const [latestTick, setLatestTick] = useState(market.tick)
  const [reloads, setReloads] = useState(0)
  const firstTick = useRef(market.tick)

  // Inertia fires `live` for every broadcast event that feeds a prop on this
  // page, before it reloads (or patches) those props.
  useEffect(
    () =>
      router.on('live', ({ detail }) => {
        const payload = detail.payload as { tick?: number }
        if (typeof payload?.tick === 'number') setLatestTick(payload.tick)
        setLog((log) =>
          [
            {
              at: Date.now(),
              channel: `${detail.channel.type}:${detail.channel.name}`,
              event: detail.event,
              props: detail.props,
            },
            ...log,
          ].slice(0, 8),
        )
      }),
    [],
  )

  useEffect(() => {
    if (market.tick !== firstTick.current) setReloads((count) => count + 1)
  }, [market.tick])

  return (
    <>
      <Head title="Broadcasts" />
      <PageHeader title="Broadcasts">
        Live props reload when the browser receives a broadcast event. The server lists each prop's channels and
        events in the page, and Inertia subscribes to them. Here the broadcaster is a Server-Sent Events stream from
        Axum, standing in for Pusher or Reverb.
      </PageHeader>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card title="Market" feature="live() · throttle(5s)">
          <table className="w-full text-sm">
            <tbody>
              {market.quotes.map((quote) => (
                <tr key={quote.symbol} className="border-t border-stone-100 first:border-0">
                  <td className="py-1.5 font-medium">{quote.symbol}</td>
                  <td className="text-right tabular-nums">{(quote.price / 100).toFixed(2)}</td>
                  <td
                    className={`w-20 text-right tabular-nums ${quote.change < 0 ? 'text-red-600' : 'text-green-700'}`}
                  >
                    {quote.change >= 0 ? '+' : ''}
                    {(quote.change / 100).toFixed(2)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <dl className="mt-3 grid grid-cols-3 gap-2 text-center text-xs">
            <Stat label="latest tick" value={latestTick} />
            <Stat label="tick shown" value={market.tick} />
            <Stat label="reloads" value={reloads} />
          </dl>
          <p className="mt-3 text-xs text-stone-500">
            The market ticks every 2 seconds on the public <code>market</code> channel. <code>.throttle(5s)</code>{' '}
            holds the reloads to one per 5 seconds, so the table skips ticks.
          </p>
        </Card>

        <Card title="Your inbox" feature="private channel">
          <p className="text-sm text-stone-600">
            {inbox.length === 0
              ? 'No nudges yet.'
              : `${inbox.length} nudge${inbox.length === 1 ? '' : 's'}, the last at ${time(inbox[inbox.length - 1])}.`}
          </p>
          <Form action="/broadcasts/nudge" method="post" options={{ preserveScroll: true }} className="mt-3">
            {({ processing }) => (
              <Button type="submit" variant="secondary" disabled={processing}>
                Nudge me in 2 seconds
              </Button>
            )}
          </Form>
          <p className="mt-3 text-xs text-stone-500">
            <code>inbox</code> listens on <code>users.{'{id}'}</code>, a private channel. The server only streams it to
            you, so other people's nudges never reach this tab. The nudge arrives after the redirect, so it's the
            broadcast, not the form, that updates the count.
          </p>
        </Card>

        <Card title="Lobby" feature="broadcast_props · toOthers">
          <ul className="mb-3 max-h-64 space-y-2 overflow-y-auto text-sm">
            {messages.map((message) => (
              <li key={message.id}>
                <span className="font-medium">{message.author}</span>{' '}
                <span className="text-xs text-stone-400 tabular-nums">{time(message.at)}</span>
                <div className="text-stone-700">{message.body}</div>
              </li>
            ))}
          </ul>
          <Form
            action="/broadcasts/messages"
            method="post"
            resetOnSuccess
            options={{ preserveScroll: true }}
            className="flex items-start gap-2"
          >
            {({ errors, processing }) => (
              <>
                <div className="flex-1">
                  <Field label="Message" error={errors.body}>
                    <input name="body" className={inputClass} autoComplete="off" />
                  </Field>
                </div>
                <Button type="submit" disabled={processing} className="mt-6">
                  Post
                </Button>
              </>
            )}
          </Form>
          <p className="mt-3 text-xs text-stone-500">
            The event carries the new list (<code>inertia::broadcast_props</code>), so other tabs write it into the
            page with no request. It's marked with this tab's <code>X-Socket-Id</code>, so this tab skips it and uses
            the redirect instead. Open a second tab to watch.
          </p>
        </Card>

        <Card title="Events received" feature="router.on('live')">
          {log.length === 0 ? (
            <p className="text-sm text-stone-500">Waiting for the first event…</p>
          ) : (
            <ul className="space-y-1 font-mono text-xs">
              {log.map((entry, index) => (
                <li key={`${entry.at}-${index}`} className="flex justify-between gap-2">
                  <span>
                    {entry.channel} <strong>{entry.event}</strong>
                  </span>
                  <span className="text-stone-400">→ {entry.props.join(', ')}</span>
                </li>
              ))}
            </ul>
          )}
        </Card>
      </div>
    </>
  )
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg bg-stone-50 p-2">
      <dt className="text-stone-500">{label}</dt>
      <dd className="text-base font-semibold tabular-nums">{value}</dd>
    </div>
  )
}
