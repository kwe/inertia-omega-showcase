import { usePage } from '@inertiajs/react'
import { useEffect, useState } from 'react'

/**
 * Flash data arrives in the page's `flash` field, not its props, so it isn't
 * kept in the browser history: going back doesn't replay the toast.
 */
export default function Toast() {
  const { flash } = usePage()
  const [message, setMessage] = useState<string | null>(null)

  useEffect(() => {
    if (flash.toast) setMessage(flash.toast)
  }, [flash])

  useEffect(() => {
    if (!message) return
    const timer = setTimeout(() => setMessage(null), 3500)
    return () => clearTimeout(timer)
  }, [message])

  if (!message) return null

  return (
    <div
      role="status"
      className="fixed top-4 right-4 z-50 rounded-lg bg-stone-900 px-4 py-3 text-sm text-white shadow-lg [animation:toast-in_150ms_ease-out]"
    >
      {message}
    </div>
  )
}
