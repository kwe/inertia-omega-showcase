import { Link, usePage } from '@inertiajs/react'
import type { ReactNode } from 'react'
import Toast from '@/components/Toast'

/** The layout of the sign-in and register pages: no nav, just the form. */
export default function GuestLayout({ children }: { children: ReactNode }) {
  const { app } = usePage().props

  return (
    <div className="grid min-h-screen place-items-center px-4 py-10">
      <div className="w-full max-w-sm">
        <Link href="/login" className="mb-6 flex items-center justify-center gap-2 text-lg font-semibold tracking-tight">
          <span className="grid size-9 place-items-center rounded-lg bg-orange-600 text-white">Ω</span>
          {app.name}
        </Link>
        <main className="rounded-xl border border-stone-200 bg-white p-6 shadow-sm">{children}</main>
      </div>
      <Toast />
    </div>
  )
}
