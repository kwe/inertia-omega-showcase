import { Form, Head, Link } from '@inertiajs/react'
import { Button, Card, Field, PageHeader, inputClass } from '@/components/ui'

export default function ContactsCreate({ roles }: { roles: string[] }) {
  return (
    <>
      <Head title="New contact" />
      <PageHeader title="New contact">
        Submit the form empty, or with a taken email such as <code>ada.lovelace@example.com</code>. The Axum handler
        validates it and redirects back with <code>inertia.back_with_errors(..)</code>.
      </PageHeader>

      <Card title="Contact details" feature="<Form> · back_with_errors · flash · redirect">
        {/* The <Form> component reads its fields from the DOM, sends them as
            JSON and exposes the server's errors, so no state is needed here. */}
        <Form action="/contacts" method="post" className="grid max-w-xl gap-4" disableWhileProcessing>
          {({ errors, processing, hasErrors }) => (
            <>
              {hasErrors && (
                <div className="rounded-lg bg-red-50 p-3 text-sm text-red-700">Please fix the errors below.</div>
              )}
              <Field label="Name" error={errors.name}>
                <input name="name" className={inputClass} autoFocus />
              </Field>
              <Field label="Email" error={errors.email}>
                <input name="email" type="text" className={inputClass} />
              </Field>
              <Field label="Company" error={errors.company}>
                <input name="company" className={inputClass} />
              </Field>
              <Field label="Role" error={errors.role}>
                <select name="role" defaultValue="" className={inputClass}>
                  <option value="" disabled>
                    Choose a role…
                  </option>
                  {roles.map((role) => (
                    <option key={role}>{role}</option>
                  ))}
                </select>
              </Field>
              <div className="flex items-center gap-3">
                <Button type="submit" disabled={processing}>
                  {processing ? 'Saving…' : 'Create contact'}
                </Button>
                <Link href="/contacts" className="text-sm text-stone-600 hover:underline">
                  Cancel
                </Link>
              </div>
            </>
          )}
        </Form>
      </Card>
    </>
  )
}
