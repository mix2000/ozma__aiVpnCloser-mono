import { DEFAULT_HOSTS } from 'entities/vpn';

import type { IConfig } from 'common/types';

/** Настройки первого запуска (как IConfig::default в ядре). */
export const DEFAULT_CONFIG: IConfig = {
  watch: [],
  mode: 'all',
  hosts: DEFAULT_HOSTS,
  targets: {
    claudeCode: true,
    claudeDesktop: true,
    codexCli: true,
    codexDesktop: true,
    grok: true,
  },
  freezeImmediately: true,
  closeAfterSec: 30,
  killGraceSec: 5,
  blockWhileDown: true,
  pollMs: 1000,
  confirmFailures: 2,
  dryRun: true,
  notify: true,
  autostart: false,
  monitoring: false,
};
