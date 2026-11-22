// Транспорт к бэкенду. В окне Tauri — invoke/listen, иначе — подставной
// бэкенд (мок для браузера и тестов, см. src/mocks). О сущностях не знает.
import { NO_BACKEND_TEXT } from './api.consts';
import { errorText } from './api-errors';
import type { ApiArgs, IFallbackBackend, Unsubscribe } from './api.types';

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export function inTauri(): boolean {
  return typeof window !== 'undefined' && window.__TAURI_INTERNALS__ !== undefined;
}

let fallback: IFallbackBackend | null = null;

/** Подключить подставной бэкенд (вне Tauri). */
export function setFallbackBackend(b: IFallbackBackend | null) {
  fallback = b;
}

function getFallback(): IFallbackBackend {
  if (!fallback) throw new Error(NO_BACKEND_TEXT);
  return fallback;
}

/** Вызвать команду. Ошибка — Error с текстом от бэкенда. */
export async function call<T>(cmd: string, args?: ApiArgs): Promise<T> {
  try {
    if (inTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(cmd, args);
    }
    return (await getFallback().call(cmd, args)) as T;
  } catch (e) {
    throw new Error(errorText(e));
  }
}

/** Подписка на событие бэкенда. Возвращает функцию отписки. */
export function listen<T>(event: string, cb: (payload: T) => void): Unsubscribe {
  if (!inTauri()) return getFallback().listen(event, cb);
  let unlisten: (() => void) | null = null;
  let cancelled = false;
  void import('@tauri-apps/api/event').then(({ listen: tauriListen }) =>
    tauriListen<T>(event, e => cb(e.payload)).then(u => {
      if (cancelled) u();
      else unlisten = u;
    })
  );
  return () => {
    cancelled = true;
    unlisten?.();
  };
}
