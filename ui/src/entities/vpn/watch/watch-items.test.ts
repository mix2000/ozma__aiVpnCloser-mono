import { describe, expect, it } from 'vitest';

import type { IDiscovery } from 'common/types';
import type { WatchItem } from 'common/types';
import {
  parseHosts,
  primaryIpv4,
  suggestWatch,
  toggleWatch,
  watchKey,
  watchTitle,
} from './watch-items';

describe('пункты наблюдения', () => {
  const iface: WatchItem = { kind: 'interface', name: 'utun233', addr: '172.19.0.1/30' };

  it('ключи и названия', () => {
    expect(watchKey(iface)).toBe('interface:utun233');
    expect(watchKey({ kind: 'process', name: 'Sing-Box' })).toBe('process:sing-box');
    expect(watchKey({ kind: 'system_vpn', id: 'X', name: 'V2BOX' })).toBe('system_vpn:X');
    expect(watchKey({ kind: 'route' })).toBe('route');
    expect(watchTitle(iface)).toBe('Интерфейс utun233 (172.19.0.1/30)');
    expect(watchTitle({ kind: 'route' })).toBe('Трафик к агентам идёт через VPN');
  });

  it('toggle без дублей', () => {
    let list = toggleWatch([], iface, true);
    list = toggleWatch(list, { ...iface, addr: '172.19.0.2/30' }, true);
    expect(list).toHaveLength(1);
    expect(list[0]).toMatchObject({ addr: '172.19.0.2/30' });
    list = toggleWatch(list, { kind: 'route' }, true);
    expect(toggleWatch(list, iface, false)).toEqual([{ kind: 'route' }]);
  });

  it('первый IPv4', () => {
    expect(primaryIpv4(['fe80::1/64', '10.0.0.2/8'])).toBe('10.0.0.2/8');
    expect(primaryIpv4(['fe80::1/64'])).toBeUndefined();
    expect(primaryIpv4(['10.8.0.6'])).toBe('10.8.0.6');
  });

  it('рекомендуемое: туннели с IPv4 + route', () => {
    const d: IDiscovery = {
      interfaces: [
        { name: 'en0', addrs: ['192.168.1.2/24'], up: true, tunnel: false },
        { name: 'utun233', addrs: ['172.19.0.1/30'], up: true, tunnel: true },
        { name: 'utun0', addrs: ['fe80::1/64'], up: true, tunnel: true },
        { name: 'utun9', addrs: ['10.9.0.1/24'], up: false, tunnel: true },
      ],
      processes: [],
      systemVpns: [],
    };
    expect(suggestWatch(d)).toEqual([
      { kind: 'interface', name: 'utun233', addr: '172.19.0.1/30' },
      { kind: 'route' },
    ]);
  });
});

describe('хосты', () => {
  it('разбор текста', () => {
    expect(
      parseHosts(' api.anthropic.com\nhttps://Claude.ai/login, claude.ai;  grok.com ')
    ).toEqual(['api.anthropic.com', 'claude.ai', 'grok.com']);
    expect(parseHosts('   ')).toEqual([]);
  });
});
