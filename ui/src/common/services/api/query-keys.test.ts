import { describe, expect, it } from 'vitest';

import { apiService } from './api.service';
import { setFallbackBackend } from './api.instance';
import { queryKeys } from './query-keys';

describe('ключи и сервис API', () => {
  it('ключи стабильны', () => {
    expect(queryKeys.state).toEqual(['state']);
    expect(queryKeys.events).toEqual(['events']);
    expect(new Set(Object.values(queryKeys).map(k => k[0])).size).toBe(
      Object.keys(queryKeys).length
    );
  });

  it('команды и подписки идут через транспорт', async () => {
    const seen: unknown[] = [];
    setFallbackBackend({
      call: async (cmd, args) => ({ cmd, args }),
      listen: (event, cb) => {
        (cb as (p: unknown) => void)(event);
        return () => undefined;
      },
    });
    await expect(apiService.command('set_monitoring', { on: true })).resolves.toEqual({
      cmd: 'set_monitoring',
      args: { on: true },
    });
    apiService.subscribe('state-changed', p => seen.push(p));
    expect(seen).toEqual(['state-changed']);
    setFallbackBackend(null);
  });
});
