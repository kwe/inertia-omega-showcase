import { Head, router } from '@inertiajs/react'
import { Button, Card, PageHeader } from '@/components/ui'

const cases = [
  {
    title: 'Redirect back',
    feature: 'inertia.back()',
    description: 'A POST answered with a 302 to the Referer, carrying a flash toast.',
    run: () => router.post('/redirects/back'),
  },
  {
    title: 'PUT, then 303',
    feature: 'redirect after PUT → 303',
    description: 'The layer turns a redirect after PUT, PATCH or DELETE into a 303, so the browser follows it with a GET.',
    run: () => router.put('/redirects/see-other'),
  },
  {
    title: 'Redirect to a #fragment',
    feature: '409 + X-Inertia-Redirect',
    description: "fetch drops URL fragments, so the server answers 409 and the client visits the URL with its fragment.",
    run: () => router.post('/redirects/fragment'),
  },
  {
    title: 'Empty response',
    feature: 'empty 200 → redirect back',
    description: 'A handler that returns a bare 200 to an Inertia visit becomes a redirect back.',
    run: () => router.put('/redirects/empty'),
  },
  {
    title: 'External location',
    feature: 'inertia.location(..)',
    description: 'A 409 with X-Inertia-Location: the client does a full page visit to inertiajs.com.',
    run: () => router.post('/redirects/external'),
  },
]

export default function Redirects() {
  return (
    <>
      <Head title="Redirects" />
      <PageHeader title="Redirects">
        The Inertia protocol has a few redirect rules, all handled by the Rust layer. Each button sends a visit to a
        handler that triggers one; watch the network tab.
      </PageHeader>

      <div className="grid gap-4 lg:grid-cols-2">
        {cases.map((item) => (
          <Card key={item.title} title={item.title} feature={item.feature}>
            <p className="mb-4 text-sm text-stone-600">{item.description}</p>
            <Button variant="secondary" onClick={item.run}>
              Try it
            </Button>
          </Card>
        ))}
      </div>

      <div className="mt-[80vh]" />
      <div id="fragment" className="rounded-xl border-2 border-dashed border-orange-300 bg-orange-50 p-6 text-center">
        <p className="font-medium">#fragment</p>
        <p className="text-sm text-stone-600">The fragment redirect scrolls you here.</p>
      </div>
    </>
  )
}
