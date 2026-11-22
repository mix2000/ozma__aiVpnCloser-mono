import { plural } from 'common/utils';

import {
  CONSOLE_TARGETS,
  HELPER_FORMS,
  NOT_RUNNING_TEXT,
  SESSION_FORMS,
  TARGET_KINDS,
} from '../agents.consts';
import type { IAgent, TargetKind, Targets } from 'common/types';
import type { ITargetCount } from './models';

export function sessionsText(n: number): string {
  return `${n} ${plural(n, SESSION_FORMS)}`;
}

export function helpersText(n: number): string {
  return `+${n} ${plural(n, HELPER_FORMS)}`;
}

/** Процесс — сама сессия агента? Нет признака — считаем сессией (agents в IAppState — только корни). */
export function isRoot(a: Pick<IAgent, 'root'>): boolean {
  return a.root !== false;
}

/** Сессии и вспомогательные процессы по каждой цели. */
export function countByTarget(
  agents: Pick<IAgent, 'target' | 'root'>[]
): Record<TargetKind, ITargetCount> {
  const out = Object.fromEntries(TARGET_KINDS.map(t => [t, { sessions: 0, helpers: 0 }])) as Record<
    TargetKind,
    ITargetCount
  >;
  for (const a of agents) {
    if (isRoot(a)) out[a.target].sessions += 1;
    else out[a.target].helpers += 1;
  }
  return out;
}

/** «15 сессий (+93 вспомогательных процесса)». */
export function targetCountText(c: ITargetCount): string {
  if (c.sessions === 0 && c.helpers === 0) return NOT_RUNNING_TEXT;
  const base = sessionsText(c.sessions);
  return c.helpers ? `${base} (+${c.helpers} ${plural(c.helpers, HELPER_FORMS)})` : base;
}

/** Число потомков у каждой сессии (по rootPid). */
export function helpersBySession(agents: Pick<IAgent, 'root' | 'rootPid'>[]): Map<number, number> {
  const m = new Map<number, number>();
  for (const a of agents) {
    if (isRoot(a) || a.rootPid == null) continue;
    m.set(a.rootPid, (m.get(a.rootPid) ?? 0) + 1);
  }
  return m;
}

/**
 * Кого затронет закрытие: найденное `test_targets` по включённым целям; пока
 * идёт заморозка, признак frozen берём из состояния монитора.
 */
export function affectedAgents(found: IAgent[], stateAgents: IAgent[], targets: Targets): IAgent[] {
  const frozen = new Set(stateAgents.filter(a => a.frozen).map(a => a.pid));
  return found
    .filter(a => targets[a.target])
    .map(a => (frozen.has(a.pid) ? { ...a, frozen: true } : a));
}

/** Выбрана ли хоть одна консольная программа (Claude Code, Codex, Grok). */
export function hasConsoleTarget(targets: Targets): boolean {
  return CONSOLE_TARGETS.some(t => targets[t]);
}
