import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invoke = vi.fn();
const unlisten = vi.fn();
let handler: ((e: { payload: unknown }) => void) | null = null;

vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock('@tauri-apps/api/event', () => ({
  listen: (_event: string, cb: (e: { payload: unknown }) => void) => {
    handler = cb;
    return Promise.resolve(unlisten);
  },
}));

import { call, inTauri, listen } from './api.instance';

describe('транспорт в окне Tauri', () => {
  beforeEach(() => {
    window.__TAURI_INTERNALS__ = {};
    invoke.mockReset();
    unlisten.mockReset();
    handler = null;
  });
  afterEach(() => {
    delete window.__TAURI_INTERNALS__;
  });

  it('команда — через invoke, ошибка строкой → Error', async () => {
    expect(inTauri()).toBe(true);
    invoke.mockResolvedValueOnce({ ok: 1 });
    await expect(call('get_state', { a: 1 })).resolves.toEqual({ ok: 1 });
    expect(invoke).toHaveBeenCalledWith('get_state', { a: 1 });
    invoke.mockRejectedValueOnce('Неверный адрес');
    await expect(call('save_config')).rejects.toThrow('Неверный адрес');
  });

  it('событие — через listen; отписка до и после подписки', async () => {
    const seen: unknown[] = [];
    const off = listen<string>('state-changed', p => seen.push(p));
    await vi.waitFor(() => expect(handler).not.toBeNull());
    handler?.({ payload: 'x' });
    expect(seen).toEqual(['x']);
    off();
    expect(unlisten).toHaveBeenCalledTimes(1);

    const early = listen('state-changed', () => undefined);
    early();
    await vi.waitFor(() => expect(unlisten).toHaveBeenCalledTimes(2));
  });
});
