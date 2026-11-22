import { describe, expect, it } from 'vitest';

import type { IAgent, Targets } from 'common/types';
import {
  affectedAgents,
  countByTarget,
  hasConsoleTarget,
  helpersBySession,
  sessionsText,
  targetCountText,
} from './agent-counts';

const ALL: Targets = {
  claudeCode: true,
  claudeDesktop: true,
  codexCli: true,
  codexDesktop: true,
  grok: true,
};

describe('цели', () => {
  it('счёт по целям: сессии отдельно от вспомогательных', () => {
    const c = countByTarget([
      { target: 'claudeCode', root: true },
      { target: 'claudeCode', root: true },
      { target: 'claudeCode', root: false },
      { target: 'claudeCode', root: false },
      { target: 'claudeCode', root: false },
      { target: 'grok' },
    ]);
    expect(c.claudeCode).toEqual({ sessions: 2, helpers: 3 });
    expect(c.grok).toEqual({ sessions: 1, helpers: 0 });
    expect(c.codexCli).toEqual({ sessions: 0, helpers: 0 });
  });

  it('текст по цели', () => {
    expect(targetCountText({ sessions: 15, helpers: 93 })).toBe(
      '15 сессий (+93 вспомогательных процесса)'
    );
    expect(targetCountText({ sessions: 1, helpers: 1 })).toBe(
      '1 сессия (+1 вспомогательный процесс)'
    );
    expect(targetCountText({ sessions: 2, helpers: 0 })).toBe('2 сессии');
    expect(targetCountText({ sessions: 11, helpers: 12 })).toBe(
      '11 сессий (+12 вспомогательных процессов)'
    );
    expect(targetCountText({ sessions: 0, helpers: 0 })).toBe('сейчас не запущен');
  });

  it('потомки по сессиям', () => {
    const m = helpersBySession([
      { root: true, rootPid: 10 },
      { root: false, rootPid: 10 },
      { root: false, rootPid: 10 },
      { root: false, rootPid: 20 },
    ]);
    expect(m.get(10)).toBe(2);
    expect(m.get(20)).toBe(1);
    expect(sessionsText(15)).toBe('15 сессий');
  });

  it('затронутые: только включённые цели, frozen — из состояния', () => {
    const found: IAgent[] = [
      { target: 'claudeCode', pid: 1, name: 'claude', frozen: false, root: true, rootPid: 1 },
      { target: 'grok', pid: 2, name: 'grok', frozen: false, root: true, rootPid: 2 },
    ];
    const state: IAgent[] = [{ target: 'claudeCode', pid: 1, name: 'claude', frozen: true }];
    const out = affectedAgents(found, state, { ...ALL, grok: false });
    expect(out).toHaveLength(1);
    expect(out[0].frozen).toBe(true);
    expect(found[0].frozen).toBe(false);
  });

  it('консольные цели', () => {
    expect(hasConsoleTarget(ALL)).toBe(true);
    expect(hasConsoleTarget({ ...ALL, claudeCode: false, codexCli: false, grok: false })).toBe(
      false
    );
  });
});
