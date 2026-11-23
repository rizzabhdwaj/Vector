import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    proxy: {
      '/vector-ws': { target: 'ws://127.0.0.1:3031', ws: true },
      '/vector-api': {
        target: 'http://127.0.0.1:3030',
        rewrite: (path) => path.replace(/^\/vector-api/, ''),
      },
    },
  },
})
