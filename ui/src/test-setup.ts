// Настройка vitest: матчеры jest-dom (toBeInTheDocument …), очистка DOM после
// каждого теста и то, чего нет в jsdom, но ждёт antd (matchMedia, ResizeObserver).
import '@testing-library/jest-dom/vitest';

import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

afterEach(() => cleanup());

if (typeof window !== 'undefined') {
  window.matchMedia ??= ((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    addListener: () => undefined,
    removeListener: () => undefined,
    dispatchEvent: () => false,
  })) as unknown as typeof window.matchMedia;

  globalThis.ResizeObserver ??= class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;

  // Node со своим (экспериментальным) localStorage оставляет jsdom без рабочего
  // хранилища — подставляем память.
  let usable = false;
  try {
    window.localStorage.setItem('__probe', '1');
    window.localStorage.removeItem('__probe');
    usable = true;
  } catch {
    usable = false;
  }
  if (!usable) {
    const data = new Map<string, string>();
    const storage = {
      getItem: (k: string) => data.get(k) ?? null,
      setItem: (k: string, v: string) => void data.set(k, String(v)),
      removeItem: (k: string) => void data.delete(k),
      clear: () => data.clear(),
      key: (i: number) => [...data.keys()][i] ?? null,
      get length() {
        return data.size;
      },
    };
    Object.defineProperty(window, 'localStorage', { value: storage, configurable: true });
  }
}
