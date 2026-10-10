import '../css/app.css'

import { createInertiaApp } from '@inertiajs/react'
import AppLayout from './layouts/AppLayout'

// One call runs the app in the browser and, through the @inertiajs/vite
// plugin, renders it on the server for first visits (SSR).
createInertiaApp({
  // Pages are resolved from ./pages: "Contacts/Index" is ./pages/Contacts/Index.tsx.
  pages: './pages',
  // The default persistent layout. It wraps every page and stays mounted
  // between visits, so its state survives navigation.
  layout: () => AppLayout,
  title: (title) => (title ? `${title} · Omega Showcase` : 'Omega Showcase'),
  progress: { color: '#ea580c', delay: 150 },
})
