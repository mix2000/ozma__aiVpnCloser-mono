// Контракт ядра: состояние монитора (get_state, событие "state-changed").
import type { IAgent } from './agents';
import type { ICheck } from './vpn';

/* Статус наблюдения. */
export type Status = 'ok' | 'down' | 'countdown' | 'closed' | 'paused' | 'unknown';

/* Цвет статуса — тот же, что у иконки в трее (считает ядро). */
export type StatusColor = 'green' | 'yellow' | 'red' | 'gray';

/* Состояние монитора. */
export interface IAppState {
  monitoring: boolean;
  dryRun: boolean;
  status: Status;
  /* Секунд до закрытия, пока идёт отсчёт. */
  countdownLeftSec: number | null;
  /* Проверки по выбранным пунктам. */
  checks: ICheck[];
  /* Сессии агентов (только корни деревьев). */
  agents: IAgent[];
  lastChangeAt: string | null;
  /* Цвет и большая строка статуса — от ядра, одни для окна и трея. */
  color: StatusColor;
  title: string;
}
