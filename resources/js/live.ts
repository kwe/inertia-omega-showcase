import type { LiveChannel, LiveEventHandler, LiveTransport } from '@inertiajs/core'

/**
 * A live props transport over the app's own Server-Sent Events stream
 * (`src/broadcast.rs`), in place of Pusher, Reverb or Echo.
 *
 * One `EventSource` carries every channel. It opens with the first
 * subscription and closes with the last, so pages without live props hold no
 * connection, and it is never touched during SSR.
 */
interface Message {
  channel: LiveChannel
  event: string
  payload: unknown
}

const key = (channel: LiveChannel, event: string) => `${channel.type}:${channel.name}:${event}`

let source: EventSource | null = null
let socketId: string | null = null
const handlers = new Map<string, Set<LiveEventHandler>>()
const statusCallbacks = new Set<(connected: boolean) => void>()

function open() {
  if (source) return
  source = new EventSource('/broadcasts/stream')
  source.addEventListener('connected', (event) => {
    socketId = JSON.parse((event as MessageEvent<string>).data).socketId
    statusCallbacks.forEach((callback) => callback(true))
  })
  source.onmessage = (event) => {
    const message: Message = JSON.parse(event.data)
    handlers.get(key(message.channel, message.event))?.forEach((handler) => handler(message.payload))
  }
  // EventSource reconnects by itself. Inertia refreshes every live prop once
  // it's back, so events missed in between don't leave stale data.
  source.onerror = () => {
    socketId = null
    statusCallbacks.forEach((callback) => callback(false))
  }
}

function close() {
  source?.close()
  source = null
  socketId = null
}

export const sseTransport: LiveTransport = {
  subscribe(channel, event, handler) {
    const id = key(channel, event)
    if (!handlers.has(id)) handlers.set(id, new Set())
    handlers.get(id)!.add(handler)
    open()

    return () => {
      const set = handlers.get(id)
      set?.delete(handler)
      if (set?.size === 0) handlers.delete(id)
      if (handlers.size === 0) close()
    }
  },
  // Sent as `X-Socket-Id`, so the server can mark events this tab caused.
  socketId: () => socketId,
  onStatusChange(callback) {
    statusCallbacks.add(callback)
    return () => statusCallbacks.delete(callback)
  },
}
