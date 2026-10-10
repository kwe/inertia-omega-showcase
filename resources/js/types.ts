import type { Page } from '@inertiajs/core'

/** Props every page receives: from `Config::share` and the auth middleware. */
export interface SharedProps {
  app: { name: string; stack?: string }
  auth: { user: { name: string; email: string } }
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
