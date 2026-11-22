import type { IEventTag } from './models';

/** Сколько последних событий показывать. */
export const EVENTS_LIMIT = 200;

/** Метки видов событий из ядра (crates/core/src/models/event.rs → EventKind). */
export const EVENT_TAGS: Record<string, IEventTag> = {
  vpn_down: { label: 'VPN пропал', color: 'red' },
  vpn_up: { label: 'VPN есть', color: 'green' },
  freeze: { label: 'заморожено', color: 'gold' },
  resume: { label: 'продолжили', color: 'green' },
  close: { label: 'закрыто', color: 'red' },
  block: { label: 'блокировка', color: 'volcano' },
  dry_run: { label: 'только смотреть', color: 'blue' },
  error: { label: 'ошибка', color: 'red' },
  config: { label: 'настройки', color: 'default' },
};

export const JOURNAL_TEXTS = {
  title: 'Что происходило',
  refresh: 'Обновить',
  empty: 'Пока пусто',
};
