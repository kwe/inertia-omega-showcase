import { Link, usePage } from '@inertiajs/react'
import { useEffect, useState, type ReactNode } from 'react'

const nav = [
  { href: '/', label: 'Dashboard', hint: 'Deferred · once · rescue' },
  { href: '/contacts', label: 'Contacts', hint: 'Forms · errors · flash' },
  { href: '/feed', label: 'Feed', hint: 'Infinite scroll' },
  { href: '/leaderboard', label: 'Leaderboard', hint: 'Merge · match on id' },
  { href: '/live', label: 'Live', hint: 'Polling · optional props' },
  { href: '/redirects', label: 'Redirects', hint: '303 · 409 · fragments' },
  { href: '/vault', label: 'Vault', hint: 'History encryption' },
  { href: '/layouts', label: 'Layouts', hint: 'Persistent · nested' },
]

/**
 * The default persistent layout (see app.tsx). Inertia keeps it mounted while
 * pages change inside it, so the clock below keeps counting across visits.
 * A full page reload is the only thing that resets it.
 */
export default function AppLayout({ children }: { children: ReactNode }) {
  const page = usePage()
  const { app, auth } = page.props
  const path = page.url.split(/[?#]/)[0]

  return (
    <div className="min-h-screen">
      <header className="border-b border-stone-200 bg-white">
        <div className="mx-auto flex max-w-6xl items-center justify-between gap-4 px-4 py-3">
          <Link href="/" className="flex items-center gap-2 font-semibold tracking-tight">
            <span className="grid size-8 place-items-center rounded-lg bg-orange-600 text-white">Ω</span>
            {app.name}
          </Link>
          <div className="flex items-center gap-4 text-sm text-stone-500">
            <MountedClock />
            <span className="hidden sm:inline">
              Signed in as <span className="font-medium text-stone-800">{auth.user.name}</span>
            </span>
          </div>
        </div>
      </header>

      <div className="mx-auto grid max-w-6xl gap-6 px-4 py-6 md:grid-cols-[220px_1fr]">
        <nav className="flex gap-1 overflow-x-auto md:flex-col">
          {nav.map((item) => {
            const active = item.href === '/' ? path === '/' : path.startsWith(item.href)
            return (
              // `prefetch` loads the page on hover, and keeps it for 30 seconds,
              // so most clicks render instantly from the cache.
              <Link
                key={item.href}
                href={item.href}
                prefetch
                cacheFor="30s"
                className={`shrink-0 rounded-lg px-3 py-2 text-sm transition ${
                  active ? 'bg-orange-600 text-white' : 'text-stone-700 hover:bg-stone-200/60'
                }`}
              >
                <div className="font-medium">{item.label}</div>
                <div className={`hidden text-xs md:block ${active ? 'text-orange-100' : 'text-stone-400'}`}>
                  {item.hint}
                </div>
              </Link>
            )
          })}
        </nav>

        <main className="min-w-0">{children}</main>
      </div>

      <footer className="mx-auto max-w-6xl px-4 pb-8 text-xs text-stone-400">
        {app.stack ?? 'Axum + inertia-omega'} · asset version <code>{page.version || 'none'}</code>
      </footer>

      <Toast />
    </div>
  )
}

/** Seconds since the layout mounted: proof that it persists between pages. */
function MountedClock() {
  const [seconds, setSeconds] = useState(0)

  useEffect(() => {
    const timer = setInterval(() => setSeconds((s) => s + 1), 1000)
    return () => clearInterval(timer)
  }, [])

  return (
    <span title="The layout has been mounted this long. It persists across Inertia visits." className="tabular-nums">
      layout up {seconds}s
    </span>
  )
}

/**
 * Flash data arrives in the page's `flash` field, not its props, so it isn't
 * kept in the browser history: going back doesn't replay the toast.
 */
function Toast() {
  const { flash } = usePage()
  const [message, setMessage] = useState<string | null>(null)

  useEffect(() => {
    if (flash.toast) setMessage(flash.toast)
  }, [flash])

  useEffect(() => {
    if (!message) return
    const timer = setTimeout(() => setMessage(null), 3500)
    return () => clearTimeout(timer)
  }, [message])

  if (!message) return null

  return (
    <div
      role="status"
      className="fixed top-4 right-4 z-50 rounded-lg bg-stone-900 px-4 py-3 text-sm text-white shadow-lg [animation:toast-in_150ms_ease-out]"
    >
      {message}
    </div>
  )
}
