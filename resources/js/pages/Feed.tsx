import { Head, InfiniteScroll } from '@inertiajs/react'
import { Card, PageHeader, Skeleton } from '@/components/ui'

interface Post {
  id: number
  author: string
  title: string
  body: string
  likes: number
}

interface Props {
  posts: { data: Post[]; current_page: number; last_page: number; total: number }
}

export default function Feed({ posts }: Props) {
  return (
    <>
      <Head title="Feed" />
      <PageHeader title="Feed">
        {posts.total} posts, 15 at a time. <code>inertia::scroll_with</code> wraps a <code>Paginator</code>; as you
        scroll, <code>&lt;InfiniteScroll&gt;</code> asks for the next page with a partial reload and merges it into the
        list. The URL follows along, so a refresh lands on the same page.
      </PageHeader>

      <Card title="Latest posts" feature="scroll_with · Paginator · <InfiniteScroll>">
        <InfiniteScroll
          data="posts"
          buffer={300}
          className="divide-y divide-stone-100"
          loading={() => (
            <div className="py-4">
              <Skeleton lines={2} />
            </div>
          )}
        >
          {posts.data.map((post) => (
            <article key={post.id} className="py-3">
              <div className="flex items-baseline justify-between gap-4">
                <h3 className="font-medium">{post.title}</h3>
                <span className="shrink-0 text-xs text-stone-400">♥ {post.likes}</span>
              </div>
              <p className="text-sm text-stone-600">{post.body}</p>
              <p className="mt-1 text-xs text-stone-400">by {post.author}</p>
            </article>
          ))}
        </InfiniteScroll>
      </Card>
    </>
  )
}
