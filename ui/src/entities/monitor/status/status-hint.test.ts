import { describe, expect, it } from 'vitest';

import type { IAppState } from 'common/types';
import { statusHint } from './status-hint';

const base: IAppState = {
  monitoring: true,
  dryRun: false,
  status: 'ok',
  countdownLeftSec: null,
  checks: [],
  agents: [],
  lastChangeAt: null,
  color: 'gray',
  title: 'Наблюдение выключено',
};

describe('пояснение под статусом', () => {
  it('по статусу, «только смотреть» и замороженным', () => {
    expect(statusHint({ ...base, monitoring: false })).toBe(
      'Включите «Следить», чтобы программа закрывала агентов без VPN.'
    );
    const agents = [
      { target: 'claudeCode' as const, pid: 1, name: 'claude', frozen: true },
      { target: 'claudeCode' as const, pid: 2, name: 'claude', frozen: false },
    ];
    expect(statusHint({ ...base, agents })).toBe('Сейчас работает: 2 сессии агентов.');
    expect(statusHint({ ...base, status: 'countdown', dryRun: true, agents })).toBe(
      'Только смотреть: ничего не трогаю, пишу в журнал. Заморожено: 1 сессия. Если VPN вернётся — агенты продолжат работу.'
    );
    expect(statusHint({ ...base, status: 'unknown', dryRun: true })).toBe(
      'Только смотреть: ничего не трогаю, пишу в журнал.'
    );
    expect(statusHint({ ...base, status: 'unknown' })).toBe('');
  });

  it('пропадает и закрыты', () => {
    expect(statusHint({ ...base, status: 'down' })).toBe(
      'Проверки не проходят — перепроверяю, прежде чем действовать.'
    );
    expect(statusHint({ ...base, status: 'closed', dryRun: true })).toBe(
      'Только смотреть: ничего не трогаю, пишу в журнал. Новые агенты будут закрываться, пока VPN не вернётся.'
    );
  });
});
