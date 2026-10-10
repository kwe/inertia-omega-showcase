import { Head, Link, useForm } from '@inertiajs/react'
import type { FormEvent } from 'react'
import { Button, Card, Field, PageHeader, inputClass } from '@/components/ui'
import type { Contact } from '@/types'

export default function ContactsEdit({ contact, roles }: { contact: Contact; roles: string[] }) {
  // useForm holds the form state in React, tracks dirtiness and sends a PUT.
  const form = useForm({
    name: contact.name,
    email: contact.email,
    company: contact.company,
    role: contact.role,
  })

  const submit = (event: FormEvent) => {
    event.preventDefault()
    form.put(`/contacts/${contact.id}`, { preserveScroll: true })
  }

  return (
    <>
      <Head title={`Edit ${contact.name}`} />
      <PageHeader title={`Edit ${contact.name}`}>
        The same validation, through the <code>useForm</code> hook. A redirect after a <code>PUT</code> is turned into
        a <code>303 See Other</code> by the Rust layer.
      </PageHeader>

      <Card title="Contact details" feature="useForm · PUT → 303">
        <form onSubmit={submit} className="grid max-w-xl gap-4">
          <Field label="Name" error={form.errors.name}>
            <input value={form.data.name} onChange={(e) => form.setData('name', e.target.value)} className={inputClass} />
          </Field>
          <Field label="Email" error={form.errors.email}>
            <input
              value={form.data.email}
              onChange={(e) => form.setData('email', e.target.value)}
              className={inputClass}
            />
          </Field>
          <Field label="Company" error={form.errors.company}>
            <input
              value={form.data.company}
              onChange={(e) => form.setData('company', e.target.value)}
              className={inputClass}
            />
          </Field>
          <Field label="Role" error={form.errors.role}>
            <select value={form.data.role} onChange={(e) => form.setData('role', e.target.value)} className={inputClass}>
              {roles.map((role) => (
                <option key={role}>{role}</option>
              ))}
            </select>
          </Field>
          <div className="flex items-center gap-3">
            <Button type="submit" disabled={form.processing || !form.isDirty}>
              {form.processing ? 'Saving…' : 'Save changes'}
            </Button>
            {form.isDirty && <span className="text-xs text-amber-700">Unsaved changes</span>}
            {form.recentlySuccessful && <span className="text-xs text-green-700">Saved</span>}
            <Link href={`/contacts/${contact.id}`} className="text-sm text-stone-600 hover:underline">
              Cancel
            </Link>
          </div>
        </form>
      </Card>
    </>
  )
}
