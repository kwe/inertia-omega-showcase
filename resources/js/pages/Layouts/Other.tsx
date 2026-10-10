import { Head } from '@inertiajs/react'
import { Card, PageHeader, time } from '@/components/ui'
import AppLayout from '@/layouts/AppLayout'
import SectionLayout from '@/layouts/SectionLayout'

function LayoutsOther({ renderedAt }: { renderedAt: number }) {
  return (
    <>
      <Head title="Layouts: second page" />
      <PageHeader title="Persistent layouts">
        A different page component, inside the same two layout instances.
      </PageHeader>
      <Card title="Second page" feature="nested persistent layouts">
        <p className="text-sm text-stone-600">
          Rendered at <strong className="tabular-nums">{time(renderedAt)}</strong>. The note you typed above is still
          there, and the header clock didn't reset.
        </p>
      </Card>
    </>
  )
}

LayoutsOther.layout = [AppLayout, SectionLayout]

export default LayoutsOther
