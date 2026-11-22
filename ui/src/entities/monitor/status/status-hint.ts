import type { IAppState } from 'common/types';
import { isRoot, sessionsText } from 'entities/agents';

import { STATUS_HINTS as H } from './status.consts';

/** Пояснение под статусом (цвет и заголовок считает ядро: state.color, state.title). */
export function statusHint(state: IAppState): string {
  if (!state.monitoring) return H.off;
  const frozen = state.agents.filter(a => a.frozen && isRoot(a)).length;
  const pref = state.dryRun ? H.dryRun : '';
  switch (state.status) {
    case 'ok':
      return `${pref}${H.ok(sessionsText(state.agents.filter(isRoot).length))}`;
    case 'countdown':
      return `${pref}${frozen ? H.frozen(sessionsText(frozen)) : ''}${H.countdown}`;
    case 'down':
      return `${pref}${H.down}`;
    case 'closed':
      return `${pref}${H.closed}`;
    default:
      return pref.trim();
  }
}
