import { Head, Link } from '@inertiajs/react'

export default function Error({ status, message }: { status: number; message: string }) {
  return (
    <>
      <Head title={`${status}`} />
      <div className="rounded-xl border border-stone-200 bg-white p-10 text-center">
        <p className="text-5xl font-bold text-orange-600">{status}</p>
        <p className="mt-3 text-stone-600">{message}</p>
        <p className="mt-1 text-xs text-stone-400">
          An Inertia page rendered with <code>(StatusCode::NOT_FOUND, inertia.render(..))</code>.
        </p>
        <Link href="/" className="mt-6 inline-block text-sm font-medium text-orange-700 hover:underline">
          Back to the dashboard
        </Link>
      </div>
    </>
  )
}
