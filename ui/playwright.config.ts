import { defineConfig, devices } from '@playwright/test';

// Свой порт, явно: не 38000/48000 (builder) и не 47613 (dev-сервер для Tauri).
const PORT = Number(process.env.VPN_CLOSER_E2E_PORT ?? 47751);
const BASE_URL = `http://127.0.0.1:${PORT}`;

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: process.env.CI ? 'list' : [['list'], ['html', { open: 'never' }]],
  use: {
    baseURL: BASE_URL,
    locale: 'ru-RU',
    viewport: { width: 720, height: 640 },
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], viewport: { width: 720, height: 640 } },
    },
  ],
  // Интерфейс в браузере работает на моке бэкенда (src/mocks): без Tauri, сети и процессов.
  // Сервер поднимает и гасит сам Playwright.
  webServer: {
    command: `npx vite --port ${PORT} --strictPort`,
    url: BASE_URL,
    reuseExistingServer: false,
    timeout: 60_000,
    env: { VPN_CLOSER_UI_PORT: String(PORT) },
  },
});
