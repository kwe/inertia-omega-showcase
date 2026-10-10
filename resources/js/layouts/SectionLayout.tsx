import { Link, usePage } from '@inertiajs/react'
import { useState, type ReactNode } from 'react'
import { Feature } from '@/components/ui'

/**
 * A nested layout, inside AppLayout. Both stay mounted while you move between
 * the two Layouts pages, so the text you type here survives the visit.
 */
export default function SectionLayout({ children }: { children: ReactNode }) {
  const { url } = usePage()
  const [note, setNote] = useState('')

  return (
    <div className="rounded-xl border border-dashed border-orange-300 p-4">
      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <div className="flex gap-1">
          {[
            ['/layouts', 'First page'],
            ['/layouts/other', 'Second page'],
          ].map(([href, label]) => (
            <Link
              key={href}
              href={href}
              className={`rounded-md px-3 py-1.5 text-sm ${url === href ? 'bg-stone-900 text-white' : 'hover:bg-stone-200/60'}`}
            >
              {label}
            </Link>
          ))}
        </div>
        <Feature>Page.layout = [AppLayout, SectionLayout]</Feature>
      </div>
      <input
        value={note}
        onChange={(event) => setNote(event.target.value)}
        placeholder="Type here, then switch pages: the nested layout keeps it."
        className="mb-4 block w-full rounded-lg border-0 px-3 py-2 text-sm ring-1 ring-stone-300 focus:ring-2 focus:ring-orange-500 focus:outline-none"
      />
      {children}
    </div>
  )
}
