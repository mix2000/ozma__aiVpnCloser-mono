import { describe, expect, it } from 'vitest';

import { ownerText } from './owner-text';

describe('подпись владельца туннеля', () => {
  it('точно и вероятно', () => {
    expect(ownerText({ name: 'sing-box', pid: 812, confidence: 'exact' })).toBe('(sing-box)');
    expect(ownerText({ name: 'OpenVPN Connect', confidence: 'likely' })).toBe(
      '(вероятно OpenVPN Connect)'
    );
  });
});
