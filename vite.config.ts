import fs from 'node:fs'
import inertia from '@inertiajs/vite'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig, type Plugin } from 'vite'

const DEV_URL = 'http://localhost:5173'

/**
 * Write the dev server's URL to `public/hot` while it runs, like
 * laravel-vite-plugin does. The Rust server reads it to point the asset tags
 * at the dev server, and to render SSR through the dev server's
 * `/__inertia_ssr` endpoint.
 */
function hotFile(path = 'public/hot'): Plugin {
  return {
    name: 'omega:hot-file',
    apply: 'serve',
    configureServer(server) {
      server.httpServer?.once('listening', () => {
        fs.mkdirSync('public', { recursive: true })
        fs.writeFileSync(path, DEV_URL)
      })

      const clean = () => fs.rmSync(path, { force: true })
      process.on('exit', clean)
      process.on('SIGINT', () => process.exit())
      process.on('SIGTERM', () => process.exit())
    },
  }
}

export default defineConfig(({ command }) => ({
  // Built assets are served by Axum from public/build at /build.
  base: command === 'build' ? '/build/' : '/',
  // Axum serves public/ itself; don't copy it into the build.
  publicDir: false,
  plugins: [
    // The production SSR server (`npm run ssr`) binds 0.0.0.0 by default;
    // only the Axum app on this machine needs to reach it.
    inertia({ ssr: { entry: 'resources/js/app.tsx', host: '127.0.0.1' } }),
    react(),
    tailwindcss(),
    hotFile(),
  ],
  resolve: {
    alias: { '@': '/resources/js' },
  },
  server: {
    port: 5173,
    strictPort: true,
    origin: DEV_URL,
    // Vite's default CORS policy only allows localhost origins, which covers
    // the Axum app on :3000. `cors: true` would let any website you visit read
    // your source from the dev server.
  },
  build: {
    manifest: true,
    outDir: 'public/build',
    rollupOptions: {
      input: 'resources/js/app.tsx',
    },
  },
}))
