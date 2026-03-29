import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
  plugins: [
    vue(),
    VitePWA({
      registerType: 'prompt',
      includeAssets: ['favicon.svg', 'pwa-icon-192.png', 'pwa-icon-512.png'],
      manifest: {
        name: 'Tornare',
        short_name: 'Tornare',
        description: 'Community match operations for Overwatch tournaments',
        theme_color: '#1a1c20',
        background_color: '#1a1c20',
        display: 'standalone',
        start_url: '/',
        icons: [
          { src: 'pwa-icon-192.png', sizes: '192x192', type: 'image/png' },
          { src: 'pwa-icon-512.png', sizes: '512x512', type: 'image/png', purpose: 'any maskable' },
        ],
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2}'],
      },
    }),
  ],
  server: {
    host: '0.0.0.0',
    port: 5173
  }
})

