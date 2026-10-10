import { Head, WhenVisible, router, usePoll } from '@inertiajs/react'
import { Button, Card, PageHeader, Skeleton, time } from '@/components/ui'

interface Props {
  metrics: { requestsPerSecond: number; p99: number; activeUsers: number }
  serverTime: number
  loadedAt: number
  report?: { generatedAt: number; rows: { region: string; uptime: string; errors: number }[] }
}

export default function Live({ metrics, serverTime, loadedAt, report }: Props) {
  // Reload only `metrics` every two seconds. Polling pauses while the tab is
  // in the background, and stops when you leave the page.
  const { polling, start, stop } = usePoll(2000, { only: ['metrics'] })

  return (
    <>
      <Head title="Live" />
      <PageHeader title="Live">
        A page that keeps itself fresh with <code>usePoll</code>. Each poll is a partial reload: the server only
        computes the props it's asked for.
      </PageHeader>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card title="Metrics" feature="usePoll · only: ['metrics']">
          <dl className="grid grid-cols-3 gap-3 text-center">
            <Metric label="req/s" value={metrics.requestsPerSecond} />
            <Metric label="p99 ms" value={metrics.p99} />
            <Metric label="users" value={metrics.activeUsers} />
          </dl>
          <div className="mt-4 flex items-center gap-2">
            <Button variant="secondary" onClick={polling ? stop : start}>
              {polling ? 'Pause polling' : 'Resume polling'}
            </Button>
            <span className={`size-2 rounded-full ${polling ? 'animate-pulse bg-green-500' : 'bg-stone-300'}`} />
          </div>
        </Card>

        <Card title="Clocks" feature="always() · partial reloads">
          <dl className="space-y-2 text-sm">
            <div className="flex justify-between">
              <dt className="text-stone-500">serverTime (always)</dt>
              <dd className="tabular-nums">{time(serverTime)}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-stone-500">loadedAt (plain)</dt>
              <dd className="tabular-nums">{time(loadedAt)}</dd>
            </div>
          </dl>
          <p className="mt-3 text-xs text-stone-500">
            <code>serverTime</code> is an <code>always</code> prop, so it comes with every poll even though the polls
            only ask for <code>metrics</code>. <code>loadedAt</code> is a plain prop and stays put.
          </p>
        </Card>
      </div>

      <div className="mt-[60vh]" />

      <Card title="Regional report" feature="optional() · <WhenVisible>">
        <p className="mb-3 text-xs text-stone-500">
          <code>report</code> is an optional prop: never sent on a normal visit. <code>&lt;WhenVisible&gt;</code> asks
          for it once you scroll here.
        </p>
        <WhenVisible data="report" fallback={<Skeleton lines={3} />}>
          {report && (
            <>
              <table className="w-full text-sm">
                <thead className="text-left text-xs text-stone-500">
                  <tr>
                    <th className="py-1">Region</th>
                    <th>Uptime</th>
                    <th className="text-right">Errors</th>
                  </tr>
                </thead>
                <tbody>
                  {report.rows.map((row) => (
                    <tr key={row.region} className="border-t border-stone-100">
                      <td className="py-1.5">{row.region}</td>
                      <td>{row.uptime}</td>
                      <td className="text-right tabular-nums">{row.errors}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <div className="mt-3 flex items-center justify-between text-xs text-stone-500">
                <span>Generated at {time(report.generatedAt)}</span>
                <Button variant="secondary" onClick={() => router.reload({ only: ['report'] })}>
                  Regenerate
                </Button>
              </div>
            </>
          )}
        </WhenVisible>
      </Card>
    </>
  )
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg bg-stone-50 p-3">
      <dt className="text-xs text-stone-500">{label}</dt>
      <dd className="text-xl font-semibold tabular-nums">{value}</dd>
    </div>
  )
}
