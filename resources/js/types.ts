import type { Page } from '@inertiajs/core'

export interface User {
  id: number
  name: string
  email: string
}

/** Props every page receives: from `Config::share` and the auth middleware. */
export interface SharedProps {
  app: { name: string; stack?: string }
  /** `null` on the guest pages (sign in, register) and guests' error pages. */
  auth: { user: User | null }
  [key: string]: unknown
}

export interface Contact {
  id: number
  name: string
  email: string
  company: string
  role: string
}

declare module '@inertiajs/core' {
  export interface InertiaConfig {
    sharedPageProps: SharedProps
    flashDataType: { toast?: string }
  }
}

export type AppPage = Page<SharedProps>
