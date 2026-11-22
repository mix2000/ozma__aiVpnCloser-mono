import { describe, expect, it } from 'vitest';

import type { WatchItem } from 'common/types';
import { watchRows } from './watch-rows';

describe('строки «За чем слежу»', () => {
  it('проверка из состояния или «ещё нет»', () => {
    const watch: WatchItem[] = [{ kind: 'route' }, { kind: 'process', name: 'xray' }];
    const checks = [
      { item: { kind: 'route' } as WatchItem, ok: true, label: 'Трафик', detail: 'ок' },
    ];
    const rows = watchRows(watch, checks, true);
    expect(rows[0]).toBe(checks[0]);
    expect(rows[1]).toMatchObject({ ok: null, detail: 'проверяю…' });
    expect(watchRows(watch, [], false)[0].detail).toBe(
      'проверка начнётся, когда включите «Следить»'
    );
  });
});
