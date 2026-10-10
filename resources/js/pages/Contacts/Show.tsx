import { Head, Link, router } from '@inertiajs/react'
import { Button, Card, PageHeader } from '@/components/ui'
import type { Contact } from '@/types'

export default function ContactsShow({ contact }: { contact: Contact }) {
  const destroy = () => {
    if (confirm(`Delete ${contact.name}?`)) {
      // A DELETE visit; the server's redirect becomes a 303, followed with a GET.
      router.delete(`/contacts/${contact.id}`)
    }
  }

  return (
    <>
      <Head title={contact.name} />
      <PageHeader title={contact.name}>
        You probably got here instantly: hovering the link prefetched this page.
      </PageHeader>

      <Card title="Details" feature="Link prefetch · router.delete">
        <dl className="grid max-w-md grid-cols-[100px_1fr] gap-y-2 text-sm">
          <dt className="text-stone-500">Email</dt>
          <dd>{contact.email}</dd>
          <dt className="text-stone-500">Company</dt>
          <dd>{contact.company}</dd>
          <dt className="text-stone-500">Role</dt>
          <dd>{contact.role}</dd>
        </dl>
        <div className="mt-5 flex gap-2">
          <Link href={`/contacts/${contact.id}/edit`}>
            <Button tabIndex={-1}>Edit</Button>
          </Link>
          <Button variant="danger" onClick={destroy}>
            Delete
          </Button>
          <Link href="/contacts" className="ml-2 self-center text-sm text-stone-600 hover:underline">
            Back to contacts
          </Link>
        </div>
      </Card>
    </>
  )
}
