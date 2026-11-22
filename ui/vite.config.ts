/// <reference types="vitest/config" />
import { fileURLToPath } from 'node:url';

import react from '@vitejs/plugin-react-swc';
import { defineConfig } from 'vite';

const src = (dir: string) => fileURLToPath(new URL(`./src/${dir}`, import.meta.url));

// Порт dev-сервера: Tauri ждёт его же (crates/app/tauri.conf.json → devUrl).
// Не 38000/48000 — там builder.
const PORT = Number(process.env.VPN_CLOSER_UI_PORT ?? 47613);

export default defineConfig({
  base: './',
  plugins: [react()],
  // Абсолютные импорты от src, как в builder (те же пути — в tsconfig.app.json).
  resolve: {
    alias: {
      common: src('common'),
      entities: src('entities'),
      mocks: src('mocks'),
      pages: src('pages'),
      providers: src('providers'),
    },
  },
  clearScreen: false,
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    chunkSizeWarningLimit: 2000,
    target: 'es2021',
  },
  server: { host: '127.0.0.1', port: PORT, strictPort: true },
  preview: { host: '127.0.0.1', port: PORT, strictPort: true },
  test: {
    include: ['src/**/*.test.{ts,tsx}'],
    environment: 'jsdom',
    setupFiles: ['./src/test-setup.ts'],
    css: { modules: { classNameStrategy: 'non-scoped' } },
    testTimeout: 20_000,
    coverage: {
      provider: 'v8',
      reporter: ['text-summary', 'text', 'html'],
      include: ['src/**/*.{ts,tsx}'],
      // Не считаем: точку входа и корень (их проверяет e2e), моки и тестовые обёртки
      // (инструменты тестов), модели и index (только типы и реэкспорты).
      exclude: [
        'src/main.tsx',
        'src/app.tsx',
        'src/mocks/**',
        'src/common/test/**',
        'src/**/*.test.{ts,tsx}',
        'src/test-setup.ts',
        'src/vite-env.d.ts',
        'src/**/models.ts',
        'src/common/types/**',
        'src/**/index.ts',
      ],
      thresholds: { lines: 90, statements: 90, functions: 90, branches: 85 },
    },
  },
});
