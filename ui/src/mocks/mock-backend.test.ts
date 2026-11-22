import { describe, expect, it } from 'vitest';

import type { IAppState, IConfig } from 'common/types';

import { MockBackend } from './mock-backend';

describe('мок бэкенда', () => {
  it('по умолчанию: только смотреть, наблюдение выключено', async () => {
    const m = new MockBackend();
    const s = (await m.call('get_state')) as IAppState;
    expect(s.dryRun).toBe(true);
    expect(s.monitoring).toBe(false);
    expect(s.status).toBe('paused');
  });

  it('валидация save_config — ошибка строкой', async () => {
    const m = new MockBackend();
    const c = (await m.call('get_config')) as IConfig;
    await expect(m.call('save_config', { config: { ...c, closeAfterSec: 9999 } })).rejects.toMatch(
      /600/
    );
  });

  it('VPN пропал → отсчёт, вернулся → ok', async () => {
    const m = new MockBackend();
    const c = (await m.call('get_config')) as IConfig;
    await m.call('save_config', { config: { ...c, watch: [{ kind: 'route' }] } });
    const seen: IAppState[] = [];
    m.subscribe(s => seen.push(s));
    await m.call('set_monitoring', { on: true });
    expect(seen.at(-1)?.status).toBe('ok');
    m.setVpn(false);
    expect(seen.at(-1)?.status).toBe('countdown');
    expect(seen.at(-1)?.countdownLeftSec).toBe(30);
    m.setVpn(true);
    expect(seen.at(-1)?.status).toBe('ok');
  });
});
