import type { WatchMode } from 'common/types';

export const WATCH_MODE_TEXTS = {
  title: 'Когда VPN считается включённым',
  hint: '«Все» надёжнее: агентов закрою, если отвалится хоть что-то из выбранного.',
};

export const WATCH_MODE_OPTIONS: { value: WatchMode; label: string }[] = [
  { value: 'all', label: 'Все пункты в порядке' },
  { value: 'any', label: 'Хотя бы один пункт в порядке' },
];
