import type { PluralForms } from 'common/utils';

import type { TargetKind } from 'common/types';

/** Цели в порядке показа. */
export const TARGET_KINDS: TargetKind[] = [
  'claudeCode',
  'claudeDesktop',
  'codexCli',
  'codexDesktop',
  'grok',
];

export const TARGET_LABELS: Record<TargetKind, string> = {
  claudeCode: 'Claude Code',
  claudeDesktop: 'Claude (приложение)',
  codexCli: 'Codex CLI',
  codexDesktop: 'Codex (приложение)',
  grok: 'Grok',
};

/** Консольные цели: без них агенты в терминале останутся без защиты. */
export const CONSOLE_TARGETS: TargetKind[] = ['claudeCode', 'codexCli', 'grok'];

/** Как часто пересчитывать, кого затронет закрытие. */
export const TARGETS_REFRESH_MS = 5000;

export const SESSION_FORMS: PluralForms = ['сессия', 'сессии', 'сессий'];
export const HELPER_FORMS: PluralForms = [
  'вспомогательный процесс',
  'вспомогательных процесса',
  'вспомогательных процессов',
];

export const NOT_RUNNING_TEXT = 'сейчас не запущен';
