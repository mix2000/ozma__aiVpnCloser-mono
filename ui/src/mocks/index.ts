import { setFallbackBackend } from 'common/services/api';
import type { IAppState } from 'common/types';
import { STATE_EVENT } from 'entities/monitor';

import { MockBackend } from './mock-backend';

export { MockBackend } from './mock-backend';

declare global {
  interface Window {
    __mock?: MockBackend;
  }
}

/**
 * Вне Tauri (npm run dev, превью): подключить мок бэкенда с отсчётом.
 * Из консоли: window.__mock.setVpn(false).
 */
export function installMockBackend(): MockBackend {
  const mock = new MockBackend({ tick: true });
  window.__mock = mock;
  setFallbackBackend({
    call: (cmd, args) => mock.call(cmd, args),
    listen: <T>(event: string, cb: (payload: T) => void) =>
      event === STATE_EVENT ? mock.subscribe((s: IAppState) => cb(s as T)) : () => undefined,
  });
  return mock;
}
