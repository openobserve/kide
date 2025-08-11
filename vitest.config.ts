import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  define: {
    'import.meta.env.MODE': '"test"'
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/tests/setup.ts'],
    include: [
      'src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}',
      'src/tests/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'
    ],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/tests/',
        'dist/',
        '**/*.d.ts',
        '**/*.config.*'
      ]
    },
    // Mock Tauri APIs and Monaco Editor (YAML + JSON) globally
    alias: {
      '@tauri-apps/api/core': resolve(__dirname, 'src/tests/mocks/tauri-core.ts'),
      '@tauri-apps/api/event': resolve(__dirname, 'src/tests/mocks/tauri-event.ts'),
      'monaco-editor': resolve(__dirname, 'src/tests/mocks/monaco-editor.ts')
    }
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  }
})