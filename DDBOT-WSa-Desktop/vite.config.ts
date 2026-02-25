import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Standard Vue+Vite project config

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    host: true,
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true
      }
    }
  },
})
