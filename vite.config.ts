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
  plugins: [inertia(), react(), tailwindcss(), hotFile()],
  resolve: {
    alias: { '@': '/resources/js' },
  },
  server: {
    port: 5173,
    strictPort: true,
    origin: DEV_URL,
    cors: true,
  },
  build: {
    manifest: true,
    outDir: 'public/build',
    rollupOptions: {
      input: 'resources/js/app.tsx',
    },
  },
}))
