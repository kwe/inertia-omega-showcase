import type { ReactNode } from 'react'

export function PageHeader({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <div className="mb-6">
      <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
      {children && <p className="mt-1 max-w-3xl text-sm text-stone-600">{children}</p>}
    </div>
  )
}

export function Card({ title, feature, children }: { title: string; feature?: string; children: ReactNode }) {
  return (
    <section className="rounded-xl border border-stone-200 bg-white p-5 shadow-sm">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <h2 className="font-medium">{title}</h2>
        {feature && <Feature>{feature}</Feature>}
      </div>
      {children}
    </section>
  )
}

/** A label naming the Inertia feature a section demonstrates. */
export function Feature({ children }: { children: ReactNode }) {
  return (
    <code className="rounded-md bg-orange-50 px-2 py-0.5 text-xs font-medium text-orange-700 ring-1 ring-orange-200">
      {children}
    </code>
  )
}

export function Skeleton({ lines = 3 }: { lines?: number }) {
  return (
    <div className="animate-pulse space-y-2">
      {Array.from({ length: lines }, (_, i) => (
        <div key={i} className="h-4 rounded bg-stone-200" style={{ width: `${90 - i * 15}%` }} />
      ))}
    </div>
  )
}

export function Button({
  children,
  variant = 'primary',
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & { variant?: 'primary' | 'secondary' | 'danger' }) {
  const styles = {
    primary: 'bg-orange-600 text-white hover:bg-orange-700',
    secondary: 'bg-white text-stone-800 ring-1 ring-stone-300 hover:bg-stone-100',
    danger: 'bg-red-600 text-white hover:bg-red-700',
  }[variant]

  return (
    <button
      {...props}
      className={`inline-flex items-center justify-center gap-2 rounded-lg px-3.5 py-2 text-sm font-medium transition disabled:opacity-50 ${styles} ${props.className ?? ''}`}
    >
      {children}
    </button>
  )
}

export function Field({
  label,
  error,
  children,
}: {
  label: string
  error?: string
  children: ReactNode
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-sm font-medium text-stone-700">{label}</span>
      {children}
      {error && <span className="mt-1 block text-sm text-red-600">{error}</span>}
    </label>
  )
}

export const inputClass =
  'block w-full rounded-lg border-0 px-3 py-2 text-sm ring-1 ring-stone-300 focus:ring-2 focus:ring-orange-500 focus:outline-none'

/** HH:MM:SS, formatted the same way on the server (SSR) and in the browser. */
export function time(millis: number) {
  const date = new Date(millis)
  return [date.getHours(), date.getMinutes(), date.getSeconds()].map((n) => String(n).padStart(2, '0')).join(':')
}
