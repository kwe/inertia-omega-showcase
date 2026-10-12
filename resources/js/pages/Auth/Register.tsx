import { Form, Head, Link } from '@inertiajs/react'
import { Button, Field, inputClass } from '@/components/ui'
import GuestLayout from '@/layouts/GuestLayout'

function Register() {
  return (
    <>
      <Head title="Create an account" />
      <h1 className="mb-5 text-xl font-semibold tracking-tight">Create an account</h1>

      {/* Accounts live in memory, like the rest of the data: restarting the
          server forgets them. */}
      <Form
        action="/register"
        method="post"
        resetOnError={['password', 'password_confirmation']}
        className="grid gap-4"
      >
        {({ errors, processing }) => (
          <>
            <Field label="Name" error={errors.name}>
              <input name="name" autoComplete="name" className={inputClass} autoFocus />
            </Field>
            <Field label="Email" error={errors.email}>
              <input name="email" type="email" autoComplete="email" className={inputClass} />
            </Field>
            <Field label="Password" error={errors.password}>
              <input name="password" type="password" autoComplete="new-password" className={inputClass} />
            </Field>
            <Field label="Confirm password" error={errors.password_confirmation}>
              <input name="password_confirmation" type="password" autoComplete="new-password" className={inputClass} />
            </Field>
            <Button type="submit" disabled={processing}>
              {processing ? 'Creating…' : 'Create account'}
            </Button>
          </>
        )}
      </Form>

      <p className="mt-5 text-center text-sm text-stone-600">
        Already have an account?{' '}
        <Link href="/login" className="font-medium text-orange-700 hover:underline">
          Sign in
        </Link>
      </p>
    </>
  )
}

Register.layout = GuestLayout

export default Register
