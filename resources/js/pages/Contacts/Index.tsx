import { Form, Head, Link, router } from '@inertiajs/react'
import { useEffect, useRef, useState } from 'react'
import { Button, Card, Feature, Field, PageHeader, inputClass } from '@/components/ui'
import type { Contact } from '@/types'

interface Props {
  contacts: Contact[]
  filters: { search: string }
}

export default function ContactsIndex({ contacts, filters }: Props) {
  const [search, setSearch] = useState(filters.search)
  const first = useRef(true)

  // Typing reloads only the `contacts` and `filters` props (a partial reload).
  // preserveState keeps the input focused; replace keeps history tidy.
  useEffect(() => {
    if (first.current) {
      first.current = false
      return
    }
    const timer = setTimeout(() => {
      router.get(
        '/contacts',
        search ? { search } : {},
        { only: ['contacts', 'filters'], preserveState: true, preserveScroll: true, replace: true },
      )
    }, 250)
    return () => clearTimeout(timer)
  }, [search])

  return (
    <>
      <Head title="Contacts" />
      <PageHeader title="Contacts">
        Forms with server-side validation in Rust. Errors come back in the <code>errors</code> prop after a redirect,
        and success messages arrive as flash data.
      </PageHeader>

      <div className="grid gap-4 xl:grid-cols-[1fr_320px]">
        <Card title={`${contacts.length} contacts`} feature="partial reload · only: ['contacts']">
          <div className="mb-4 flex gap-2">
            <input
              type="search"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search name, email or company…"
              className={inputClass}
            />
            {/* prefetch="mount" loads the create page as soon as this one renders. */}
            <Link href="/contacts/create" prefetch="mount" className="shrink-0">
              <Button tabIndex={-1}>New contact</Button>
            </Link>
          </div>

          <ul className="divide-y divide-stone-100">
            {contacts.map((contact) => (
              <li key={contact.id}>
                <Link
                  href={`/contacts/${contact.id}`}
                  prefetch
                  className="flex items-center justify-between gap-4 rounded-lg px-2 py-2.5 hover:bg-stone-50"
                >
                  <div className="min-w-0">
                    <div className="truncate font-medium">{contact.name}</div>
                    <div className="truncate text-sm text-stone-500">{contact.email}</div>
                  </div>
                  <div className="shrink-0 text-right text-xs text-stone-500">
                    <div>{contact.company}</div>
                    <div>{contact.role}</div>
                  </div>
                </Link>
              </li>
            ))}
            {contacts.length === 0 && <li className="py-6 text-center text-sm text-stone-500">No matches.</li>}
          </ul>
        </Card>

        <Card title="Quick add" feature="errorBag: 'quickAdd'">
          <p className="mb-3 text-xs text-stone-500">
            This form sets an error bag, so its errors arrive under <code>errors.quickAdd</code> and can't clash with
            another form's. On success the server redirects <em>back</em> with a flash toast.
          </p>
          <Form
            action="/contacts/quick"
            method="post"
            errorBag="quickAdd"
            resetOnSuccess
            options={{ preserveScroll: true }}
            className="space-y-3"
          >
            {({ errors, processing }) => (
              <>
                <Field label="Name" error={errors.name}>
                  <input name="name" className={inputClass} />
                </Field>
                <Field label="Email" error={errors.email}>
                  <input name="email" className={inputClass} />
                </Field>
                <Button type="submit" disabled={processing} className="w-full">
                  {processing ? 'Adding…' : 'Add'}
                </Button>
              </>
            )}
          </Form>
          <p className="mt-3 text-xs text-stone-400">
            Built with the <Feature>&lt;Form&gt;</Feature> component: no form state in React at all.
          </p>
        </Card>
      </div>
    </>
  )
}
