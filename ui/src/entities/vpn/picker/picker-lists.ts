// Какие строки показать в окне выбора «что считать VPN».
import type { IDiscoveredInterface, IDiscoveredProcess, IDiscovery } from 'common/types';
import type { InterfaceWatchItem, ProcessWatchItem, WatchItem } from 'common/types';
import { interfaceItem, watchKey } from '../watch/watch-items';

/** Туннельные (или все) интерфейсы + уже выбранные; туннели первыми. */
export function pickerInterfaces(
  found: IDiscovery | null,
  selected: Set<string>,
  showAll: boolean
): IDiscoveredInterface[] {
  return (found?.interfaces ?? [])
    .filter(i => showAll || i.tunnel || selected.has(watchKey(interfaceItem(i))))
    .sort(
      (a, b) =>
        Number(b.tunnel) - Number(a.tunnel) || a.name.localeCompare(b.name, 'en', { numeric: true })
    );
}

/** Выбранные интерфейсы, которых сейчас нет на машине. */
export function missingInterfaces(
  watch: WatchItem[],
  found: IDiscovery | null
): InterfaceWatchItem[] {
  return watch.filter(
    (w): w is InterfaceWatchItem =>
      w.kind === 'interface' && !(found?.interfaces ?? []).some(i => i.name === w.name)
  );
}

/** Похожие на VPN (или все) процессы + уже выбранные, по одному на имя. */
export function pickerProcesses(
  found: IDiscovery | null,
  selected: Set<string>,
  showAll: boolean
): IDiscoveredProcess[] {
  const procs = (found?.processes ?? []).filter(
    p => showAll || p.knownVpn || selected.has(watchKey({ kind: 'process', name: p.name }))
  );
  return procs.filter(
    (p, i) => procs.findIndex(q => q.name.toLowerCase() === p.name.toLowerCase()) === i
  );
}

/** Выбранные программы, которые сейчас не запущены. */
export function missingProcesses(watch: WatchItem[], found: IDiscovery | null): ProcessWatchItem[] {
  const running = new Set((found?.processes ?? []).map(p => p.name.toLowerCase()));
  return watch.filter(
    (w): w is ProcessWatchItem => w.kind === 'process' && !running.has(w.name.toLowerCase())
  );
}
