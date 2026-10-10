import { Head } from '@inertiajs/react'
import { Card, PageHeader, time } from '@/components/ui'
import AppLayout from '@/layouts/AppLayout'
import SectionLayout from '@/layouts/SectionLayout'

function LayoutsIndex({ renderedAt }: { renderedAt: number }) {
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
    </>
  )
}

LayoutsIndex.layout = [AppLayout, SectionLayout]

export default LayoutsIndex
