import type { IDiscoveredInterface, IDiscovery } from 'common/types';
import type { WatchItem } from 'common/types';

import { WATCH_TITLES as T } from './watch.consts';

/** Ключ пункта: по нему сравниваем выбранное и найденное. */
export function watchKey(item: WatchItem): string {
  switch (item.kind) {
    case 'interface':
      return `interface:${item.name}`;
    case 'process':
      return `process:${item.name.toLowerCase()}`;
    case 'system_vpn':
      return `system_vpn:${item.id}`;
    case 'route':
      return 'route';
  }
}

/** Название пункта до первой проверки (потом — `ICheck.label` от ядра). */
export function watchTitle(item: WatchItem): string {
  switch (item.kind) {
    case 'interface':
      return T.interface(item.name, item.addr);
    case 'process':
      return T.process(item.name);
    case 'system_vpn':
      return T.systemVpn(item.name);
    case 'route':
      return T.route;
  }
}

/** IPv4-адрес интерфейса (без маски) — его и храним в пункте. */
export function primaryIpv4(addrs: string[]): string | undefined {
  for (const a of addrs) {
    const ip = a.split('/')[0];
    if (/^\d{1,3}(\.\d{1,3}){3}$/.test(ip)) return a;
  }
  return undefined;
}

export function interfaceItem(iface: IDiscoveredInterface): WatchItem {
  return { kind: 'interface', name: iface.name, addr: primaryIpv4(iface.addrs) };
}

/** Добавить или убрать пункт из выбранных. */
export function toggleWatch(list: WatchItem[], item: WatchItem, on: boolean): WatchItem[] {
  const key = watchKey(item);
  const rest = list.filter(w => watchKey(w) !== key);
  return on ? [...rest, item] : rest;
}

/**
 * Что предложить при первом запуске: работающие туннельные интерфейсы с
 * адресом и проверку маршрута к хостам агентов.
 */
export function suggestWatch(d: IDiscovery): WatchItem[] {
  const ifaces = d.interfaces
    .filter(i => i.tunnel && i.up && primaryIpv4(i.addrs))
    .map(interfaceItem);
  return [...ifaces, { kind: 'route' }];
}

/** Список хостов из текста: по строкам/запятым/пробелам, без дублей. */
export function parseHosts(text: string): string[] {
  const out: string[] = [];
  for (const raw of text.split(/[\s,;]+/)) {
    const h = raw
      .trim()
      .toLowerCase()
      .replace(/^https?:\/\//, '')
      .replace(/\/.*$/, '');
    if (h && !out.includes(h)) out.push(h);
  }
  return out;
}
