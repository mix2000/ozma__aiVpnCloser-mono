import { useMemo, useState } from 'react';

import type { IConfig, WatchItem } from 'common/types';

import { DEFAULT_HOSTS } from '../vpn.consts';
import { parseHosts, toggleWatch, watchKey } from '../watch/watch-items';

/** Состояние окна выбора: отмеченные пункты и текст хостов. */
export function usePickerState(config: IConfig) {
  const [watch, setWatch] = useState<WatchItem[]>(config.watch);
  const [hostsText, setHostsText] = useState(config.hosts.join('\n'));
  const selected = useMemo(() => new Set(watch.map(watchKey)), [watch]);
  const hosts = parseHosts(hostsText);
  return {
    watch,
    setWatch,
    selected,
    toggle: (item: WatchItem, on: boolean) => setWatch(w => toggleWatch(w, item, on)),
    hostsText,
    setHostsText,
    resetHosts: () => setHostsText(DEFAULT_HOSTS.join('\n')),
    /* Пустой список — стандартные хосты. */
    hostsToSave: hosts.length ? hosts : DEFAULT_HOSTS,
    hostsCount: hosts.length,
  };
}
