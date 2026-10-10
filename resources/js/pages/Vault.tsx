import { Head, router } from '@inertiajs/react'
import { Button, Card, PageHeader, time } from '@/components/ui'

export default function Vault({ secret, openedAt }: { secret: string; openedAt: number }) {
  return (
    <>
      <Head title="Vault" />
      <PageHeader title="Vault">
        This page is rendered with <code>.encrypt_history(true)</code>: its props are encrypted before Inertia stores
        them in the browser's history state.
      </PageHeader>

      <Card title="The secret" feature="encrypt_history · clear_history">
        <blockquote className="rounded-lg bg-stone-900 p-4 font-mono text-sm text-amber-200">{secret}</blockquote>
        <p className="mt-2 text-xs text-stone-500">Opened at {time(openedAt)}.</p>
        <p className="mt-4 text-sm text-stone-600">
          Locking the vault calls <code>inertia.clear_history()</code>, as a logout would. The key is rotated, so after
          the redirect the back button can't restore this page from history: Inertia fetches it from the server again.
        </p>
        <Button className="mt-4" onClick={() => router.post('/vault/lock')}>
          Lock the vault
        </Button>
      </Card>
    </>
  )
}
