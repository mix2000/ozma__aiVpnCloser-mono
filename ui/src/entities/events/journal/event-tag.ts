import { EVENT_TAGS } from './journal.consts';
import type { IEventTag } from './models';

/** Метка для вида события в журнале. Неизвестные виды (и info) — без метки. */
export function eventTag(kind: string): IEventTag | null {
  return EVENT_TAGS[kind] ?? null;
}

/** Ключ строки журнала. */
export function eventKey(e: { at: string; kind: string; message: string }): string {
  return `${e.at}-${e.kind}-${e.message}`;
}
