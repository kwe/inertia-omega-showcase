import { Head } from '@inertiajs/react'
import { Card, PageHeader, time } from '@/components/ui'
import AppLayout from '@/layouts/AppLayout'
import SectionLayout from '@/layouts/SectionLayout'

interface Props {
  renderedAt: number
  hints: { generatedAt: number; items: string[] }
}

function LayoutsIndex({ renderedAt, hints }: Props) {
  return (
    <>
      <Head title="Layouts" />
      <PageHeader title="Persistent layouts">
        Every page uses <code>AppLayout</code> by default (the <code>layout</code> option in <code>app.tsx</code>). Its
        "layout up" clock in the header has been counting since your first visit: Inertia swaps the page inside it
        without unmounting it.
      </PageHeader>
      <Card title="First page" feature="nested persistent layouts">
        <p className="text-sm text-stone-600">
          This page was rendered at <strong className="tabular-nums">{time(renderedAt)}</strong>. It sets{' '}
          <code>layout</code> to two layouts, so <code>SectionLayout</code> (the dashed box) nests inside{' '}
          <code>AppLayout</code>.
        </p>
      </Card>
      <div className="mt-4">
        <Card title="Hints" feature="once_as('tips')">
          <p className="mb-3 text-xs text-stone-500">
            The dashboard's tips, remembered under the same key. Visit the dashboard first and this page skips the
            prop: the timestamp, <strong className="tabular-nums">{time(hints.generatedAt)}</strong>, matches the
            dashboard's.
          </p>
          <ul className="space-y-1 text-sm">
            {hints.items.map((hint) => (
              <li key={hint}>💡 {hint}</li>
            ))}
          </ul>
        </Card>
      </div>
    </>
  )
}

LayoutsIndex.layout = [AppLayout, SectionLayout]

export default LayoutsIndex
