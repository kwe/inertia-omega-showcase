import { Form, Head, Link, router } from '@inertiajs/react'
import { Button, Feature, Field, inputClass } from '@/components/ui'
import GuestLayout from '@/layouts/GuestLayout'

interface Props {
  demo: { email: string; password: string }
}

function Login({ demo }: Props) {
  return (
    <>
      <Head title="Sign in" />
      <div className="mb-5 flex items-center justify-between gap-2">
        <h1 className="text-xl font-semibold tracking-tight">Sign in</h1>
        <Feature>Session · Argon2id</Feature>
      </div>

      {/* Failures come back as a validation error on `email`, with the
          password cleared by `resetOnError`. */}
      <Form action="/login" method="post" resetOnError={['password']} className="grid gap-4">
        {({ errors, processing }) => (
          <>
            <Field label="Email" error={errors.email}>
              <input name="email" type="email" autoComplete="username" className={inputClass} autoFocus />
            </Field>
            <Field label="Password" error={errors.password}>
              <input name="password" type="password" autoComplete="current-password" className={inputClass} />
            </Field>
            <Button type="submit" disabled={processing}>
              {processing ? 'Signing in…' : 'Sign in'}
            </Button>
          </>
        )}
      </Form>

      <div className="mt-5 rounded-lg bg-orange-50 p-3 text-sm text-orange-900 ring-1 ring-orange-200">
        <p className="font-medium">Demo account</p>
        <p>
          <code>{demo.email}</code> · <code>{demo.password}</code>
        </p>
        <button
          type="button"
          onClick={() => router.post('/login', demo)}
          className="mt-1 font-medium text-orange-700 hover:underline"
        >
          Sign in with it →
        </button>
      </div>

      <p className="mt-5 text-center text-sm text-stone-600">
        New here?{' '}
        <Link href="/register" className="font-medium text-orange-700 hover:underline">
          Create an account
        </Link>
      </p>
    </>
  )
}

Login.layout = GuestLayout

export default Login
