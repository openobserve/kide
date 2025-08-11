import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  test: {
    environment: 'jsdom',
    globals: true
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      'vue': 'vue/dist/vue.esm-bundler.js'
    },
  },
  optimizeDeps: {
    include: [
      'monaco-editor',
      'monaco-editor/esm/vs/basic-languages/yaml/yaml',
      'monaco-editor/esm/vs/language/json/jsonMode'
    ]
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          monaco: [
            'monaco-editor'
          ]
        }
      }
    }
  },
  define: {
    // Monaco Editor environment configuration
    'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV)
  }
})