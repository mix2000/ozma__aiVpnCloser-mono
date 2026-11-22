import { describe, expect, it } from 'vitest';

import { eventKey, eventTag } from './event-tag';

describe('журнал', () => {
  it('метки событий', () => {
    expect(eventTag('vpn_down')?.label).toBe('VPN пропал');
    expect(eventTag('vpn_up')?.label).toBe('VPN есть');
    expect(eventTag('close')?.label).toBe('закрыто');
    expect(eventTag('info')).toBeNull();
    expect(eventTag('something')).toBeNull();
  });

  it('ключ строки', () => {
    expect(eventKey({ at: 't', kind: 'k', message: 'm' })).toBe('t-k-m');
  });
});
