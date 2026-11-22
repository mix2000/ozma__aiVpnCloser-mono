import { describe, expect, it } from 'vitest';

import type { WatchItem } from 'common/types';
import { watchKey } from '../watch/watch-items';
import type { IDiscovery } from 'common/types';
import {
  missingInterfaces,
  missingProcesses,
  pickerInterfaces,
  pickerProcesses,
} from './picker-lists';

const found: IDiscovery = {
  interfaces: [
    { name: 'en0', addrs: ['192.168.1.20/24'], up: true, tunnel: false },
    { name: 'utun10', addrs: [], up: true, tunnel: true },
    { name: 'utun9', addrs: [], up: true, tunnel: true },
  ],
  processes: [
    { name: 'xray', pid: 1, knownVpn: true },
    { name: 'Xray', pid: 2, knownVpn: true },
    { name: 'zsh', pid: 3, knownVpn: false },
  ],
  systemVpns: [],
};

describe('окно выбора', () => {
  it('интерфейсы: туннели, выбранные и «все»', () => {
    const none = new Set<string>();
    expect(pickerInterfaces(found, none, false).map(i => i.name)).toEqual(['utun9', 'utun10']);
    const withEn0 = new Set([watchKey({ kind: 'interface', name: 'en0' })]);
    expect(pickerInterfaces(found, withEn0, false).map(i => i.name)).toEqual([
      'utun9',
      'utun10',
      'en0',
    ]);
    expect(pickerInterfaces(found, none, true)).toHaveLength(3);
    expect(pickerInterfaces(null, none, true)).toEqual([]);
  });

  it('процессы: по одному на имя, «все» — с обычными', () => {
    const none = new Set<string>();
    expect(pickerProcesses(found, none, false).map(p => p.pid)).toEqual([1]);
    expect(pickerProcesses(found, none, true).map(p => p.pid)).toEqual([1, 3]);
    const zsh = new Set([watchKey({ kind: 'process', name: 'ZSH' })]);
    expect(pickerProcesses(found, zsh, false).map(p => p.pid)).toEqual([1, 3]);
  });

  it('выбранное, которого сейчас нет', () => {
    const watch: WatchItem[] = [
      { kind: 'interface', name: 'utun233', addr: '172.19.0.1' },
      { kind: 'interface', name: 'en0' },
      { kind: 'process', name: 'XRAY' },
      { kind: 'process', name: 'sing-box' },
      { kind: 'route' },
    ];
    expect(missingInterfaces(watch, found).map(w => w.name)).toEqual(['utun233']);
    expect(missingProcesses(watch, found).map(w => w.name)).toEqual(['sing-box']);
  });
});
