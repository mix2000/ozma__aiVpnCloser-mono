import { describe, expect, it } from 'vitest';

import { clampInt, formatTime, plural, type PluralForms } from '.';

describe('common/utils', () => {
  it('склонения', () => {
    const f: PluralForms = ['сессия', 'сессии', 'сессий'];
    expect([1, 2, 5, 11, 12, 14, 21, 22, 25, 101, 111].map(n => plural(n, f))).toEqual([
      'сессия',
      'сессии',
      'сессий',
      'сессий',
      'сессий',
      'сессий',
      'сессия',
      'сессии',
      'сессий',
      'сессия',
      'сессий',
    ]);
  });

  it('время: сегодня без даты, раньше — с датой', () => {
    const now = new Date('2026-11-22T15:00:00');
    expect(formatTime('2026-11-22T10:20:30', now)).toBe('10:20:30');
    expect(formatTime('2026-11-21T10:20:30', now)).toBe('21.11 10:20:30');
    expect(formatTime('не дата', now)).toBe('не дата');
  });

  it('целое в пределах', () => {
    expect(clampInt(700, 0, 600)).toBe(600);
    expect(clampInt(-3, 0, 600)).toBe(0);
    expect(clampInt(12.6, 0, 600)).toBe(13);
    expect(clampInt(null, 0, 600)).toBe(0);
  });
});
